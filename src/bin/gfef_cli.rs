//! GFEF Extraction CLI
//!
//! Manual command-line tool for triggering GFEF extraction.
//! Part of NULL SPACE AI Inference Network by NEUNOMY.
//!
//! Usage:
//!   gfef-cli extract --model <path> --customer <uuid>
//!   gfef-cli status --job <uuid>
//!   gfef-cli list
//!   gfef-cli watch --dir <path>

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use uuid::Uuid;

const BANNER: &str = r#"
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║   ██████╗ ███████╗███████╗███████╗     ██████╗██╗     ██╗                     ║
║  ██╔════╝ ██╔════╝██╔════╝██╔════╝    ██╔════╝██║     ██║                     ║
║  ██║  ███╗█████╗  █████╗  █████╗      ██║     ██║     ██║                     ║
║  ██║   ██║██╔══╝  ██╔══╝  ██╔══╝      ██║     ██║     ██║                     ║
║  ╚██████╔╝██║     ███████╗██║         ╚██████╗███████╗██║                     ║
║   ╚═════╝ ╚═╝     ╚══════╝╚═╝          ╚═════╝╚══════╝╚═╝                     ║
║                                                                               ║
║   Galois Field Eigenmode Folding - Extraction CLI                             ║
║   NULL SPACE AI Inference Network by NEUNOMY                                  ║
║   Part of Triple IP Lock™ Architecture (Lock 1)                               ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
"#;

#[derive(Parser)]
#[command(name = "gfef-cli")]
#[command(author = "NEUNOMY")]
#[command(version = "1.0.0")]
#[command(about = "GFEF Extraction CLI for NULL SPACE AI")]
struct Cli {
    /// Control Plane server URL
    #[arg(short, long, default_value = "http://localhost:8080")]
    server: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Trigger GFEF extraction for a model
    Extract {
        /// Path to the model directory or safetensors file
        #[arg(short, long)]
        model: PathBuf,

        /// Customer UUID (auto-generated if not provided)
        #[arg(short, long)]
        customer: Option<Uuid>,

        /// Wait for extraction to complete
        #[arg(short, long)]
        wait: bool,
    },
    /// Check extraction job status
    Status {
        /// Job UUID to check
        #[arg(short, long)]
        job: Uuid,
    },
    /// List all GFEF indices
    List {
        /// Output format (json, table)
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Watch for real-time events via WebSocket
    Watch {
        /// Filter by customer ID (optional)
        #[arg(short, long)]
        customer: Option<Uuid>,
    },
    /// Health check the Control Plane
    Health,
}

#[derive(Debug, Serialize)]
struct ExtractRequest {
    customer_id: Uuid,
    model_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExtractResponse {
    job_id: Uuid,
    status: String,
    model_path: String,
    index_path: String,
    stats: Option<ExtractionStats>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExtractionStats {
    duration_secs: f64,
    layers_processed: usize,
    neurons_indexed: usize,
    index_size_bytes: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IndexMetadata {
    model_name: String,
    customer_id: Uuid,
    created_at: String,
    num_layers: usize,
    total_neurons: usize,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
enum WsMessage {
    Connected { message: String, server_version: String, timestamp: String },
    ModelDetected { path: String, customer_id: Uuid, timestamp: String },
    ExtractionStarted { path: String, job_id: Uuid, timestamp: String },
    ExtractionCompleted { job_id: Uuid, success: bool, model_path: String, index_path: String, stats: Option<ExtractionStats>, error: Option<String>, timestamp: String },
    WatcherError { message: String, timestamp: String },
    Ping { timestamp: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", BANNER);
    
    let cli = Cli::parse();
    let client = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?;

    match cli.command {
        Commands::Extract { model, customer, wait } => {
            cmd_extract(&client, &cli.server, model, customer, wait, cli.verbose).await
        }
        Commands::Status { job } => {
            cmd_status(&client, &cli.server, job, cli.verbose).await
        }
        Commands::List { format } => {
            cmd_list(&client, &cli.server, &format, cli.verbose).await
        }
        Commands::Watch { customer } => {
            cmd_watch(&cli.server, customer, cli.verbose).await
        }
        Commands::Health => {
            cmd_health(&client, &cli.server, cli.verbose).await
        }
    }
}

/// Extract command - trigger GFEF extraction
async fn cmd_extract(
    client: &Client,
    server: &str,
    model: PathBuf,
    customer: Option<Uuid>,
    wait: bool,
    verbose: bool,
) -> Result<()> {
    let customer_id = customer.unwrap_or_else(Uuid::new_v4);
    let model_path = model.canonicalize()
        .context("Failed to resolve model path")?;

    println!("🚀 Triggering GFEF extraction...");
    println!("   Model: {}", model_path.display());
    println!("   Customer: {}", customer_id);

    let request = ExtractRequest {
        customer_id,
        model_path: model_path.to_string_lossy().to_string(),
    };

    let response = client
        .post(format!("{}/v1/extract", server))
        .json(&request)
        .send()
        .await
        .context("Failed to connect to Control Plane")?;

    if !response.status().is_success() {
        let error: serde_json::Value = response.json().await?;
        println!("❌ Extraction failed: {}", error);
        return Ok(());
    }

    let result: ExtractResponse = response.json().await?;

    println!("\n✅ Extraction triggered successfully!");
    println!("   Job ID: {}", result.job_id);
    println!("   Status: {}", result.status);

    if let Some(stats) = &result.stats {
        println!("\n📊 Extraction Stats:");
        println!("   Duration: {:.2}s", stats.duration_secs);
        println!("   Layers: {}", stats.layers_processed);
        println!("   Neurons: {}", stats.neurons_indexed);
        println!("   Index Size: {} bytes", stats.index_size_bytes);
    }

    if verbose {
        println!("\n🔧 Full Response:");
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

/// Status command - check extraction job status
async fn cmd_status(
    client: &Client,
    server: &str,
    job: Uuid,
    verbose: bool,
) -> Result<()> {
    println!("🔍 Checking job status: {}", job);

    let response = client
        .get(format!("{}/v1/extract/{}", server, job))
        .send()
        .await
        .context("Failed to connect to Control Plane")?;

    if !response.status().is_success() {
        println!("❌ Job not found: {}", job);
        return Ok(());
    }

    let result: ExtractResponse = response.json().await?;

    let status_icon = if result.status == "completed" { "✅" } else { "❌" };
    println!("\n{} Job Status: {}", status_icon, result.status);
    println!("   Job ID: {}", result.job_id);
    println!("   Model: {}", result.model_path);
    println!("   Index: {}", result.index_path);

    if let Some(error) = &result.error {
        println!("   Error: {}", error);
    }

    if let Some(stats) = &result.stats {
        println!("\n📊 Stats:");
        println!("   Duration: {:.2}s", stats.duration_secs);
        println!("   Layers: {}", stats.layers_processed);
        println!("   Neurons: {}", stats.neurons_indexed);
        println!("   Index Size: {} bytes", stats.index_size_bytes);
    }

    if verbose {
        println!("\n🔧 Full Response:");
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

/// List command - list all GFEF indices
async fn cmd_list(
    client: &Client,
    server: &str,
    format: &str,
    _verbose: bool,
) -> Result<()> {
    println!("📋 Listing GFEF indices...\n");

    let response = client
        .get(format!("{}/v1/indices", server))
        .send()
        .await
        .context("Failed to connect to Control Plane")?;

    if !response.status().is_success() {
        println!("❌ Failed to list indices");
        return Ok(());
    }

    let indices: Vec<IndexMetadata> = response.json().await?;

    if indices.is_empty() {
        println!("   No indices found.");
        return Ok(());
    }

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&indices)?);
    } else {
        println!("┌─────────────────────────┬──────────────────────────────────────┬────────┬───────────┐");
        println!("│ Model                   │ Customer ID                          │ Layers │ Neurons   │");
        println!("├─────────────────────────┼──────────────────────────────────────┼────────┼───────────┤");
        for idx in &indices {
            println!(
                "│ {:23} │ {} │ {:6} │ {:9} │",
                truncate(&idx.model_name, 23),
                idx.customer_id,
                idx.num_layers,
                idx.total_neurons
            );
        }
        println!("└─────────────────────────┴──────────────────────────────────────┴────────┴───────────┘");
        println!("\nTotal: {} indices", indices.len());
    }

    Ok(())
}

/// Watch command - connect to WebSocket for real-time events
async fn cmd_watch(
    server: &str,
    customer: Option<Uuid>,
    _verbose: bool,
) -> Result<()> {
    // Convert HTTP URL to WebSocket URL
    let ws_url = server
        .replace("http://", "ws://")
        .replace("https://", "wss://");
    let ws_url = format!("{}/ws/events", ws_url);

    println!("📡 Connecting to WebSocket: {}", ws_url);
    if let Some(cid) = customer {
        println!("   Filtering by customer: {}", cid);
    }
    println!("\n👀 Watching for events (Ctrl+C to stop)...\n");

    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .context("Failed to connect to WebSocket")?;

    let (mut write, mut read) = ws_stream.split();

    // Send subscribe message if filtering by customer
    if let Some(cid) = customer {
        let subscribe = serde_json::json!({
            "type": "Subscribe",
            "data": { "customer_id": cid }
        });
        write.send(Message::Text(subscribe.to_string())).await?;
    }

    // Listen for events
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(event) = serde_json::from_str::<WsMessage>(&text) {
                    print_ws_event(&event);
                }
            }
            Ok(Message::Close(_)) => {
                println!("\n📡 Connection closed by server");
                break;
            }
            Err(e) => {
                println!("\n❌ WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Health command - check Control Plane health
async fn cmd_health(
    client: &Client,
    server: &str,
    verbose: bool,
) -> Result<()> {
    println!("🏥 Checking Control Plane health...\n");

    let response = client
        .get(format!("{}/v1/health", server))
        .send()
        .await
        .context("Failed to connect to Control Plane")?;

    if !response.status().is_success() {
        println!("❌ Control Plane is unhealthy");
        return Ok(());
    }

    let health: HealthResponse = response.json().await?;

    println!("✅ Control Plane is healthy!");
    println!("   Service: {}", health.service);
    println!("   Version: {}", health.version);
    println!("   Status: {}", health.status);

    if verbose {
        println!("\n🔧 Full Response:");
        println!("{}", serde_json::to_string_pretty(&health)?);
    }

    Ok(())
}

/// Print WebSocket event in readable format
fn print_ws_event(event: &WsMessage) {
    match event {
        WsMessage::Connected { message, server_version, timestamp } => {
            println!("✅ [{}] Connected: {} (v{})", timestamp, message, server_version);
        }
        WsMessage::ModelDetected { path, customer_id, timestamp } => {
            println!("📦 [{}] Model Detected", timestamp);
            println!("   Path: {}", path);
            println!("   Customer: {}", customer_id);
        }
        WsMessage::ExtractionStarted { path, job_id, timestamp } => {
            println!("🚀 [{}] Extraction Started", timestamp);
            println!("   Path: {}", path);
            println!("   Job: {}", job_id);
        }
        WsMessage::ExtractionCompleted { job_id, success, model_path, index_path, stats, error, timestamp } => {
            let icon = if *success { "✅" } else { "❌" };
            println!("{} [{}] Extraction Completed", icon, timestamp);
            println!("   Job: {}", job_id);
            println!("   Model: {}", model_path);
            println!("   Index: {}", index_path);
            if let Some(s) = stats {
                println!("   Duration: {:.2}s | Layers: {} | Neurons: {}",
                    s.duration_secs, s.layers_processed, s.neurons_indexed);
            }
            if let Some(e) = error {
                println!("   Error: {}", e);
            }
        }
        WsMessage::WatcherError { message, timestamp } => {
            println!("❌ [{}] Watcher Error: {}", timestamp, message);
        }
        WsMessage::Ping { timestamp } => {
            println!("💓 [{}] Ping", timestamp);
        }
    }
    println!();
}

/// Truncate string to max length
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        format!("{:width$}", s, width = max)
    } else {
        format!("{}...", &s[..max-3])
    }
}

