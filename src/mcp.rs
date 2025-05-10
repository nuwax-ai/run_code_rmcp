use std::process::Command;
use std::io::Write;

use anyhow::{Result, Context};
use log::debug;
use serde::{Serialize, Deserialize};
use tempfile::NamedTempFile;

#[derive(Serialize, Deserialize, Debug)]
pub struct CodeExecutionParams {
    pub code: String,
    pub language: String,
    pub show_logs: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CodeExecutionResult {
    pub logs: Vec<String>,
    pub result: Option<String>,
    pub error: Option<String>,
}

pub async fn execute_code_with_mcp(params: CodeExecutionParams) -> Result<CodeExecutionResult> {
    match params.language.to_lowercase().as_str() {
        "js" | "javascript" => {
            // Create a wrapper for JS with MCP SDK
            let mcp_js_wrapper = format!(r#"
// MCP SDK integration for JavaScript
const mcp = require('mcp');

// Log capture setup
const logs = [];
const originalConsoleLog = console.log;
console.log = function() {{
    const message = Array.from(arguments).map(arg => 
        typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
    ).join(' ');
    logs.push(message);
    if ({}) {{
        originalConsoleLog.apply(console, arguments);
    }}
}};

async function mcpExecute() {{
    try {{
        // User code
        {}

        // Execute handler
        let result = null;
        if (typeof handler === 'function') {{
            result = await handler();
        }}

        return {{
            logs: logs,
            result: result !== undefined ? String(result) : null,
            error: null
        }};
    }} catch (error) {{
        return {{
            logs: logs,
            result: null,
            error: error.toString()
        }};
    }}
}}

// Execute and return results
mcpExecute().then(result => {{
    originalConsoleLog(JSON.stringify(result));
}});
"#, params.show_logs, params.code);
            
            // Execute with MCP SDK
            let result = execute_with_mcp(mcp_js_wrapper, "js").await?;
            Ok(result)
        },
        "py" | "python" => {
            // Create a wrapper for Python with MCP SDK
            let show_logs_value = if params.show_logs { "True" } else { "False" };
            let mcp_py_wrapper = format!(r#"
# MCP SDK integration for Python
import mcp
import sys
import json
import io
from contextlib import redirect_stdout

# Store logs
logs = []

# Create custom stdout to capture logs
class LogCapture(io.StringIO):
    def write(self, text):
        if text.strip():
            logs.append(text.rstrip())
            if {}:
                sys.__stdout__.write(text)

async def mcp_execute():
    try:
        # User code
{}

        # Execute handler function and capture result
        result = None
        with redirect_stdout(LogCapture()):
            if 'handler' in globals() and callable(handler):
                result = await handler()
        
        return {{
            "logs": logs,
            "result": str(result) if result is not None else None,
            "error": None
        }}
    except Exception as e:
        return {{
            "logs": logs,
            "result": None,
            "error": str(e)
        }}

# Execute and print results
import asyncio
result = asyncio.run(mcp_execute())
print(json.dumps(result))
"#, show_logs_value, params.code.replace('\n', "\n        "));
            
            // Execute with MCP SDK
            let result = execute_with_mcp(mcp_py_wrapper, "python").await?;
            Ok(result)
        },
        _ => {
            anyhow::bail!("Unsupported language: {}", params.language)
        }
    }
}

async fn execute_with_mcp(code: String, language: &str) -> Result<CodeExecutionResult> {
    // Create temporary file
    let mut temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path().to_path_buf();
    
    // 写入代码
    write!(temp_file.as_file_mut(), "{}", code)?;
    
    // 确保文件有执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        use std::fs;
        let mut perms = fs::metadata(&temp_path)?.permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(&temp_path, perms)?;
    }
    
    // Execute with the appropriate command
    let output = match language {
        "js" => {
            Command::new("deno")
                .arg("run")
                .arg("--allow-all")
                .arg(&temp_path)
                .output()
                .context("Failed to execute JavaScript with Deno")?
        },
        "python" => {
            // 使用uv run命令执行Python脚本，提供隔离环境
            Command::new("uv")
                .arg("run")
                .arg("-s")  // 明确指定作为脚本运行
                .arg("--isolated")  // 在隔离环境中运行
                .arg("-p")
                .arg("python3")  // 指定Python解释器
                .arg(&temp_path)
                .output()
                .context("Failed to execute Python with uv")?
        },
        _ => {
            anyhow::bail!("Unsupported language for execution: {}", language)
        }
    };
    
    // Parse output
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    // Try to parse as JSON
    if !stdout.is_empty() {
        for line in stdout.lines() {
            if line.trim().starts_with('{') && line.trim().ends_with('}') {
                match serde_json::from_str::<CodeExecutionResult>(line) {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        debug!("Failed to parse output as JSON: {}", e);
                        // Continue to next line
                    }
                }
            }
        }
    }
    
    // If no structured output found, return raw output
    Ok(CodeExecutionResult {
        logs: if !stdout.is_empty() { 
            stdout.lines().map(String::from).collect() 
        } else { 
            vec![] 
        },
        result: None,
        error: Some(format!("Error: {}", stderr)),
    })
} 