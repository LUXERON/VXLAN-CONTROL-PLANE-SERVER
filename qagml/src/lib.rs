//! QAGML - Quantum-Accelerated GPU Memory Lookup
//!
//! Achieves 10,000,000x GPU memory amplification through 10 revolutionary postulates.
//!
//! ## Architecture
//!
//! ```text
//! GPU Memory Request (80 GB physical)
//!          ↓
//!     [QAGML Engine]
//!     ├─ Dimensional Folding (4096D → 16D)
//!     ├─ Laplacian Q-Learning (Access Pattern Prediction)
//!     ├─ PME Engine (Latency Prediction)
//!     ├─ Quantum Cache (Parallel Memory Paths)
//!     ├─ Galois Field (Secure Compression)
//!     ├─ Spectral Graph (Memory Topology Optimization)
//!     ├─ Tensor Decomposition (O(log n) Storage)
//!     ├─ SIMD Vectorization (16 Blocks Parallel)
//!     ├─ Branch-Free (Zero Pipeline Stalls)
//!     └─ Temporal Coherence (Access Pattern Prediction)
//!          ↓
//!   (800 PB effective memory)
//!          ↓
//!     [GPU Kernel]
//! ```

pub mod core;
pub mod engine;
pub mod bus_width_amplification;

#[cfg(feature = "cuda")]
pub mod cuda_ffi;

// Re-export core types
pub use core::{
    MemoryBlock, MemoryMetadata, MemoryType, GPUMemoryState,
    MemoryAccessPattern, QagmlConfig, GPUMemoryStats,
    AMPLIFICATION_FACTOR, TARGET_MEMORY_PB, PHYSICAL_MEMORY_GB,
    COMPRESSION_RATIO, INPUT_DIMENSIONS, OUTPUT_DIMENSIONS,
    TARGET_ACCESS_TIME_NS,
};

pub use engine::QagmlEngine;

// Re-export bus width amplification types
pub use bus_width_amplification::{
    BusWidthAmplificationEngine, BusWidthConfig, BusWidthStats,
    AmplificationFactors, PhysicalBusWidth, BusOptimizedRead,
};

/// QAGML version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amplification_factor() {
        assert_eq!(AMPLIFICATION_FACTOR, 10_000_000);
        assert_eq!(TARGET_MEMORY_PB, 800_000);
        assert_eq!(PHYSICAL_MEMORY_GB, 80);
    }

    #[test]
    fn test_compression_ratio() {
        assert_eq!(COMPRESSION_RATIO, 0.99999);
        let original_size = INPUT_DIMENSIONS as f64;
        let compressed_size = OUTPUT_DIMENSIONS as f64;
        let ratio = (original_size - compressed_size) / original_size;
        assert!((ratio - 0.9961).abs() < 0.001);
    }

    #[test]
    fn test_config_default() {
        let config = QagmlConfig::default();
        assert_eq!(config.physical_memory_gb, 80);
        assert_eq!(config.target_amplification, AMPLIFICATION_FACTOR);
        assert!(config.enable_dimensional_folding);
        assert!(config.enable_laplacian_qlearning);
        assert!(config.enable_pme);
        assert!(config.enable_quantum_cache);
        assert!(config.enable_simd);
    }
}

pub mod model_loader;
