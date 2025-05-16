use crate::model::CommandExecutor;
use anyhow::{Context, Result};
use log::{info, warn};
use tokio::process::Command;

/// 预热Python环境，安装常用依赖
async fn warm_up_python_env(custom_deps: Option<Vec<String>>) -> Result<()> {
    info!("开始预热Python环境...");

    // 默认Python依赖列表
    let default_deps = [
        "requests",
        "pandas",
        "numpy",
        "matplotlib",
        "scikit-learn",
        "pytest",
        "pydantic",
        "fastapi",
        "uvicorn",
        "sqlalchemy",
    ];

    // 使用自定义依赖或默认依赖
    let deps_to_install = if let Some(deps) = custom_deps {
        info!("使用自定义Python依赖列表");
        deps
    } else {
        info!("使用默认Python依赖列表");
        default_deps.iter().map(|&s| s.to_string()).collect()
    };

    let total_deps = deps_to_install.len();
    info!("总共需要预热 {} 个Python依赖", total_deps);

    // 使用uv安装依赖
    for (index, dep) in deps_to_install.iter().enumerate() {
        let progress = ((index + 1) as f32 / total_deps as f32 * 100.0) as u32;
        info!("预热进度: {}% - 正在安装Python依赖: {}", progress, dep);

        let mut cmd = Command::new("uv");
        cmd.args(["pip", "install", dep]).kill_on_drop(true);

        match CommandExecutor::with_timeout(cmd.status(), 60).await {
            Ok(Ok(status)) => {
                if !status.success() {
                    warn!("安装依赖 {} 失败", dep);
                }
            }
            Ok(Err(e)) => {
                warn!("命令执行失败: {} - 依赖: {}", e, dep);
                continue;
            }
            Err(e) => {
                warn!("执行超时或系统错误: {} - 依赖: {}", e, dep);
                continue;
            }
        }
    }

    info!("Python环境预热完成 (100%)");
    Ok(())
}

/// 预热JavaScript/TypeScript环境，缓存常用模块
async fn warm_up_js_env(
    custom_npm_packages: Option<Vec<String>>,
    custom_jsr_packages: Option<Vec<String>>,
    custom_node_modules: Option<Vec<String>>,
) -> Result<()> {
    info!("开始预热JavaScript/TypeScript环境...");

    // 默认npm包列表
    let default_npm_packages = [
        "lodash",
        "axios",
        "moment",
        "uuid",
        "express",
        "react",
        "react-dom",
        "typescript",
        "jest",
        "webpack",
    ];

    // 默认JSR包列表
    let default_jsr_packages = [
        "@std/testing",
        "@std/http",
        "@std/path",
        "@std/fs",
        "@std/encoding/json",
    ];

    // 默认Node.js内置模块列表
    let default_node_modules = [
        "crypto", "buffer", "fs", "path", "http", "https", "url", "util", "stream", "events",
    ];

    // 使用自定义npm包或默认包
    let npm_packages = if let Some(packages) = custom_npm_packages {
        info!("使用自定义npm包列表");
        packages
    } else {
        info!("使用默认npm包列表");
        default_npm_packages
            .iter()
            .map(|&s| s.to_string())
            .collect()
    };

    // 使用自定义JSR包或默认包
    let jsr_packages = if let Some(packages) = custom_jsr_packages {
        info!("使用自定义JSR包列表");
        packages
    } else {
        info!("使用默认JSR包列表");
        default_jsr_packages
            .iter()
            .map(|&s| s.to_string())
            .collect()
    };

    // 使用自定义Node.js模块或默认模块
    let node_modules = if let Some(modules) = custom_node_modules {
        info!("使用自定义Node.js模块列表");
        modules
    } else {
        info!("使用默认Node.js模块列表");
        default_node_modules
            .iter()
            .map(|&s| s.to_string())
            .collect()
    };

    // 计算总任务数
    let total_tasks = npm_packages.len() + jsr_packages.len() + node_modules.len();
    info!("总共需要预热 {} 个JavaScript/TypeScript模块", total_tasks);

    let mut completed_tasks = 0;

    // 预热npm包
    for pkg in npm_packages.iter() {
        completed_tasks += 1;
        let progress = (completed_tasks as f32 / total_tasks as f32 * 100.0) as u32;
        info!("预热进度: {}% - 正在缓存npm包: {}", progress, pkg);

        let mut cmd = Command::new("deno");
        cmd.args(["cache", "--reload", &format!("npm:{}", pkg)]);

        match CommandExecutor::with_timeout(cmd.status(), 60).await {
            Ok(Ok(status)) => {
                if !status.success() {
                    warn!("缓存npm包 {} 失败", pkg);
                }
            }
            Ok(Err(e)) => {
                warn!("命令执行失败: {} - 包: {}", e, pkg);
                continue;
            }
            Err(e) => {
                warn!("执行超时或系统错误: {} - 包: {}", e, pkg);
                continue;
            }
        }
    }

    // 预热JSR包
    for pkg in jsr_packages.iter() {
        completed_tasks += 1;
        let progress = (completed_tasks as f32 / total_tasks as f32 * 100.0) as u32;
        info!("预热进度: {}% - 正在缓存JSR包: {}", progress, pkg);

        let mut cmd = Command::new("deno");
        cmd.args(["cache", "--reload", &format!("jsr:{}", pkg)]);

        match CommandExecutor::with_timeout(cmd.status(), 60).await {
            Ok(Ok(status)) => {
                if !status.success() {
                    warn!("缓存JSR包 {} 失败", pkg);
                }
            }
            Ok(Err(e)) => {
                warn!("命令执行失败: {} - 包: {}", e, pkg);
                continue;
            }
            Err(e) => {
                warn!("执行超时或系统错误: {} - 包: {}", e, pkg);
                continue;
            }
        }
    }

    // 预热Node.js内置模块
    for module in node_modules.iter() {
        completed_tasks += 1;
        let progress = (completed_tasks as f32 / total_tasks as f32 * 100.0) as u32;
        info!("预热进度: {}% - 正在缓存Node.js模块: {}", progress, module);

        let mut cmd = Command::new("deno");
        cmd.args(["cache", "--reload", &format!("node:{}", module)]);

        match CommandExecutor::with_timeout(cmd.status(), 60).await {
            Ok(Ok(status)) => {
                if !status.success() {
                    warn!("缓存Node.js模块 {} 失败", module);
                }
            }
            Ok(Err(e)) => {
                warn!("命令执行失败: {} - 模块: {}", e, module);
                continue;
            }
            Err(e) => {
                warn!("执行超时或系统错误: {} - 包: {}", e, module);
                continue;
            }
        }
    }

    info!("JavaScript/TypeScript环境预热完成 (100%)");
    Ok(())
}

/// 预热所有脚本执行环境
pub async fn warm_up_all_envs(
    custom_python_deps: Option<Vec<String>>,
    custom_npm_packages: Option<Vec<String>>,
    custom_jsr_packages: Option<Vec<String>>,
    custom_node_modules: Option<Vec<String>>,
) -> Result<()> {
    info!("开始预热所有脚本执行环境...");

    // 预热Python环境
    if let Err(e) = warm_up_python_env(custom_python_deps).await {
        warn!("预热Python环境失败: {}", e);
    }

    // 预热JavaScript/TypeScript环境
    if let Err(e) = warm_up_js_env(
        custom_npm_packages,
        custom_jsr_packages,
        custom_node_modules,
    )
    .await
    {
        warn!("预热JavaScript/TypeScript环境失败: {}", e);
    }

    info!("所有脚本执行环境预热完成");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_warm_up_python_env() {
        // 测试默认依赖
        warm_up_python_env(None).await.unwrap();

        // 测试自定义依赖
        let custom_deps = vec!["requests".to_string(), "pandas".to_string()];
        warm_up_python_env(Some(custom_deps)).await.unwrap();
    }

    #[tokio::test]
    async fn test_warm_up_js_env() {
        // 测试默认依赖
        warm_up_js_env(None, None, None).await.unwrap();

        // 测试自定义依赖
        let custom_npm = vec!["lodash".to_string(), "axios".to_string()];
        let custom_jsr = vec!["@std/testing".to_string()];
        let custom_node = vec!["crypto".to_string(), "buffer".to_string()];
        warm_up_js_env(Some(custom_npm), Some(custom_jsr), Some(custom_node))
            .await
            .unwrap();
    }
}
