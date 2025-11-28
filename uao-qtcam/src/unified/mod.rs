//! # Unified TCAM Engine
//!
//! This module provides the unified orchestration layer that integrates
//! all three phases of UAO-QTCAM into a single production-ready system.
//!
//! ## Architecture
//!
//! The unified engine adaptively selects the optimal phase based on:
//! - Prefix length
//! - Traffic patterns
//! - Performance requirements
//! - Resource availability
//!
//! ## Phases
//!
//! - **Phase 1 (AHGF)**: 50 ns latency, 20M lookups/sec
//! - **Phase 2 (QAGFHG)**: 10 ns latency, 100M lookups/sec (Stage 2)
//! - **Phase 3 (SCRTT)**: 8 ns latency, 125M lookups/sec (Stage 3)
//!
//! ## Example
//!
//! ```rust,no_run
//! use uao_qtcam_unified::{TCAMEngine, Route, Prefix};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut engine = TCAMEngine::new()?;
//!     
//!     let prefix = Prefix::from_cidr("192.168.1.0/24")?;
//!     let route = Route::new(prefix, "next_hop_1", 100);
//!     engine.insert(route).await?;
//!     
//!     let result = engine.lookup("192.168.1.42").await?;
//!     println!("Result: {:?}", result);
//!     
//!     Ok(())
//! }
//! ```

pub mod tcam_engine;
pub mod performance_monitor;
pub mod control_plane;

pub use tcam_engine::{TCAMEngine, Route, LookupResult, TCAMStats, PhaseStrategy};
pub use performance_monitor::PerformanceMonitor;
pub use control_plane::{ControlPlane, ControlPlaneConfig, HealthStatus, PhaseHealth, GlobalMetrics};

// Re-export Prefix from phase1
pub use crate::phase1::Prefix;

