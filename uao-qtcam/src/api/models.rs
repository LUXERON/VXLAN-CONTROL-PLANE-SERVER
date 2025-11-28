//! # API Models
//!
//! Request and response models for the HTTP API.

use serde::{Deserialize, Serialize};
use crate::unified::{TCAMEngine, TCAMStats, LookupResult};
use std::sync::Arc;
use anyhow::Result;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<TCAMEngine>,
}

/// Lookup request
#[derive(Debug, Deserialize)]
pub struct LookupRequest {
    pub ip: String,
}

/// Lookup response
#[derive(Debug, Serialize)]
pub struct LookupResponse {
    pub found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<LookupResultDto>,
}

/// Lookup result DTO
#[derive(Debug, Serialize)]
pub struct LookupResultDto {
    pub prefix: String,
    pub next_hop: String,
    pub metric: u32,
    pub latency_ns: f64,
    pub phase: String,
}

impl From<LookupResult> for LookupResultDto {
    fn from(result: LookupResult) -> Self {
        Self {
            prefix: result.prefix,
            next_hop: result.next_hop,
            metric: result.metric,
            latency_ns: result.latency_ns,
            phase: result.phase,
        }
    }
}

/// Insert request
#[derive(Debug, Deserialize)]
pub struct InsertRequest {
    pub prefix: String,
    pub next_hop: String,
    pub metric: u32,
}

/// Insert response
#[derive(Debug, Serialize)]
pub struct InsertResponse {
    pub success: bool,
    pub message: String,
}

/// Delete request
#[derive(Debug, Deserialize)]
pub struct DeleteRequest {
    pub prefix: String,
}

/// Delete response
#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub message: String,
}

/// Stats response
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_lookups: u64,
    pub total_inserts: u64,
    pub total_deletes: u64,
    pub route_count: usize,
    pub avg_lookup_ns: f64,
    pub phase1_lookups: u64,
    pub phase2_lookups: u64,
    pub phase3_lookups: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
}

impl From<TCAMStats> for StatsResponse {
    fn from(stats: TCAMStats) -> Self {
        let cache_hit_rate = if stats.total_lookups > 0 {
            stats.cache_hits as f64 / stats.total_lookups as f64
        } else {
            0.0
        };

        Self {
            total_lookups: stats.total_lookups,
            total_inserts: stats.total_inserts,
            total_deletes: stats.total_deletes,
            route_count: stats.route_count,
            avg_lookup_ns: stats.avg_lookup_ns,
            phase1_lookups: stats.phase1_lookups,
            phase2_lookups: stats.phase2_lookups,
            phase3_lookups: stats.phase3_lookups,
            cache_hits: stats.cache_hits,
            cache_misses: stats.cache_misses,
            cache_hit_rate,
        }
    }
}

/// Health response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Info response
#[derive(Debug, Serialize)]
pub struct InfoResponse {
    pub name: String,
    pub version: String,
    pub description: String,
}

/// Start HTTP server
pub async fn start_server(config: ServerConfig, engine: Arc<TCAMEngine>) -> Result<()> {
    use crate::api::routes::create_router;

    let app = create_router(engine);
    let addr = format!("{}:{}", config.host, config.port);
    
    tracing::info!("🚀 Starting UAO-QTCAM server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

