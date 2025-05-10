use std::process::Command;
use std::io::Write;
use std::path::PathBuf;
use std::fs;

use anyhow::{Result, Context};
use clap::{Parser, Subcommand, Args};
use serde::{Serialize, Deserialize};
use tempfile::NamedTempFile;
use regex::Regex;

mod mcp;
mod error;

use mcp::CodeExecutionParams;

#[derive(Parser)]
#[command(name = "run_code_rmcp")]
#[command(about = "Execute JavaScript and Python code using MCP SDK", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Show execution logs
    #[arg(short, long)]
    show_logs: bool,
    
    /// Use MCP SDK integration
    #[arg(short, long)]
    use_mcp: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute JavaScript code
    Js(CodeArgs),
    
    /// Execute Python code
    Python(CodeArgs),
}

#[derive(Args)]
struct CodeArgs {
    /// Path to the code file
    #[arg(short, long)]
    file: Option<PathBuf>,
    
    /// Code content as string
    #[arg(short, long)]
    code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExecutionResult {
    logs: Vec<String>,
    result: Option<String>,
    error: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Get code from args
    let (code, language) = match &cli.command {
        Commands::Js(args) => (get_code(args)?, "js"),
        Commands::Python(args) => (get_code(args)?, "python"),
    };
    
    // Execute code
    let result = if cli.use_mcp {
        // Use MCP SDK integration
        let params = CodeExecutionParams {
            code,
            language: language.to_string(),
            show_logs: cli.show_logs,
        };
        
        let mcp_result = mcp::execute_code_with_mcp(params).await
            .map_err(|e| anyhow::anyhow!("MCP execution error: {}", e))?;
        
        // Convert to ExecutionResult
        ExecutionResult {
            logs: mcp_result.logs,
            result: mcp_result.result,
            error: mcp_result.error,
        }
    } else {
        // Use direct execution
        match language {
            "js" => execute_js(&code, cli.show_logs)?,
            "python" => execute_python(&code, cli.show_logs)?,
            _ => unreachable!(),
        }
    };
    
    // Print result
    print_result(result);
    
    Ok(())
}

fn get_code(args: &CodeArgs) -> Result<String> {
    if let Some(file) = &args.file {
        fs::read_to_string(file).context("Failed to read code file")
    } else if let Some(code) = &args.code {
        Ok(code.clone())
    } else {
        anyhow::bail!("Either file or code must be provided")
    }
}

fn execute_js(code: &str, show_logs: bool) -> Result<ExecutionResult> {
    // Prepare the JavaScript code with wrapped handler
    let wrapped_code = prepare_js_code(code, show_logs);
    
    // Create a temporary file
    let mut temp_file = NamedTempFile::new()?;
    write!(temp_file.as_file_mut(), "{}", wrapped_code)?;
    
    // Execute with Deno
    let output = Command::new("deno")
        .arg("run")
        .arg("--allow-all")
        .arg(temp_file.path())
        .output()
        .context("Failed to execute Deno")?;
    
    // Parse the output
    parse_execution_output(&output.stdout, &output.stderr)
}

fn execute_python(code: &str, show_logs: bool) -> Result<ExecutionResult> {
    // Prepare the Python code with wrapped handler
    let wrapped_code = prepare_python_code(code, show_logs);
    
    // Create a temporary file
    let mut temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path().to_path_buf();
    
    // 写入代码
    write!(temp_file.as_file_mut(), "{}", wrapped_code)?;
    
    // 确保文件有执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)?.permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(&temp_path, perms)?;
    }
    
    // Print temporary file path for debugging
    println!("Temporary file path: {:?}", temp_path);
    
    // 使用uv run命令执行Python脚本，提供隔离环境
    let output = Command::new("uv")
        .arg("run")
        .arg("-s")  // 明确指定作为脚本运行
        .arg("--isolated")  // 在隔离环境中运行
        .arg("-p")
        .arg("python3")  // 指定Python解释器
        .arg(&temp_path)
        .output()
        .context("Failed to execute Python with uv")?;
    
    // Parse the output
    parse_execution_output(&output.stdout, &output.stderr)
}

fn prepare_js_code(code: &str, show_logs: bool) -> String {
    let wrapper = format!(r#"
// Save original console.log
const originalConsoleLog = console.log;
let logs = [];

// Replace console.log to capture logs
console.log = function() {{
    // Convert arguments to string and join them
    const message = Array.from(arguments).map(arg => 
        typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
    ).join(' ');
    
    // Store log
    logs.push(message);
    
    // Also log to original console if showing logs
    if ({}) {{
        originalConsoleLog.apply(console, arguments);
    }}
}};

try {{
    // Add the original code
    {}
    
    // Execute handler function and get result
    let result = null;
    if (typeof handler === 'function') {{
        result = handler();
    }}
    
    // Print final output as JSON
    originalConsoleLog(JSON.stringify({{
        logs: logs,
        result: result !== undefined ? String(result) : null,
        error: null
    }}));
}} catch (error) {{
    // Handle errors
    originalConsoleLog(JSON.stringify({{
        logs: logs,
        result: null,
        error: error.toString()
    }}));
}}
"#, show_logs, code);

    wrapper
}

fn prepare_python_code(code: &str, show_logs: bool) -> String {
    let show_logs_value = if show_logs { "True" } else { "False" };
    
    let wrapper = format!(r#"
import sys
import json
import io
from contextlib import redirect_stdout

# Store logs
logs = []

# Create a custom stdout to capture logs
class LogCapture(io.StringIO):
    def write(self, text):
        if text.strip():  # Skip empty lines
            logs.append(text.rstrip())
            if {}:  # show_logs flag
                sys.__stdout__.write(text)

# Original code
{}

# Execute handler function and capture result
result = None
error = None

try:
    # Capture all print statements during handler execution
    with redirect_stdout(LogCapture()):
        if 'handler' in globals() and callable(handler):
            result = handler()
        
    # Print final output as JSON
    print(json.dumps({{
        "logs": logs,
        "result": str(result) if result is not None else None,
        "error": None
    }}))
except Exception as e:
    # Handle errors
    print(json.dumps({{
        "logs": logs,
        "result": None,
        "error": str(e)
    }}))
"#, show_logs_value, code);

    wrapper
}

fn parse_execution_output(stdout: &[u8], stderr: &[u8]) -> Result<ExecutionResult> {
    let stdout_str = String::from_utf8_lossy(stdout).to_string();
    let stderr_str = String::from_utf8_lossy(stderr).to_string();
    
    // Try to find JSON output in stdout
    let json_pattern = r"\{[\s\S]*\}";
    let re = Regex::new(json_pattern)?;
    
    if let Some(captures) = re.find(&stdout_str) {
        let json_str = captures.as_str();
        let result: ExecutionResult = serde_json::from_str(json_str)
            .context("Failed to parse JSON output")?;
        return Ok(result);
    }
    
    // If no structured output found, return error with raw output
    Ok(ExecutionResult {
        logs: if !stdout_str.is_empty() { vec![stdout_str] } else { vec![] },
        result: None,
        error: Some(format!("Failed to extract structured output: {}", stderr_str)),
    })
}

fn print_result(result: ExecutionResult) {
    if !result.logs.is_empty() {
        println!("--- Logs ---");
        for log in result.logs {
            println!("{}", log);
        }
        println!("------------");
    }
    
    if let Some(result_str) = result.result {
        println!("Result: {}", result_str);
    }
    
    if let Some(error) = result.error {
        eprintln!("Error: {}", error);
    }
}
