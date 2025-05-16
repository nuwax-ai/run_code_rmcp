// @ts-nocheck
// 普通脚本格式

// 保存原始console.log
const originalConsoleLog = console.log;
let logs = [];

// 替换console.log以捕获日志
console.log = function() {
    // 将参数转换为字符串并连接它们
    const message = Array.from(arguments).map(arg =>
        typeof arg === 'object' && arg !== null ? JSON.stringify(arg) : String(arg)
    ).join(' ');

    // 存储日志
    logs.push(message);

    // 如果显示日志，也输出到原始控制台
    if ({{SHOW_LOGS}}) {
        originalConsoleLog.apply(console, arguments);
    }
};

// 从环境变量获取输入参数
let input = {};
try {
    const inputJson = Deno.env.get("INPUT_JSON");
    if (inputJson) {
        input = JSON.parse(inputJson);
        console.log("接收到的参数:", JSON.stringify(input));
    }
} catch (error) {
    console.error("解析输入参数失败:", error);
}

// 异步立即执行函数
(async () => {
    try {
        // 用户代码开始
        {{USER_CODE}}
        // 用户代码结束

        // 执行handler函数并获取结果
        let result = null;
        if (typeof handler === 'function') {
            // 检查handler是否是异步函数
            if (handler.constructor.name === 'AsyncFunction') {
                result = await handler(input);
            } else {
                result = handler(input);
            }
        }

        // 打印最终输出为JSON
        originalConsoleLog(JSON.stringify({
            logs: logs,
            result: result !== undefined ? (typeof result === 'object' ? JSON.stringify(result) : String(result)) : null,
            error: null
        }));
    } catch (error) {
        // 处理错误
        originalConsoleLog(JSON.stringify({
            logs: logs,
            result: null,
            error: error.toString()
        }));
    }
})(); 