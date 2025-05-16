use crate::cache::CodeFileCache;
use crate::model::{CodeExecutor, CodeScriptExecutionResult, CommandExecutor, LanguageScript};
use anyhow::{Context, Result};
use log::{debug, error, info};
use serde_json::Value;
use tokio::process::Command;

/// 通用的 Deno 脚本执行逻辑，供 JS/TS Runner 复用
pub async fn run_deno_script_with_params<F>(
    code: &str,
    params: Option<Value>,
    timeout_seconds: Option<u64>,
    lang: LanguageScript,
    prepare_code_fn: F,
) -> Result<CodeScriptExecutionResult>
where
    F: Fn(&str, bool) -> String,
{
    debug!("开始执行{:?}脚本...,执行参数: {:?}", lang, params);

    let hash = CodeFileCache::obtain_code_hash(code);
    let cache_exist = CodeFileCache::check_code_file_cache_exisht(&hash, &lang).await;

    let run_code_script_file_tuple = if cache_exist {
        let cache_code = CodeFileCache::get_code_file_cache(&hash, &lang).await;
        debug!("从缓存中读取代码:hash值 {:?}", &hash);
        cache_code?
    } else {
        let wrapped_code = prepare_code_fn(code, true);
        CodeFileCache::save_code_file_cache(&hash, &wrapped_code, &lang).await?;
        let code_script_file_tuple = CodeFileCache::get_code_file_cache(&hash, &lang).await?;
        debug!("创建脚本缓存:hash值 {:?}", &hash);
        code_script_file_tuple
    };

    let temp_path = run_code_script_file_tuple.1;

    let params_json = match params {
        Some(p) => serde_json::to_string(&p)?,
        None => "{}".to_string(),
    };

    let mut execute_command = Command::new("deno");
    execute_command
        .arg("run")
        .arg("--allow-net")
        .arg("--allow-env")
        .arg("--allow-read")
        .arg("--no-check")
        .arg("--v8-flags=--max-heap-size=512")
        .env("INPUT_JSON", &params_json)
        .arg(&temp_path)
        .kill_on_drop(true);

    debug!("Deno命令[{:?}]: {:?}", lang, &execute_command);

    let executor = match timeout_seconds {
        Some(timeout) => CommandExecutor::with_timeout(execute_command.output(), timeout),
        None => CommandExecutor::default(execute_command.output()),
    };
    info!("执行命令: {:?}", &execute_command);

    let executor_result = executor.await;
    let output = match executor_result {
        Ok(cmd_result) => match cmd_result {
            Ok(output) => output,
            Err(e) => {
                error!("Deno命令执行失败 [{:?}]: {:?}", lang, e);
                return Err(e).context(format!("Failed to execute {:?} with Deno", lang));
            }
        },
        Err(e) => {
            error!("Deno任务执行异常 [{:?}]: {:?}", lang, e);
            return Err(e).context(format!("Deno executor await error for {:?}", lang));
        }
    };
    debug!("标准输出:\n{}", String::from_utf8_lossy(&output.stdout));
    debug!("错误输出:\n{}", String::from_utf8_lossy(&output.stderr));

    CodeExecutor::parse_execution_output(&output.stdout, &output.stderr).await
}
