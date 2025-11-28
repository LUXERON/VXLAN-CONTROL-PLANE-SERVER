//! Core types for QANBAN
//!
//! Production-ready packet and configuration types for the
//! Quantum-Accelerated Network Bandwidth Amplification system.

use serde::{Serialize, Deserialize};

/// Network packet with metadata for QANBAN processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    /// Source IP address
    pub src_ip: String,
    /// Destination IP address
    pub dst_ip: String,
    /// Source port
    pub src_port: u16,
    /// Destination port
    pub dst_port: u16,
    /// Protocol (TCP=6, UDP=17, etc.)
    pub protocol: u8,
    /// Packet payload
    pub payload: Vec<u8>,
    /// Raw packet data for feature extraction
    pub data: Vec<u8>,
    /// Packet metadata
    pub metadata: PacketMetadata,
}

impl Packet {
    /// Create a new packet with default metadata
    pub fn new(src_ip: &str, dst_ip: &str, data: Vec<u8>) -> Self {
        Self {
            src_ip: src_ip.to_string(),
            dst_ip: dst_ip.to_string(),
            src_port: 0,
            dst_port: 0,
            protocol: 6, // TCP default
            payload: data.clone(),
            data,
            metadata: PacketMetadata::default(),
        }
    }

    /// Create packet with full configuration
    pub fn with_metadata(
        src_ip: &str,
        dst_ip: &str,
        src_port: u16,
        dst_port: u16,
        protocol: u8,
        data: Vec<u8>,
        metadata: PacketMetadata,
    ) -> Self {
        Self {
            src_ip: src_ip.to_string(),
            dst_ip: dst_ip.to_string(),
            src_port,
            dst_port,
            protocol,
            payload: data.clone(),
            data,
            metadata,
        }
    }
}

/// Packet metadata for routing and processing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketMetadata {
    /// Pre-extracted features (optional)
    pub features: Vec<f32>,
    /// Flow identifier
    pub flow_id: u32,
    /// Packet priority (0-255)
    pub priority: u8,
    /// Time-to-live
    pub ttl: u8,
    /// Timestamp in nanoseconds
    pub timestamp: u64,
    /// Sequence number within flow
    pub sequence: u64,
}

impl Default for PacketMetadata {
    fn default() -> Self {
        Self {
            features: Vec::new(),
            flow_id: 0,
            priority: 128, // Medium priority
            ttl: 64,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            sequence: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkFlow {
    pub flow_id: u64,
    pub packet_count: u64,
    pub byte_count: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthStats {
    pub physical_bandwidth_gbps: f64,
    pub effective_bandwidth_pbps: f64,
    pub amplification_factor: f64,
    pub compression_ratio: f64,
    pub packet_loss_rate: f64,
    pub avg_latency_ns: f64,
    pub throughput_pps: u64,
    pub packets_processed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QanbanConfig {
    pub physical_bandwidth_gbps: u64,
    pub target_amplification: u64,
    pub enable_dimensional_folding: bool,
    pub enable_laplacian_qlearning: bool,
    pub enable_pme: bool,
    pub enable_quantum_cache: bool,
    pub enable_simd: bool,
}

impl Default for QanbanConfig {
    fn default() -> Self {
        Self {
            physical_bandwidth_gbps: 100,
            target_amplification: 1_000_000,
            enable_dimensional_folding: true,
            enable_laplacian_qlearning: true,
            enable_pme: true,
            enable_quantum_cache: true,
            enable_simd: true,
        }
    }
}
