//! # Phase 2: QAGFHG (Quantum-Accelerated Galois Field Hint Generation)
//!
//! ## Target Performance
//!
//! - **Latency**: 10 ns per lookup
//! - **Throughput**: 100 Million lookups/second
//! - **Speedup**: 1,000x vs hardware TCAM
//!
//! ## Components
//!
//! - **Quantum State Management**: Superposition of routing states
//! - **Laplacian Spectral Analysis**: Graph-based prefix clustering
//! - **Dimensional Folding**: High-dimensional prefix space compression
//! - **Hardware Hints Generation**: CPU cache-optimized lookup hints

pub mod quantum_state;
pub mod spectral_analysis;
pub mod dimensional_folding;
pub mod hardware_hints;
pub mod qagfhg_engine;
pub mod qagfhg_v2_engine;

pub use quantum_state::QuantumState;
pub use spectral_analysis::SpectralAnalyzer;
pub use dimensional_folding::DimensionalFolder;
pub use hardware_hints::{HardwareHint, HintGenerator};
pub use qagfhg_engine::{QAGFHGEngine, QAGFHGLookupResult, QAGFHGStats};
pub use qagfhg_v2_engine::{QAGFHGv2Engine, QAGFHGv2Stats};

