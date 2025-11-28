//! # API Routes
//!
//! HTTP route handlers for the UAO-QTCAM API.

use super::models::*;
use crate::unified::{TCAMEngine, Route, Prefix};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;

/// Create router with all routes
pub fn create_router(engine: Arc<TCAMEngine>) -> Router {
    let state = AppState { engine };

    Router::new()
        .route("/", get(root))
        .route("/lookup", post(lookup))
        .route("/insert", post(insert))
        .route("/delete", delete(delete_route))
        .route("/stats", get(stats))
        .route("/health", get(health))
        .with_state(state)
}

/// Root endpoint - API information
async fn root() -> Json<InfoResponse> {
    Json(InfoResponse {
        name: crate::NAME.to_string(),
        version: crate::VERSION.to_string(),
        description: "UAO-QTCAM: Unified Software-Defined TCAM".to_string(),
    })
}

/// Lookup route for IP address
async fn lookup(
    State(state): State<AppState>,
    Json(req): Json<LookupRequest>,
) -> Result<Json<LookupResponse>, AppError> {
    let result = state.engine.lookup(&req.ip).await?;

    Ok(Json(LookupResponse {
        found: result.is_some(),
        result: result.map(|r| r.into()),
    }))
}

/// Insert new route
async fn insert(
    State(state): State<AppState>,
    Json(req): Json<InsertRequest>,
) -> Result<Json<InsertResponse>, AppError> {
    let prefix = Prefix::from_cidr(&req.prefix)?;
    let route = Route::new(prefix, req.next_hop, req.metric);

    state.engine.insert(route).await?;

    Ok(Json(InsertResponse {
        success: true,
        message: format!("Route {} inserted successfully", req.prefix),
    }))
}

/// Delete route
async fn delete_route(
    State(state): State<AppState>,
    Json(req): Json<DeleteRequest>,
) -> Result<Json<DeleteResponse>, AppError> {
    let prefix = Prefix::from_cidr(&req.prefix)?;
    
    state.engine.delete(prefix).await?;

    Ok(Json(DeleteResponse {
        success: true,
        message: format!("Route {} deleted successfully", req.prefix),
    }))
}

/// Get engine statistics
async fn stats(State(state): State<AppState>) -> Json<StatsResponse> {
    let stats = state.engine.stats().await;
    Json(stats.into())
}

/// Health check
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: crate::VERSION.to_string(),
    })
}

/// Application error wrapper
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_root() {
        let engine = Arc::new(TCAMEngine::new().unwrap());
        let app = create_router(engine);

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health() {
        let engine = Arc::new(TCAMEngine::new().unwrap());
        let app = create_router(engine);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}

