//! WebSocket Support for Real-Time Watcher Event Streaming
//!
//! Provides real-time streaming of GFEF extraction events to connected clients.
//! Part of NULL SPACE AI Inference Network by NEUNOMY.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::watcher::WatcherEvent;
use super::extraction::ExtractionStats;

/// WebSocket message types for client communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// Server: New model detected
    ModelDetected {
        path: String,
        customer_id: Uuid,
        timestamp: String,
    },
    /// Server: Extraction started
    ExtractionStarted {
        path: String,
        job_id: Uuid,
        timestamp: String,
    },
    /// Server: Extraction completed
    ExtractionCompleted {
        job_id: Uuid,
        success: bool,
        model_path: String,
        index_path: String,
        stats: Option<ExtractionStatsWs>,
        error: Option<String>,
        timestamp: String,
    },
    /// Server: Watcher error
    WatcherError {
        message: String,
        timestamp: String,
    },
    /// Server: Connection established
    Connected {
        message: String,
        server_version: String,
        timestamp: String,
    },
    /// Server: Heartbeat/ping
    Ping { timestamp: String },
    /// Client: Subscribe to events
    Subscribe { customer_id: Option<Uuid> },
    /// Client: Unsubscribe
    Unsubscribe,
    /// Client: Pong response
    Pong { timestamp: String },
    /// Client: Request extraction status
    GetStatus { job_id: Uuid },
}

/// Simplified extraction stats for WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionStatsWs {
    pub duration_secs: f64,
    pub layers_processed: u32,
    pub neurons_indexed: u64,
    pub index_size_bytes: u64,
}

impl From<ExtractionStats> for ExtractionStatsWs {
    fn from(stats: ExtractionStats) -> Self {
        Self {
            duration_secs: stats.extraction_time_secs,
            layers_processed: stats.num_layers,
            neurons_indexed: stats.total_neurons,
            index_size_bytes: stats.index_size_bytes,
        }
    }
}

/// WebSocket event broadcaster
pub struct WsEventBroadcaster {
    tx: broadcast::Sender<WsMessage>,
    active_connections: RwLock<usize>,
}

impl WsEventBroadcaster {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self {
            tx,
            active_connections: RwLock::new(0),
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.tx.subscribe()
    }

    /// Broadcast an event to all connected clients
    pub fn broadcast(&self, msg: WsMessage) {
        let _ = self.tx.send(msg);
    }

    /// Get active connection count
    pub async fn connection_count(&self) -> usize {
        *self.active_connections.read().await
    }

    /// Increment connection count
    pub async fn add_connection(&self) {
        let mut count = self.active_connections.write().await;
        *count += 1;
        info!("📡 WebSocket connection added. Total: {}", *count);
    }

    /// Decrement connection count
    pub async fn remove_connection(&self) {
        let mut count = self.active_connections.write().await;
        if *count > 0 {
            *count -= 1;
        }
        info!("📡 WebSocket connection removed. Total: {}", *count);
    }

    /// Convert WatcherEvent to WsMessage and broadcast
    pub fn broadcast_watcher_event(&self, event: WatcherEvent) {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let msg = match event {
            WatcherEvent::NewModelDetected { path, customer_id } => {
                WsMessage::ModelDetected {
                    path: path.to_string_lossy().to_string(),
                    customer_id,
                    timestamp,
                }
            }
            WatcherEvent::ExtractionStarted { path, job_id } => {
                WsMessage::ExtractionStarted {
                    path: path.to_string_lossy().to_string(),
                    job_id,
                    timestamp,
                }
            }
            WatcherEvent::ExtractionCompleted { result } => {
                WsMessage::ExtractionCompleted {
                    job_id: result.job_id,
                    success: result.success,
                    model_path: result.model_path.to_string_lossy().to_string(),
                    index_path: result.index_path.to_string_lossy().to_string(),
                    stats: result.stats.map(|s| s.into()),
                    error: result.error_message,
                    timestamp,
                }
            }
            WatcherEvent::WatcherError { message } => {
                WsMessage::WatcherError { message, timestamp }
            }
        };
        self.broadcast(msg);
    }
}

/// WebSocket upgrade handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(broadcaster): State<Arc<WsEventBroadcaster>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket_direct(socket, broadcaster))
}

/// Handle individual WebSocket connection (public for direct use)
pub async fn handle_socket_direct(socket: WebSocket, broadcaster: Arc<WsEventBroadcaster>) {
    let (mut sender, mut receiver) = socket.split();

    // Increment connection count
    broadcaster.add_connection().await;

    // Send welcome message
    let welcome = WsMessage::Connected {
        message: "Connected to NULL SPACE AI Control Plane".to_string(),
        server_version: "1.0.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    if let Ok(json) = serde_json::to_string(&welcome) {
        let _ = sender.send(Message::Text(json)).await;
    }

    // Subscribe to broadcast events
    let mut rx = broadcaster.subscribe();

    // Spawn task to forward broadcast events to this client
    let sender_broadcaster = broadcaster.clone();
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Receive from broadcast channel
                result = rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if let Ok(json) = serde_json::to_string(&msg) {
                                if sender.send(Message::Text(json)).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            warn!("WebSocket client lagged, missed {} messages", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
                // Send periodic pings
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                    let ping = WsMessage::Ping {
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };
                    if let Ok(json) = serde_json::to_string(&ping) {
                        if sender.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
        sender_broadcaster.remove_connection().await;
    });

    // Handle incoming messages from client
    let recv_broadcaster = broadcaster.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        match ws_msg {
                            WsMessage::Pong { .. } => {
                                debug!("Received pong from client");
                            }
                            WsMessage::Subscribe { customer_id } => {
                                info!("Client subscribed: {:?}", customer_id);
                            }
                            WsMessage::Unsubscribe => {
                                info!("Client unsubscribed");
                            }
                            _ => {
                                debug!("Received message: {:?}", ws_msg);
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    info!("Client requested close");
                    break;
                }
                _ => {}
            }
        }
        recv_broadcaster.remove_connection().await;
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }
}

/// Spawn event forwarder from watcher channel to WebSocket broadcaster
pub fn spawn_event_forwarder(
    mut watcher_rx: tokio::sync::mpsc::Receiver<WatcherEvent>,
    broadcaster: Arc<WsEventBroadcaster>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        info!("📡 WebSocket event forwarder started");
        while let Some(event) = watcher_rx.recv().await {
            broadcaster.broadcast_watcher_event(event);
        }
        info!("📡 WebSocket event forwarder stopped");
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let msg = WsMessage::Connected {
            message: "Test".to_string(),
            server_version: "1.0.0".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Connected"));
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_extraction_completed_serialization() {
        let msg = WsMessage::ExtractionCompleted {
            job_id: Uuid::new_v4(),
            success: true,
            model_path: "/path/to/model".to_string(),
            index_path: "/path/to/index".to_string(),
            stats: Some(ExtractionStatsWs {
                duration_secs: 5.5,
                layers_processed: 32,
                neurons_indexed: 100000,
                index_size_bytes: 1024000,
            }),
            error: None,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("ExtractionCompleted"));
        assert!(json.contains("neurons_indexed"));
    }

    #[tokio::test]
    async fn test_broadcaster_connection_count() {
        let broadcaster = WsEventBroadcaster::new(100);
        assert_eq!(broadcaster.connection_count().await, 0);

        broadcaster.add_connection().await;
        assert_eq!(broadcaster.connection_count().await, 1);

        broadcaster.add_connection().await;
        assert_eq!(broadcaster.connection_count().await, 2);

        broadcaster.remove_connection().await;
        assert_eq!(broadcaster.connection_count().await, 1);
    }
}

