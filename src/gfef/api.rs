//! Control Plane HTTP API
//!
//! Endpoints:
//! - POST /v1/predict - Get activation predictions
//! - GET /v1/calibration - Get current calibration matrix
//! - GET /v1/subscription/{customer_id} - Get subscription status
//! - POST /v1/extract - Trigger GFEF extraction for a model
//! - GET /v1/extract/{job_id} - Get extraction job status
//! - GET /v1/indices - List all GFEF indices
//! - WS /ws/events - WebSocket for real-time watcher events

use axum::{
    extract::{Path, State, Json, ws::WebSocketUpgrade},
    routing::{get, post},
    Router,
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::{RwLock, mpsc};
use std::collections::HashMap;

use super::prediction::{ActivationPredictor, PredictionRequest, PredictionResponse, PredictionError};
use super::calibration::{CalibrationService, CalibrationMatrix};
use super::subscription::{SubscriptionManager, SubscriptionTier, Subscription};
use super::index::{GFEFIndexGenerator, IndexConfig, IndexMetadata};
use super::storage::IndexStorage;
use super::extraction::{ExtractionService, ExtractionConfig, ExtractionResult};
use super::websocket::{WsEventBroadcaster, ws_handler};

/// Shared application state
pub struct AppState {
    pub predictor: RwLock<ActivationPredictor>,
    pub calibration: CalibrationService,
    pub subscriptions: RwLock<SubscriptionManager>,
    pub index_generator: RwLock<GFEFIndexGenerator>,
    pub storage: RwLock<IndexStorage>,
    pub extraction_service: Option<ExtractionService>,
    pub extraction_jobs: RwLock<HashMap<Uuid, ExtractionResult>>,
    pub extraction_result_rx: Option<RwLock<mpsc::Receiver<ExtractionResult>>>,
    pub ws_broadcaster: Arc<WsEventBroadcaster>,
}

impl AppState {
    pub fn new(calibration_rotation_secs: u64) -> Self {
        Self {
            predictor: RwLock::new(ActivationPredictor::new(0.95)),
            calibration: CalibrationService::new(calibration_rotation_secs),
            subscriptions: RwLock::new(SubscriptionManager::new()),
            index_generator: RwLock::new(GFEFIndexGenerator::new(IndexConfig::default())),
            storage: RwLock::new(IndexStorage::new(std::path::PathBuf::from("./indices"))),
            extraction_service: None,
            extraction_jobs: RwLock::new(HashMap::new()),
            extraction_result_rx: None,
            ws_broadcaster: Arc::new(WsEventBroadcaster::new(1000)),
        }
    }

    pub fn with_extraction(mut self, config: ExtractionConfig) -> Self {
        let (tx, rx) = mpsc::channel(100);
        self.extraction_service = Some(ExtractionService::new(config, tx));
        self.extraction_result_rx = Some(RwLock::new(rx));
        self
    }

    /// Get the WebSocket broadcaster for event forwarding
    pub fn get_broadcaster(&self) -> Arc<WsEventBroadcaster> {
        self.ws_broadcaster.clone()
    }
}

/// Create API router
pub fn create_router(state: Arc<AppState>) -> Router {
    let broadcaster = state.ws_broadcaster.clone();

    Router::new()
        .route("/v1/health", get(health_check))
        .route("/v1/predict", post(predict_activation))
        .route("/v1/calibration", get(get_calibration))
        .route("/v1/subscription/:customer_id", get(get_subscription))
        .route("/v1/subscription", post(create_subscription))
        .route("/v1/indices", get(list_indices))
        // GFEF Extraction endpoints
        .route("/v1/extract", post(trigger_extraction))
        .route("/v1/extract/:job_id", get(get_extraction_status))
        // WebSocket endpoint for real-time events
        .route("/ws/events", get(ws_events_handler))
        .with_state(state)
        // Nest WebSocket broadcaster state
        .layer(axum::Extension(broadcaster))
}

/// WebSocket events handler
async fn ws_events_handler(
    ws: WebSocketUpgrade,
    axum::Extension(broadcaster): axum::Extension<Arc<WsEventBroadcaster>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| super::websocket::handle_socket_direct(socket, broadcaster))
}

// === Handlers ===

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "NULL SPACE AI Control Plane".to_string(),
        version: "1.0.0".to_string(),
    })
}

async fn predict_activation(
    State(state): State<Arc<AppState>>,
    Json(request): Json<PredictionRequest>,
) -> Result<Json<PredictionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate subscription
    let subs = state.subscriptions.read().await;
    let subscription = subs.validate_access(&request.customer_id)
        .map_err(|e| (StatusCode::FORBIDDEN, Json(ErrorResponse { error: e.to_string() })))?;
    
    // Get calibration matrix
    let calibration = state.calibration.get_matrix();
    
    // Run prediction
    let predictor = state.predictor.read().await;
    let response = predictor.predict(&request, subscription, &calibration)
        .map_err(|e| {
            let status = match e {
                PredictionError::InvalidSession => StatusCode::UNAUTHORIZED,
                PredictionError::ModelNotFound(_) => StatusCode::NOT_FOUND,
                PredictionError::LayerNotFound(_) => StatusCode::NOT_FOUND,
                PredictionError::SubscriptionExpired => StatusCode::PAYMENT_REQUIRED,
                PredictionError::QuotaExceeded => StatusCode::TOO_MANY_REQUESTS,
                PredictionError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (status, Json(ErrorResponse { error: e.to_string() }))
        })?;
    
    Ok(Json(response))
}

async fn get_calibration(
    State(state): State<Arc<AppState>>,
) -> Json<CalibrationResponse> {
    let matrix = state.calibration.get_matrix();
    Json(CalibrationResponse {
        id: matrix.id,
        expires_at: matrix.expires_at.to_rfc3339(),
        matrix_size: matrix.values.len(),
        signature: matrix.signature,
    })
}

async fn get_subscription(
    State(state): State<Arc<AppState>>,
    Path(customer_id): Path<Uuid>,
) -> Result<Json<Subscription>, (StatusCode, Json<ErrorResponse>)> {
    let subs = state.subscriptions.read().await;
    let sub = subs.get_subscription(&customer_id)
        .ok_or((StatusCode::NOT_FOUND, Json(ErrorResponse { 
            error: "Subscription not found".to_string() 
        })))?;
    Ok(Json(sub.clone()))
}

async fn create_subscription(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateSubscriptionRequest>,
) -> Json<Subscription> {
    let mut subs = state.subscriptions.write().await;
    let sub = subs.create_subscription(request.customer_id, request.tier);
    Json(sub)
}

async fn list_indices(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<IndexMetadata>> {
    let storage = state.storage.read().await;
    Json(storage.list_metadata())
}

/// Trigger GFEF extraction for a model
async fn trigger_extraction(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ExtractRequest>,
) -> Result<Json<ExtractResponse>, (StatusCode, Json<ErrorResponse>)> {
    let extraction_service = state.extraction_service.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, Json(ErrorResponse {
            error: "Extraction service not configured".to_string()
        })))?;

    let model_path = PathBuf::from(&request.model_path);
    if !model_path.exists() {
        return Err((StatusCode::NOT_FOUND, Json(ErrorResponse {
            error: format!("Model not found: {}", request.model_path)
        })));
    }

    // Start extraction (async)
    let result = extraction_service.extract_model(&model_path, &request.customer_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: e.to_string()
        })))?;

    // Store result
    let mut jobs = state.extraction_jobs.write().await;
    jobs.insert(result.job_id, result.clone());

    Ok(Json(ExtractResponse {
        job_id: result.job_id,
        status: if result.success { "completed" } else { "failed" }.to_string(),
        model_path: request.model_path,
        index_path: result.index_path.to_string_lossy().to_string(),
        stats: result.stats,
        error: result.error_message,
    }))
}

/// Get extraction job status
async fn get_extraction_status(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<ExtractResponse>, (StatusCode, Json<ErrorResponse>)> {
    let jobs = state.extraction_jobs.read().await;
    let result = jobs.get(&job_id)
        .ok_or((StatusCode::NOT_FOUND, Json(ErrorResponse {
            error: "Extraction job not found".to_string()
        })))?;

    Ok(Json(ExtractResponse {
        job_id: result.job_id,
        status: if result.success { "completed" } else { "failed" }.to_string(),
        model_path: result.model_path.to_string_lossy().to_string(),
        index_path: result.index_path.to_string_lossy().to_string(),
        stats: result.stats.clone(),
        error: result.error_message.clone(),
    }))
}

// === Request/Response types ===

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct CalibrationResponse {
    id: Uuid,
    expires_at: String,
    matrix_size: usize,
    signature: String,
}

#[derive(Deserialize)]
struct CreateSubscriptionRequest {
    customer_id: Uuid,
    tier: SubscriptionTier,
}

#[derive(Deserialize)]
struct ExtractRequest {
    customer_id: Uuid,
    model_path: String,
}

#[derive(Serialize)]
struct ExtractResponse {
    job_id: Uuid,
    status: String,
    model_path: String,
    index_path: String,
    stats: Option<super::extraction::ExtractionStats>,
    error: Option<String>,
}
