//! # UAO-QTCAM Unified Server
//!
//! Production-ready HTTP server for the UAO-QTCAM routing engine.

use uao_qtcam_unified::{TCAMEngine, start_server, ServerConfig};
use anyhow::Result;
use std::sync::Arc;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    // Print banner
    print_banner();

    // Create TCAM engine
    tracing::info!("🔧 Initializing UAO-QTCAM engine...");
    let engine = Arc::new(TCAMEngine::new()?);
    tracing::info!("✅ Engine initialized successfully");

    // Load sample routes for testing
    load_sample_routes(&engine).await?;

    // Server configuration
    let config = ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
    };

    // Start server
    tracing::info!("🌐 Starting HTTP server on {}:{}", config.host, config.port);
    start_server(config, engine).await?;

    Ok(())
}

/// Print ASCII banner
fn print_banner() {
    println!(r#"
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║   ██╗   ██╗ █████╗  ██████╗        ██████╗ ████████╗ ██████╗ █████╗ ███╗   ███╗
║   ██║   ██║██╔══██╗██╔═══██╗      ██╔═══██╗╚══██╔══╝██╔════╝██╔══██╗████╗ ████║
║   ██║   ██║███████║██║   ██║█████╗██║   ██║   ██║   ██║     ███████║██╔████╔██║
║   ██║   ██║██╔══██║██║   ██║╚════╝██║▄▄ ██║   ██║   ██║     ██╔══██║██║╚██╔╝██║
║   ╚██████╔╝██║  ██║╚██████╔╝      ╚██████╔╝   ██║   ╚██████╗██║  ██║██║ ╚═╝ ██║
║    ╚═════╝ ╚═╝  ╚═╝ ╚═════╝        ╚══▀▀═╝    ╚═╝    ╚═════╝╚═╝  ╚═╝╚═╝     ╚═╝
║                                                              ║
║              Unified Software-Defined TCAM v1.0.0           ║
║                                                              ║
║  🎯 Phase 1 (AHGF): 50 ns latency, 20M lookups/sec         ║
║  🌌 Revolutionary routing intelligence                      ║
║  🔬 Production-ready performance                            ║
║                                                              ║
║  Author: LUXERON                                            ║
║  GitHub: https://github.com/LUXERON/UAO-QTCAM              ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
"#);
}

/// Load sample routes for testing
async fn load_sample_routes(engine: &Arc<TCAMEngine>) -> Result<()> {
    use uao_qtcam_unified::{Route, Prefix};

    tracing::info!("📋 Loading sample routes...");

    let sample_routes = vec![
        ("10.0.0.0/8", "gateway_1", 100),
        ("172.16.0.0/12", "gateway_2", 50),
        ("192.168.0.0/16", "gateway_3", 10),
        ("192.168.1.0/24", "gateway_4", 5),
        ("0.0.0.0/0", "default_gateway", 1000),
    ];

    for (cidr, next_hop, metric) in sample_routes {
        let prefix = Prefix::from_cidr(cidr)?;
        let route = Route::new(prefix, next_hop, metric);
        engine.insert(route).await?;
        tracing::info!("  ✓ Loaded route: {} -> {}", cidr, next_hop);
    }

    tracing::info!("✅ Loaded {} sample routes", engine.route_count());

    Ok(())
}

