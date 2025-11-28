//! # HTTP API
//!
//! RESTful HTTP API for the UAO-QTCAM unified engine.
//!
//! ## Endpoints
//!
//! - `POST /lookup` - Lookup route for IP address
//! - `POST /insert` - Insert new route
//! - `DELETE /delete` - Delete route
//! - `GET /stats` - Get engine statistics
//! - `GET /health` - Health check
//! - `GET /` - API information

pub mod routes;
pub mod models;

pub use routes::create_router;
pub use models::{ServerConfig, start_server};

