# MCP 代码执行工具

这是一个支持通过MCP（Model Context Protocol）协议执行JavaScript、TypeScript和Python代码的工具。

## 功能特点

- 执行JavaScript代码（使用Deno运行时）
- 执行TypeScript代码（使用Deno运行时）
- 执行Python代码（使用uv提供隔离环境）
- 捕获并区分脚本中的日志输出和执行结果
- 支持JavaScript/TypeScript的handler函数和Python的handler/main函数作为执行结果的返回点
- 通过环境变量传递参数给脚本
- 可以通过参数控制是否显示日志输出
- 支持MCP SDK集成
- 提供命令行工具和Rust库两种使用方式

## 安装方法

### 从源码安装

```bash
# 克隆仓库
git clone https://github.com/yourusername/run_code_rmcp.git
cd run_code_rmcp

# 安装工具
cargo install --path . --bin script_runner
```

安装完成后，`script_runner` 命令将可用于系统中。

## 使用方法

### 直接使用cargo run执行代码

```bash
# 执行JavaScript文件
cargo run -- --show-logs js -f fixtures/test_js.js

# 执行带参数的JavaScript文件
cargo run -- --show-logs js -f fixtures/test_js_params.js -p '{"name":"User"}'

# 执行TypeScript文件
cargo run -- --show-logs ts -f fixtures/test_ts.ts

# 执行带参数的TypeScript文件
cargo run -- --show-logs ts -f fixtures/test_ts_params.ts -p '{"a":10, "b":20, "name":"User"}'

# 执行Python文件
cargo run -- --show-logs python -f fixtures/test_python.py

# 带参数执行Python文件
cargo run -- --show-logs python -f fixtures/test_python_params.py -p '{"a":10, "b":20}'

# 执行不同类型的Python示例
cargo run -- --show-logs python -f fixtures/test_python_types.py -p '{"type":"string"}'
cargo run -- --show-logs python -f fixtures/test_python_types.py -p '{"type":"number"}'
cargo run -- --show-logs python -f fixtures/test_python_types.py -p '{"type":"list"}'
cargo run -- --show-logs python -f fixtures/test_python_types.py -p '{"type":"dict"}'

# 直接执行JavaScript代码
cargo run -- js -c "function handler(input) { return 'Hello from JS: ' + input.name; }" -p '{"name":"User"}'

# 直接执行Python代码
cargo run -- python -c "def handler(args): return 'Hello from Python: ' + args.get('name', 'Guest')" -p '{"name":"User"}'
```

### 作为命令行工具使用

`script_runner` 是一个基于标准输入/输出的MCP服务器，可以通过以下方式启动：

```bash
# 启动MCP服务器
script_runner

# 启用详细日志输出
script_runner --verbose
```

启动后，`script_runner` 将监听标准输入，等待MCP协议格式的JSON请求，并通过标准输出返回响应。

### 与MCP客户端交互

`script_runner` 是一个MCP服务器，可以与任何支持MCP协议的客户端交互。例如，可以使用官方的MCP Inspector工具进行交互：

```bash
# 安装MCP Inspector
npm install -g @modelcontextprotocol/inspector

# 使用MCP Inspector连接到script_runner
npx @modelcontextprotocol/inspector script_runner
```

### 使用示例脚本

项目提供了两个示例脚本，用于测试与`script_runner`的交互：

1. Node.js客户端示例 (`examples/mcp_client.js`)：
```bash
# 运行Node.js客户端示例
node examples/mcp_client.js
```

2. Bash脚本示例 (`examples/test_mcp.sh`)：
```bash
# 运行Bash脚本示例
bash examples/test_mcp.sh
```

### 作为Rust库引用

本项目不仅提供命令行工具，还可以作为库被其他Rust项目引用。在你的Rust项目中添加依赖：

```toml
# Cargo.toml
[dependencies]
run_code_rmcp = { git = "https://github.com/yourusername/run_code_rmcp.git" }
```

然后在你的Rust代码中使用：

```rust
use run_code_rmcp::execute_javascript;
use run_code_rmcp::execute_typescript;
use run_code_rmcp::execute_python;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 执行JavaScript代码
    let js_result = execute_javascript("console.log('Hello'); return 42;")?;
    println!("JavaScript执行结果: {}", js_result);
    
    // 执行TypeScript代码
    let ts_result = execute_typescript("const x: number = 10; return x * 2;")?;
    println!("TypeScript执行结果: {}", ts_result);
    
    // 执行Python代码
    let py_result = execute_python("import math; result = math.sqrt(16); result")?;
    println!("Python执行结果: {}", py_result);
    
    Ok(())
}
```

## 可用工具

通过MCP协议，可以使用以下工具：

1. `run_javascript` - 执行JavaScript代码
   - 参数：
     - `code`: 要执行的JavaScript代码
     - `params`: 可选的执行参数

2. `run_typescript` - 执行TypeScript代码
   - 参数：
     - `code`: 要执行的TypeScript代码
     - `params`: 可选的执行参数

3. `run_python` - 执行Python代码
   - 参数：
     - `code`: 要执行的Python代码
     - `params`: 可选的执行参数

## 示例

### 执行JavaScript代码

```json
{
  "name": "run_javascript",
  "arguments": {
    "code": "console.log('Hello, World!'); return 42;"
  }
}
```

### 执行TypeScript代码

```json
{
  "name": "run_typescript",
  "arguments": {
    "code": "const greeting: string = 'Hello, TypeScript!'; console.log(greeting); return { message: greeting };"
  }
}
```

### 执行Python代码

```json
{
  "name": "run_python",
  "arguments": {
    "code": "print('Hello from Python!'); import math; result = math.sqrt(16); print(f'Square root of 16 is {result}'); result"
  }
}
```

## 代码结构

每个脚本都应该有一个`handler`函数（对于JS/TS）或`handler`/`main`函数（对于Python）来返回最终结果：

### JavaScript示例

```javascript
// 一些处理代码
console.log("Processing...");

// 处理函数，将被调用以获取结果
function handler(input) {
    // 输入包含通过-p/--params传递的参数
    console.log("Received input:", input);
    
    // 返回最终结果
    return "Hello from JavaScript! Input: " + JSON.stringify(input);
}
```

### TypeScript示例

```typescript
// 一些处理代码
console.log("Processing...");

// 处理函数，将被调用以获取结果
function handler(input: any): string {
    // 输入包含通过-p/--params传递的参数
    console.log("Received input:", input);
    
    // 返回最终结果
    return `Hello from TypeScript! Input: ${JSON.stringify(input)}`;
}
```

### Python示例

```python
# 一些处理代码
print("Processing...")

# 可以使用handler函数（优先）或main函数
def handler(args):
    # args包含通过-p/--params传递的参数
    print(f"Received args: {args}")
    
    # 返回最终结果
    return f"Hello from Python! Args: {args}"

# 或者使用main函数
def main(args):
    # args包含通过-p/--params传递的参数
    print(f"Received args: {args}")
    
    # 返回最终结果
    return f"Hello from Python! Args: {args}"
```

## 系统要求

- Rust 1.70.0 或更高版本
- 对于JavaScript/TypeScript执行：Deno
- 对于Python执行：Python 3.8+

## 许可证

MIT 