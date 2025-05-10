# Run Code with MCP

A Rust application that can execute JavaScript (using Deno) and Python (using uv) code with logging capabilities and result extraction.

## Features

- Execute JavaScript code using Deno runtime
- Execute Python code using uv with isolated environments for better security
- Capture and separate logs from execution results
- Support for handler functions to return execution results
- Option to show or hide logs
- Support for MCP SDK integration

## Requirements

- Rust (latest stable)
- Deno (for JavaScript execution): https://deno.land/
- uv (for isolated Python execution): https://github.com/astral-sh/uv
- Python 3 (base interpreter for uv)

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/run_code_rmcp.git
cd run_code_rmcp

# Build the application
cargo build --release

# Make sure you have the required dependencies
# Deno installation: curl -fsSL https://deno.land/x/install/install.sh | sh
# uv installation: curl -fsSL https://astral.sh/uv/install.sh | sh
```

## Usage

### Command-line Interface

```
run_code_rmcp [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -s, --show-logs    Show execution logs
    -u, --use-mcp      Use MCP SDK integration
    -h, --help         Print help

SUBCOMMANDS:
    js       Execute JavaScript code
    python   Execute Python code
    help     Print this message or the help of the given subcommand(s)
```

### Execute JavaScript Code

```bash
# From a file
cargo run -- --show-logs js -f examples/test_js.js

# Directly from command line
cargo run -- --show-logs js -c "function handler() { return 'Hello from JS'; }"
```

### Execute Python Code

```bash
# From a file
cargo run -- --show-logs python -f examples/test_python.py

# Directly from command line
cargo run -- --show-logs python -c "def handler(): return 'Hello from Python'"
```

## Code Structure

Every script should have a `handler` function that returns the final result:

### JavaScript Example

```javascript
// Some processing code
console.log("Processing...");

// Handler function that will be called to get the result
function handler() {
    // Return the final result
    return "Hello from JavaScript!";
}
```

### Python Example

```python
# Some processing code
print("Processing...")

# Handler function that will be called to get the result
def handler():
    # Return the final result
    return "Hello from Python!"
```

## License

MIT 