//! Core types for QAGML (Quantum-Accelerated GPU Memory Lookup)
//!
//! Defines fundamental data structures for GPU memory amplification.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// QAGML amplification factor: 10,000,000x
pub const AMPLIFICATION_FACTOR: u64 = 10_000_000;

/// Target GPU memory capacity: 800 PB (from 80 GB)
pub const TARGET_MEMORY_PB: u64 = 800_000;

/// Physical GPU memory: 80 GB (RTX 5090)
pub const PHYSICAL_MEMORY_GB: u64 = 80;

/// Compression ratio: 99.999%
pub const COMPRESSION_RATIO: f64 = 0.99999;

/// Input dimensions for memory tensor
pub const INPUT_DIMENSIONS: usize = 4096;

/// Output dimensions after folding
pub const OUTPUT_DIMENSIONS: usize = 16;

/// GPU memory access target: 0.00001 ns
pub const TARGET_ACCESS_TIME_NS: f64 = 0.00001;

/// GPU Memory Block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBlock {
    /// Block address
    pub address: u64,
    /// Block size in bytes
    pub size: usize,
    /// Memory data
    pub data: Vec<u8>,
    /// Metadata
    pub metadata: MemoryMetadata,
}

/// Memory Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    /// Access frequency
    pub access_frequency: u64,
    /// Last access timestamp (as u64 for serialization)
    pub last_access_ns: u64,
    /// Memory type (global, shared, local, constant)
    pub memory_type: MemoryType,
    /// Tensor features (4096D)
    pub features: Vec<f32>,
    /// Priority score
    pub priority: f32,
}

/// Memory Type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryType {
    Global,
    Shared,
    Local,
    Constant,
    Texture,
}

/// GPU Memory State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUMemoryState {
    /// Total allocated memory (bytes)
    pub allocated_bytes: u64,
    /// Total free memory (bytes)
    pub free_bytes: u64,
    /// Active memory blocks
    pub active_blocks: u32,
    /// Cache hit rate
    pub cache_hit_rate: f32,
}

/// Memory Access Pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAccessPattern {
    /// Sequential access ratio
    pub sequential_ratio: f32,
    /// Random access ratio
    pub random_ratio: f32,
    /// Stride pattern
    pub stride: usize,
    /// Temporal locality score
    pub temporal_locality: f32,
    /// Spatial locality score
    pub spatial_locality: f32,
}

/// QAGML Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QagmlConfig {
    /// Physical GPU memory in GB
    pub physical_memory_gb: u64,
    /// Target amplification factor
    pub target_amplification: u64,
    /// Enable dimensional folding
    pub enable_dimensional_folding: bool,
    /// Enable Laplacian Q-learning
    pub enable_laplacian_qlearning: bool,
    /// Enable PME engine
    pub enable_pme: bool,
    /// Enable quantum cache
    pub enable_quantum_cache: bool,
    /// Enable SIMD vectorization
    pub enable_simd: bool,
    /// Enable Galois field encryption
    pub enable_galois_field: bool,
    /// Enable spectral graph optimization
    pub enable_spectral_graph: bool,
    /// Enable tensor decomposition
    pub enable_tensor_decomposition: bool,
    /// Enable branch-free computation
    pub enable_branch_free: bool,
    /// Enable temporal coherence
    pub enable_temporal_coherence: bool,
}

impl Default for QagmlConfig {
    fn default() -> Self {
        Self {
            physical_memory_gb: 80, // RTX 5090
            target_amplification: AMPLIFICATION_FACTOR,
            enable_dimensional_folding: true,
            enable_laplacian_qlearning: true,
            enable_pme: true,
            enable_quantum_cache: true,
            enable_simd: true,
            enable_galois_field: true,
            enable_spectral_graph: true,
            enable_tensor_decomposition: true,
            enable_branch_free: true,
            enable_temporal_coherence: true,
        }
    }
}

/// GPU Memory Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUMemoryStats {
    /// Effective memory capacity (PB)
    pub effective_memory_pb: f64,
    /// Amplification factor achieved
    pub amplification_factor: f64,
    /// Average access time (ns)
    pub avg_access_time_ns: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Compression ratio
    pub compression_ratio: f64,
    /// Total memory accesses
    pub total_accesses: u64,
    /// Total cache hits
    pub cache_hits: u64,
}

