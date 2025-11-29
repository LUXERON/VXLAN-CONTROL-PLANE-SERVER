//! Control Plane HTTP API
//!
//! Endpoints:
//! - POST /v1/predict - Get activation predictions
//! - GET /v1/calibration - Get current calibration matrix
//! - GET /v1/subscription/{customer_id} - Get subscription status
//! - POST /v1/extract - Trigger GFEF extraction for a model
//! - GET /v1/extract/{job_id} - Get extraction job status
//! - GET /v1/indices - List all GFEF indices
//! - POST /v1/index/upload - Receive GFEF index from Extractor (Triple IP Lock entry point)
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
use tracing::{info, warn, error};

use super::prediction::{ActivationPredictor, PredictionRequest, PredictionResponse, PredictionError};
use super::calibration::{CalibrationService, CalibrationMatrix};
use super::subscription::{SubscriptionManager, SubscriptionTier, Subscription};
use super::index::{GFEFIndex, GFEFIndexGenerator, IndexConfig, IndexMetadata, LayerIndex, NeuronSignature};
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

    /// Load GFEF index from file (for startup or manual loading)
    /// This is the CORE of Triple IP Lock - index stays on Control Plane FOREVER
    pub async fn load_index_from_file(&self, path: &std::path::Path) -> Result<IndexMetadata, String> {
        info!("🔐 Loading GFEF index from file: {:?}", path);

        // Read JSON metadata
        let json_path = path.with_extension("json");
        let json_content = std::fs::read_to_string(&json_path)
            .map_err(|e| format!("Failed to read index JSON: {}", e))?;

        // Parse the index metadata from our Python-generated format
        let raw: serde_json::Value = serde_json::from_str(&json_content)
            .map_err(|e| format!("Failed to parse index JSON: {}", e))?;

        // Convert to our GFEFIndex format
        let index = Self::convert_python_index_to_gfef(&raw)?;

        let metadata = IndexMetadata {
            id: index.id,
            customer_id: index.customer_id,
            model_id: index.model_id.clone(),
            model_name: index.model_name.clone(),
            generated_at: index.generated_at,
            total_neurons: index.total_neurons,
            num_layers: index.layers.len(),
            index_size_bytes: std::fs::metadata(&json_path).map(|m| m.len()).unwrap_or(0),
        };

        // Register the index with the predictor
        {
            let mut predictor = self.predictor.write().await;
            predictor.register_index(index);
        }

        info!("✅ GFEF index loaded: {} ({} neurons, {} layers)",
            metadata.model_name, metadata.total_neurons, metadata.num_layers);
        info!("🔒 Triple IP Lock ACTIVE - Index secured on Control Plane");

        Ok(metadata)
    }

    /// Convert Python-generated index format to our Rust GFEFIndex
    fn convert_python_index_to_gfef(raw: &serde_json::Value) -> Result<GFEFIndex, String> {
        let model_name = raw["model"].as_str().unwrap_or("unknown").to_string();
        let k_components = raw["k_components"].as_u64().unwrap_or(32) as u32;
        let fft_bins = raw["fft_bins"].as_u64().unwrap_or(16) as u32;
        let total_neurons = raw["total_neurons"].as_u64().unwrap_or(0);

        // Parse layers
        let layers_raw = raw["layers"].as_array()
            .ok_or("Missing layers array")?;

        let mut layers = Vec::new();
        for layer_raw in layers_raw {
            let layer_id = layer_raw["layer_id"].as_u64().unwrap_or(0) as u32;
            let layer_name = layer_raw["name"].as_str().unwrap_or("").to_string();
            let neurons = layer_raw["neurons"].as_u64().unwrap_or(0) as u32;

            // Parse pc_shape to get input_dim
            let pc_shape = layer_raw["pc_shape"].as_array();
            let input_dim = pc_shape
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;

            layers.push(LayerIndex {
                layer_id,
                layer_name,
                num_neurons: neurons,
                input_dim,
                k_components,
                principal_components: Vec::new(), // Will be loaded from binary
                signatures: Vec::new(), // Will be loaded from binary
            });
        }

        Ok(GFEFIndex {
            id: Uuid::new_v4(),
            customer_id: Uuid::nil(), // Will be set when customer uploads
            model_id: model_name.clone(),
            model_name,
            generated_at: chrono::Utc::now(),
            expires_at: None,
            layers,
            total_neurons,
            config: IndexConfig {
                k_components,
                fft_bins,
                target_sparsity: 0.95,
            },
        })
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
        .route("/v1/indices/stats", get(get_predictor_stats))
        // GFEF Index upload - Triple IP Lock entry point
        .route("/v1/index/upload", post(upload_index))
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

/// Request to upload a GFEF index (from Extractor to Control Plane)
#[derive(Deserialize)]
struct UploadIndexRequest {
    /// Customer ID
    customer_id: Uuid,
    /// Model identifier
    model_id: String,
    /// Model name
    model_name: String,
    /// Index configuration
    k_components: u32,
    fft_bins: u32,
    /// Total neurons in model
    total_neurons: u64,
    /// Layer metadata
    layers: Vec<UploadLayerData>,
}

#[derive(Deserialize)]
struct UploadLayerData {
    layer_id: u32,
    name: String,
    neurons: u32,
    input_dim: u32,
}

#[derive(Serialize)]
struct UploadIndexResponse {
    success: bool,
    index_id: Uuid,
    model_id: String,
    total_neurons: u64,
    num_layers: usize,
    message: String,
}

#[derive(Serialize)]
struct PredictorStatsResponse {
    models_loaded: usize,
    total_neurons: u64,
    total_layers: usize,
    target_sparsity: f32,
    triple_ip_lock_active: bool,
}

/// Upload GFEF index from Extractor - TRIPLE IP LOCK ENTRY POINT
/// Once the index is here, it NEVER leaves the Control Plane
async fn upload_index(
    State(state): State<Arc<AppState>>,
    Json(request): Json<UploadIndexRequest>,
) -> Result<Json<UploadIndexResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("🔐 Receiving GFEF index upload for model: {}", request.model_name);
    info!("   Customer: {}", request.customer_id);
    info!("   Neurons: {}, Layers: {}", request.total_neurons, request.layers.len());

    // Convert upload request to GFEFIndex
    let layers: Vec<LayerIndex> = request.layers.iter().map(|l| {
        LayerIndex {
            layer_id: l.layer_id,
            layer_name: l.name.clone(),
            num_neurons: l.neurons,
            input_dim: l.input_dim,
            k_components: request.k_components,
            principal_components: Vec::new(), // Would come from binary data
            signatures: Vec::new(), // Would come from binary data
        }
    }).collect();

    let index_id = Uuid::new_v4();
    let num_layers = layers.len();

    let index = GFEFIndex {
        id: index_id,
        customer_id: request.customer_id,
        model_id: request.model_id.clone(),
        model_name: request.model_name.clone(),
        generated_at: chrono::Utc::now(),
        expires_at: None,
        layers,
        total_neurons: request.total_neurons,
        config: IndexConfig {
            k_components: request.k_components,
            fft_bins: request.fft_bins,
            target_sparsity: 0.95,
        },
    };

    // Register with predictor - THIS IS WHERE TRIPLE IP LOCK ACTIVATES
    {
        let mut predictor = state.predictor.write().await;
        predictor.register_index(index);
    }

    info!("✅ GFEF index registered: {} ({} neurons)", request.model_name, request.total_neurons);
    info!("🔒 TRIPLE IP LOCK ACTIVE - Index secured on Control Plane");
    info!("   Lock 1: GFEF Index (SECURED)");
    info!("   Lock 2: Calibration Matrix (rotating every 60s)");
    info!("   Lock 3: Activation Prediction Service (real-time oracle)");

    Ok(Json(UploadIndexResponse {
        success: true,
        index_id,
        model_id: request.model_id,
        total_neurons: request.total_neurons,
        num_layers,
        message: "GFEF index secured on Control Plane. Triple IP Lock ACTIVE.".to_string(),
    }))
}

/// Get predictor statistics (how many indices loaded)
async fn get_predictor_stats(
    State(state): State<Arc<AppState>>,
) -> Json<PredictorStatsResponse> {
    let predictor = state.predictor.read().await;
    let stats = predictor.stats();

    Json(PredictorStatsResponse {
        models_loaded: stats.models_loaded,
        total_neurons: stats.total_neurons,
        total_layers: stats.total_layers,
        target_sparsity: stats.target_sparsity,
        triple_ip_lock_active: stats.models_loaded > 0,
    })
}
