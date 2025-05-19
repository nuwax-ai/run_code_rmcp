// TypeScript类型声明
// @ts-nocheck
type LogFunction = (...args: any[]) => void;
type Handler = (input: any) => any;

// Save original console.log
const originalConsoleLog: LogFunction = console.log;
let logs: string[] = [];

// Replace console.log to capture logs
console.log = function(...args: any[]): void {
    // Convert arguments to string and join them
    const message = args.map(arg =>
        typeof arg === 'object' && arg !== null ? JSON.stringify(arg) : String(arg)
    ).join(' ');

    // Store log
    logs.push(message);

    // Also log to original console if showing logs
    if ({{SHOW_LOGS}}) {
        originalConsoleLog.apply(console, args);
    }
};

// 从环境变量获取输入参数
let input: any = {};
try {
    const inputJson = Deno.env.get("INPUT_JSON");
    if (inputJson) {
        input = JSON.parse(inputJson);
        console.log("接收到的参数:", JSON.stringify(input));
    }
} catch (error) {
    console.error("解析输入参数失败:", error);
}

async function executeHandler() {
    try {
        // Add the original code
        {{USER_CODE}}

        // Execute handler function and get result
        let result: any = null;
        
        // 优先检查main函数
        if (typeof main === 'function') {
            // 检查main是否是异步函数
            if (main.constructor.name === 'AsyncFunction') {
                result = await (main as (input: any) => Promise<any>)(input);
            } else {
                result = (main as Handler)(input);
            }
        } else if (typeof handler === 'function') {
            // 如果没有main函数，检查handler
            if (handler.constructor.name === 'AsyncFunction') {
                result = await (handler as (input: any) => Promise<any>)(input);
            } else {
                result = (handler as Handler)(input);
            }
        } else {
            throw new Error("没有找到main或handler函数");
        }

        // Print final output as JSON
        originalConsoleLog(JSON.stringify({
            logs: logs,
            result: result !== undefined ? (typeof result === 'object' ? JSON.stringify(result) : String(result)) : null,
            error: null
        }));
    } catch (error) {
        // Handle errors
        originalConsoleLog(JSON.stringify({
            logs: logs,
            result: null,
            error: String(error)
        }));
    }
}

// 执行并等待结果
executeHandler(); 