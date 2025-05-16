use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

///代码运行请求,mcp调用tool工具时传入的参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCodeMessageRequest {
    //js运行参数
    pub(crate) json_param: HashMap<String, Value>,
    //运行的代码
    pub code: String,
    //前端生成的随机uid,用于查找websocket连接,发送执行过程中的log日志
    pub uid: String,
    pub engine_type: String,
}

//http返回的结构,data的结构一般是: CodeScriptExecutionResult,就是代码脚本的执行结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RunCodeHttpResult {
    //js 结果,json_value
    pub data: Value,
    // 是否执行成功,ture:默认值,执行成功
    pub success: bool,
    //如果执行错误的话,错误日志
    pub error: Option<String>,
}

/// 代码执行参数
#[derive(Debug, Serialize, Deserialize)]
pub struct CodeExecutionParams {
    /// 要执行的代码内容
    pub code: String,
    /// 代码语言
    pub language: String,
    /// 是否显示日志
    pub show_logs: bool,
}

/// 代码执行结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    /// 执行日志
    pub logs: Vec<String>,
    /// 执行结果
    pub result: Option<Value>,
    /// 执行是否成功
    pub success: bool,
    /// 错误信息
    pub error: Option<String>,
}
