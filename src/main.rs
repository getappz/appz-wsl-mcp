use appz_wsl_mcp::config;
use appz_wsl_mcp::server::transport;
use clap::Parser;

#[derive(Parser)]
#[command(name = "wsl-mcp-server", about = "Intelligent WSL MCP Server")]
struct Cli {
    #[arg(long, default_value = "stdio")]
    transport: String,
    #[arg(long)]
    host: Option<String>,
    #[arg(long)]
    port: Option<u16>,
    #[arg(long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();
    let config_path = cli.config.unwrap_or_else(|| "wsl-mcp-server.yaml".into());
    let cfg = config::load(&config_path).await;

    tracing::info!("WSL MCP Server v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("23 WSL management tools available");

    match cli.transport.as_str() {
        "http" => {
            let host = cli.host.unwrap_or(cfg.server.address.clone());
            let port = cli.port.unwrap_or(cfg.server.port);
            transport::serve_http(host, port, cfg).await;
        }
        _ => {
            transport::serve_stdio(cfg).await;
        }
    }
}
