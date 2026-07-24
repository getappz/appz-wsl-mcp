# appz-wsl-mcp

Rust MCP server for intelligent WSL management. Dual-use: **library** (embed in your Rust app) or **CLI** (standalone MCP server).

```
cargo add appz-wsl-mcp --git https://github.com/getappz/appz-wsl-mcp
```

---

## Library

```rust
use appz_wsl_mcp::{WslService, ExecService, PolicyEngine, handle_request, config};

let cfg = config::load("wsl-mcp-server.yaml").await;
let wsl = WslService::new();

// Core WSL operations
let distros = wsl.list_distributions().await;
let info = wsl.get_info().await;
let result = wsl.execute_command("Ubuntu", "uname -a").await;

// Policy-gated execution
let exec = ExecService::new(cfg);
match exec.check_policy("docker build .") {
    PolicyResult::Denied(reason) => eprintln!("blocked: {reason}"),
    _ => { exec.execute("docker", &["build", "."], None, None).await; }
}

// Task planning
let planner = TaskPlanner::new(Arc::new(wsl.clone()));
let task = planner.execute_task("install Ubuntu with Docker", None).await;

// Performance intelligence
let intelligence = IntelligenceService::new(Arc::new(wsl.clone()));
let recommendations = intelligence.get_recommendations(None).await;
let prediction = intelligence.predict_resources("Ubuntu", "Docker containers").await;

// Read-only tools
use appz_wsl_mcp::read_tools;
let sysinfo = read_tools::get_system_info().await;
let procs = read_tools::list_processes(Some("docker")).await;

// Path mapping
use appz_wsl_mcp::path_map;
let wsl_path = path_map::windows_to_wsl(r"C:\Users\me\project");
let win_path = path_map::wsl_to_windows("/mnt/c/Users/me/project");

// Auth guard
use appz_wsl_mcp::auth::Authenticator;
let auth = Authenticator::new(&cfg);
if !auth.validate(Some("Bearer sk-...")) { /* reject */ }

// JSON-RPC dispatch for custom transports
let response = handle_request(state, json_request).await;
```

### Public API

| Module | Key exports |
|---|---|
| `WslService` | `list_distributions`, `get_info`, `install_distro`, `start_distro`, `stop_distro`, `execute_command`, `configure`, `export_distro`, `import_distro`, `copy_from_wsl`, `copy_to_wsl`, `shutdown` |
| `ExecService` | `execute`, `check_policy`, `apply_path_mappings` |
| `WslConfigService` | `read`, `write` — .wslconfig parser/generator |
| `TaskPlanner` | `execute_task` — decompose natural-language tasks into WSL steps |
| `IntelligenceService` | `get_recommendations`, `analyze_performance`, `predict_resources` |
| `PolicyEngine` | `evaluate` — regex-based allow/deny/confirm rules |
| `Authenticator` | `validate` — Bearer token check |
| `read_tools` | `get_system_info`, `get_os_release`, `list_processes`, `get_disk_usage`, `get_package_managers`, `get_default_shell`, `get_env`, `get_mounts`, `get_wsl_config`, `get_shells` |
| `path_map` | `apply`, `windows_to_wsl`, `wsl_to_windows` |
| `server::handler` | `handle_request` — JSON-RPC 2.0 dispatch for all 23 tools |
| `config` | `AppConfig`, `load` — YAML config with policy, auth, path mappings |

---

## CLI

### Install

```bash
cargo install --git https://github.com/getappz/appz-wsl-mcp
```

Or download from [releases](https://github.com/getappz/appz-wsl-mcp/releases).

### Run

```bash
# stdio (default — for MCP clients)
wsl-mcp-server

# HTTP (streamable HTTP transport)
wsl-mcp-server --transport http --host 127.0.0.1 --port 8787

# Custom config
wsl-mcp-server --config my-config.yaml
```

### MCP Client Setup

**Claude Desktop** (`%APPDATA%\Claude\claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "wsl": {
      "command": "wsl.exe",
      "args": ["--", "wsl-mcp-server", "stdio"]
    }
  }
}
```

**Claude Code**:
```bash
claude mcp add wsl -- wsl.exe -- wsl-mcp-server stdio
```

**Cursor** (`.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "wsl": {
      "command": "wsl.exe",
      "args": ["--", "wsl-mcp-server", "stdio"]
    }
  }
}
```

---

## Tools (23)

| Tool | Description |
|---|---|
| `wsl_list_distros` | List installed WSL distributions |
| `wsl_get_info` | WSL version and status |
| `wsl_install_distro` | Install a distribution |
| `wsl_start_distro` | Start a distribution |
| `wsl_stop_distro` | Stop a distribution |
| `wsl_execute_command` | Execute command in a distro |
| `wsl_configure` | Write .wslconfig settings |
| `wsl_export_distro` | Export distro to tar |
| `wsl_import_distro` | Import distro from tar |
| `wsl_copy_from_wsl` | Copy WSL → Windows |
| `wsl_copy_to_wsl` | Copy Windows → WSL |
| `wsl_agent_task` | AI-plan complex WSL tasks |
| `wsl_get_recommendations` | Optimization recommendations |
| `wsl_analyze_performance` | Resource analysis |
| `wsl_predict_resources` | Resource prediction by workload |
| `read:get_system_info` | `uname -a` and host info |
| `read:get_os_release` | `/etc/os-release` parser |
| `read:list_procs` | Process list with filter |
| `read:get_disk_usage` | `df -h` for a path |
| `read:get_package_managers` | Detect available package managers |
| `read:get_default_shell` | Current user's `$SHELL` |
| `read:get_env` | Environment variables with filter |
| `read:get_disk_usage` | Filesystem disk usage |

---

## Configuration

Default `wsl-mcp-server.yaml`:

```yaml
server:
  address: 127.0.0.1
  port: 8787
  transport: stdio

execution:
  default_action: allow
  default_timeout_ms: 30000
  max_timeout_ms: 300000
  commands:
    - pattern: "shutdown|reboot|poweroff|halt"
      action: deny
    - pattern: "rm -rf /|mkfs|dd if="
      action: deny
    - pattern: "sudo"
      action: confirm

auth:
  api_key_env: WSL_MCP_API_KEY

logging:
  dir: .wsl-mcp-logs
  persist: false

path_mappings:
  - from_prefix: "C:\\Users\\"
    to_prefix: "/mnt/c/Users/"
  - from_prefix: "D:\\"
    to_prefix: "/mnt/d/"
```

---

## Architecture

```
src/
├── lib.rs              # Crate root — re-exports all public API
├── main.rs             # CLI binary
├── config.rs           # YAML config loader
├── server/
│   ├── handler.rs      # MCP JSON-RPC 2.0 dispatch
│   └── transport.rs    # stdio + HTTP (axum)
├── services/
│   ├── wsl.rs          # wsl.exe process management
│   ├── exec.rs         # Policy-gated execution
│   └── config.rs       # .wslconfig read/write
├── agents/
│   ├── planner.rs      # Task decomposition
│   └── intelligence.rs # Health/predictions
├── policy/             # Regex allow/deny/confirm engine
├── path_map.rs         # WSL↔Windows path translation
├── auth.rs             # Bearer API key auth
├── logging.rs          # Execution persistence
└── read_tools.rs       # WSL read-only introspection
```

---

## License

MIT
