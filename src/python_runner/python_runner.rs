//通过 uv 命令,来运行 python脚本
use crate::{
    cache::CodeFileCache,
    model::{
        CodeExecutor, CodeScriptExecutionResult, CommandExecutor, LanguageScript, RunCode,
        TokioHeapSize,
    },
    python_runner::parse_import,
};
use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde_json;
use tokio::process::Command;

#[derive(Default)]
pub struct PythonRunner;

impl RunCode for PythonRunner {
    async fn run_with_params(
        &self,
        code: &str,
        params: Option<serde_json::Value>,
        timeout_seconds: Option<u64>,
    ) -> Result<CodeScriptExecutionResult> {
        debug!("开始执行Python脚本...,执行参数: {:?}", params);
        // 根据 code ,获取对应的hash, 对用户脚本代码,使用胶水代码处理后,缓存到文件系统里,下次使用如果hash相同,直接使用
        let hash = CodeFileCache::obtain_code_hash(code);
        let cache_exist =
            CodeFileCache::check_code_file_cache_exisht(&hash, &LanguageScript::Python).await;

        let run_code_script_file_tuple = if cache_exist {
            // 从缓存中读取代码
            let cache_code =
                CodeFileCache::get_code_file_cache(&hash, &LanguageScript::Python).await;
            debug!("从缓存中读取代码:hash值 {:?}", &hash);
            cache_code?
        } else {
            // 分析用户python代码依赖
            let dependencies = parse_import(code)?;
            // 准备Python代码，添加日志捕获和handler函数执行逻辑
            let wrapped_code = self.prepare_python_code(code, true);
            // 缓存代码
            CodeFileCache::save_code_file_cache(&hash, &wrapped_code, &LanguageScript::Python)
                .await?;

            //保存后，获取文件
            let code_script_file_tuple =
                CodeFileCache::get_code_file_cache(&hash, &LanguageScript::Python).await?;
            let run_code_script_file_path = code_script_file_tuple.1.clone();
            // 按照 uv命令的规范,添加用户脚本所需的依赖
            // uv add --script example.py 'requests<3' 'rich'  这是添加依赖的参考命令,dependencies 是解析出来的依赖列表

            // 执行添加依赖命令
            if !dependencies.is_empty() {
                info!("正在添加依赖: {:?}", dependencies);
                let mut cmd = Command::new("uv");
                cmd.arg("add")
                    .arg("--script")
                    .arg(&run_code_script_file_path);

                // 为每个依赖添加一个参数
                for dep in &dependencies {
                    cmd.arg(dep);
                }
                // 打印 cmd 命令,可以直接复制执行的命令字符串
                let cmd_str = format!("{:?}", &cmd);
                info!("uv命令字符串: {}", cmd_str);

                let cmd_output = match cmd.kill_on_drop(true).output().await {
                    Ok(output) => output,
                    Err(e) => {
                        error!("安装Python依赖失败: {:?}", e);
                        error!("失败的命令: {:?}", cmd);
                        return Err(e).context("Failed to add dependencies with uv");
                    }
                };

                let stdout = String::from_utf8_lossy(&cmd_output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&cmd_output.stderr).to_string();
                info!("添加依赖结果 - stdout: {}", stdout);
                info!("添加依赖结果 - stderr: {}", stderr);

                if !cmd_output.status.success() {
                    warn!("添加依赖失败，状态码: {}", cmd_output.status);
                }
            }
            debug!("创建脚本缓存:hash值 {:?}", &hash);
            code_script_file_tuple
        };

        let temp_path = run_code_script_file_tuple.1;

        // 将参数序列化为JSON字符串
        let params_json = match params {
            Some(p) => serde_json::to_string(&p)?,
            None => "{}".to_string(),
        };

        // 使用uv run命令执行Python脚本，提供隔离环境
        let mut execute_command = Command::new("uv");
        execute_command
            .arg("run")
            .arg("-s") // 明确指定作为脚本运行
            .arg("-p")
            .arg("3.13") // 指定Python解释器版本3.13
            .env("INPUT_JSON", &params_json) // 通过环境变量传递参数
            .arg(&temp_path)
            .kill_on_drop(true);

        info!("执行命令: {:?}", &execute_command);

        // let tokio_child_command = TokioHeapSize::default();
        // // 设置堆大小限制
        // tokio_child_command
        //     .with_heap_limit(&mut execute_command)
        //     .await;

        //限制command 的执行超时时间
        let executor = match timeout_seconds {
            Some(timeout) => CommandExecutor::with_timeout(execute_command.output(), timeout),
            None => CommandExecutor::default(execute_command.output()),
        };

        let executor_result = executor.await;
        let output = match executor_result {
            Ok(cmd_result) => match cmd_result {
                Ok(output) => output,
                Err(e) => {
                    error!("Python命令执行失败: {:?}", e);
                    return Err(e.into());
                }
            },
            Err(e) => {
                error!("Python任务执行异常: {:?}", e);
                return Err(e.into());
            }
        };
        // 调试输出
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        debug!("Python stdout: {}", stdout);
        debug!("Python stderr: {}", stderr);

        // 解析输出
        CodeExecutor::parse_execution_output(&output.stdout, &output.stderr).await
    }
}

impl PythonRunner {
    /// 准备Python代码，添加日志捕获和handler函数执行逻辑
    fn prepare_python_code(&self, code: &str, show_logs: bool) -> String {
        let show_logs_value = if show_logs { "True" } else { "False" };

        let template = include_str!("../templates/python_template.py");

        template
            .replace("{{USER_CODE}}", code)
            .replace("{{SHOW_LOGS}}", show_logs_value)
    }
}
