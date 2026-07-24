use crate::agents::intelligence::IntelligenceService;
use crate::agents::planner::TaskPlanner;
use crate::config::AppConfig;
use crate::read_tools;
use crate::services::exec::ExecService;
use crate::services::wsl::WslService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn};

pub async fn handle_request(state: Arc<AppConfig>, request: Value) -> Value {
    let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = request.get("id");
    let params = request.get("params");

    info!("MCP request: {method}");

    let result = match method {
        "tools/list" => tools_list(),
        "tools/call" => {
            let tool_name = params
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let args = params
                .and_then(|p| p.get("arguments"))
                .and_then(|a| a.as_object())
                .cloned()
                .unwrap_or_default();
            tools_call(state, tool_name, &args).await
        }
        "initialize" => json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "wsl-mcp-server",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
        "ping" => json!("pong"),
        _ => {
            warn!("Unknown method: {method}");
            jsonrpc_error(-32601, format!("Method not found: {method}"))
        }
    };

    let mut response = json!({
        "jsonrpc": "2.0",
        "result": result
    });
    if let Some(id) = id {
        response["id"] = id.clone();
    }
    response
}

fn tools_list() -> Value {
    json!([
        {"name": "wsl_list_distros", "description": "List all installed WSL distributions with status"},
        {"name": "wsl_get_info", "description": "Get WSL version and system information"},
        {"name": "wsl_install_distro", "description": "Install a new WSL distribution", "inputSchema": {"type": "object", "properties": {"distro": {"type": "string"}}}},
        {"name": "wsl_start_distro", "description": "Start a WSL distribution", "inputSchema": {"type": "object", "properties": {"distro": {"type": "string"}}}},
        {"name": "wsl_stop_distro", "description": "Stop a WSL distribution", "inputSchema": {"type": "object", "properties": {"distro": {"type": "string"}}}},
        {"name": "wsl_execute_command", "description": "Execute a command in a WSL distribution", "inputSchema": {"type": "object", "properties": {"distro": {"type": "string"}, "command": {"type": "string"}}}},
        {"name": "wsl_configure", "description": "Configure WSL settings (.wslconfig)", "inputSchema": {"type": "object", "properties": {"settings": {"type": "object"}}}},
        {"name": "wsl_export_distro", "description": "Export a WSL distribution to tar", "inputSchema": {"type": "object", "properties": {"distro": {"type": "string"}, "output_path": {"type": "string"}}}},
        {"name": "wsl_import_distro", "description": "Import a WSL distribution from tar", "inputSchema": {"type": "object", "properties": {"distro": {"type": "string"}, "tar_path": {"type": "string"}, "install_path": {"type": "string"}}}},
        {"name": "wsl_copy_from_wsl", "description": "Copy files from WSL to Windows", "inputSchema": {"type": "object", "properties": {"distro": {"type": "string"}, "wsl_path": {"type": "string"}, "windows_path": {"type": "string"}}}},
        {"name": "wsl_copy_to_wsl", "description": "Copy files from Windows to WSL", "inputSchema": {"type": "object", "properties": {"distro": {"type": "string"}, "windows_path": {"type": "string"}, "wsl_path": {"type": "string"}}}},
        {"name": "wsl_agent_task", "description": "Use AI agent to perform complex WSL tasks", "inputSchema": {"type": "object", "properties": {"task": {"type": "string"}, "context": {"type": "string"}}}},
        {"name": "read:get_system_info", "description": "Get WSL system information (uname -a)"},
        {"name": "read:get_os_release", "description": "Get OS distribution info from /etc/os-release"},
        {"name": "read:list_procs", "description": "List running processes", "inputSchema": {"type": "object", "properties": {"filter": {"type": "string"}}}},
        {"name": "read:get_disk_usage", "description": "Get disk usage for a path", "inputSchema": {"type": "object", "properties": {"path": {"type": "string"}}}},
        {"name": "read:get_package_managers", "description": "Detect available package managers"},
        {"name": "read:get_default_shell", "description": "Get current user's default shell"},
        {"name": "read:get_env", "description": "Get environment variables", "inputSchema": {"type": "object", "properties": {"filter": {"type": "string"}}}},
        {"name": "wsl_get_recommendations", "description": "Get AI-powered optimization recommendations", "inputSchema": {"type": "object", "properties": {"distro": {"type": "string"}}}},
        {"name": "wsl_analyze_performance", "description": "Analyze WSL performance with ML insights", "inputSchema": {"type": "object", "properties": {"distro": {"type": "string"}}}},
        {"name": "wsl_predict_resources", "description": "Predict resource requirements for a workload", "inputSchema": {"type": "object", "properties": {"distro": {"type": "string"}, "workload": {"type": "string"}}}}
    ])
}

async fn tools_call(state: Arc<AppConfig>, tool: &str, args: &serde_json::Map<String, Value>) -> Value {
    let wsl = WslService::new();
    let exec = ExecService::new(state.clone());
    let planner = TaskPlanner::new(Arc::new(WslService::new()));
    let intelligence = IntelligenceService::new(Arc::new(WslService::new()));

    match tool {
        "wsl_list_distros" => json!(wsl.list_distributions().await),
        "wsl_get_info" => json!(wsl.get_info().await),
        "wsl_install_distro" => {
            let d = str_arg(args, "distro");
            json!(wsl.install_distro(&d).await)
        }
        "wsl_start_distro" => {
            let d = str_arg(args, "distro");
            json!(wsl.start_distro(&d).await)
        }
        "wsl_stop_distro" => {
            let d = str_arg(args, "distro");
            json!(wsl.stop_distro(&d).await)
        }
        "wsl_execute_command" => {
            let d = str_arg(args, "distro");
            let c = str_arg(args, "command");
            let policy = exec.check_policy(&c);
            match policy {
                crate::services::exec::PolicyResult::Denied(reason) => {
                    jsonrpc_error(-32000, format!("Command denied by policy: {reason}"))
                }
                _ => json!(wsl.execute_command(&d, &c).await),
            }
        }
        "wsl_configure" => {
            let settings = args
                .get("settings")
                .and_then(|v| v.as_object())
                .map(|o| {
                    o.iter()
                        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                        .collect::<HashMap<_, _>>()
                })
                .unwrap_or_default();
            json!(wsl.configure(&settings).await)
        }
        "wsl_export_distro" => {
            let d = str_arg(args, "distro");
            let p = str_arg(args, "output_path");
            json!(wsl.export_distro(&d, &p).await)
        }
        "wsl_import_distro" => {
            let d = str_arg(args, "distro");
            let t = str_arg(args, "tar_path");
            let i = str_arg(args, "install_path");
            json!(wsl.import_distro(&d, &t, &i).await)
        }
        "wsl_copy_from_wsl" => {
            let d = str_arg(args, "distro");
            let w = str_arg(args, "wsl_path");
            let p = str_arg(args, "windows_path");
            json!(wsl.copy_from_wsl(&d, &w, &p).await)
        }
        "wsl_copy_to_wsl" => {
            let d = str_arg(args, "distro");
            let p = str_arg(args, "windows_path");
            let w = str_arg(args, "wsl_path");
            json!(wsl.copy_to_wsl(&d, &p, &w).await)
        }
        "wsl_agent_task" => {
            let task = str_arg(args, "task");
            let ctx = args.get("context").and_then(|v| v.as_str());
            json!(planner.execute_task(&task, ctx).await)
        }
        "read:get_system_info" => json!(read_tools::get_system_info().await),
        "read:get_os_release" => json!(read_tools::get_os_release().await),
        "read:list_procs" => {
            let filter = args.get("filter").and_then(|v| v.as_str());
            json!(read_tools::list_processes(filter).await)
        }
        "read:get_disk_usage" => {
            let path = str_arg(args, "path");
            let path = if path.is_empty() { "/" } else { &path };
            json!(read_tools::get_disk_usage(path).await)
        }
        "read:get_package_managers" => json!(read_tools::get_package_managers().await),
        "read:get_default_shell" => json!(read_tools::get_default_shell().await),
        "read:get_env" => {
            let filter = args.get("filter").and_then(|v| v.as_str());
            json!(read_tools::get_env(filter).await)
        }
        "wsl_get_recommendations" => {
            let d = args.get("distro").and_then(|v| v.as_str());
            json!(intelligence.get_recommendations(d).await)
        }
        "wsl_analyze_performance" => {
            let d = args.get("distro").and_then(|v| v.as_str());
            json!(intelligence.analyze_performance(d).await)
        }
        "wsl_predict_resources" => {
            let d = str_arg(args, "distro");
            let w = str_arg(args, "workload");
            json!(intelligence.predict_resources(&d, &w).await)
        }
        _ => jsonrpc_error(-32601, format!("Unknown tool: {tool}")),
    }
}

fn str_arg<'a>(args: &'a serde_json::Map<String, Value>, key: &str) -> String {
    args.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn jsonrpc_error(code: i32, message: String) -> Value {
    json!({"error": {"code": code, "message": message}})
}
