//! Quantum State Management for QAGFHG
//!
//! This module implements quantum state representation for routing prefixes using
//! superposition and entanglement concepts. Each routing prefix is represented as
//! a quantum state with amplitude vectors and phase information.

use nalgebra::{DVector, Complex};
use std::f64::consts::PI;
use anyhow::{Result, anyhow};

/// Quantum state representing a routing prefix in superposition
#[derive(Debug, Clone)]
pub struct QuantumState {
    /// Amplitude vector (complex coefficients)
    amplitudes: DVector<Complex<f64>>,
    /// Phase information (in radians)
    phase: f64,
    /// Normalization factor
    norm: f64,
}

impl QuantumState {
    /// Create a new quantum state from a routing prefix
    ///
    /// # Arguments
    /// * `prefix_addr` - IPv4 prefix address (32-bit)
    /// * `prefix_len` - Prefix length (0-32)
    ///
    /// # Returns
    /// A normalized quantum state in superposition
    pub fn from_prefix(prefix_addr: u32, prefix_len: u8) -> Result<Self> {
        if prefix_len > 32 {
            return Err(anyhow!("Invalid prefix length: {}", prefix_len));
        }

        // Dimension: 2^(prefix_len) for superposition space
        let dim = if prefix_len == 0 { 1 } else { 1 << prefix_len.min(16) }; // Cap at 2^16 for memory
        
        // Initialize amplitudes with Hadamard-like superposition
        let mut amplitudes = DVector::from_element(dim, Complex::new(0.0, 0.0));
        
        // Encode prefix bits into quantum amplitudes
        for i in 0..dim {
            let bit_pattern = (prefix_addr >> (32 - prefix_len)) ^ (i as u32);
            let amplitude = 1.0 / (dim as f64).sqrt();
            let phase_shift = 2.0 * PI * (bit_pattern as f64) / (dim as f64);
            
            amplitudes[i] = Complex::new(
                amplitude * phase_shift.cos(),
                amplitude * phase_shift.sin(),
            );
        }

        // Calculate phase from prefix address
        let phase = 2.0 * PI * (prefix_addr as f64) / (u32::MAX as f64);

        // Normalize
        let norm = amplitudes.iter().map(|a| a.norm_sqr()).sum::<f64>().sqrt();

        Ok(Self {
            amplitudes,
            phase,
            norm,
        })
    }

    /// Collapse quantum state to classical routing decision
    ///
    /// This simulates quantum measurement, collapsing superposition to a single state
    pub fn collapse(&self) -> u32 {
        // Find amplitude with maximum probability
        let max_idx = self.amplitudes
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.norm_sqr().partial_cmp(&b.norm_sqr()).unwrap()
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        max_idx as u32
    }

    /// Calculate entanglement entropy (measure of superposition)
    pub fn entanglement_entropy(&self) -> f64 {
        let mut entropy = 0.0;
        for amp in self.amplitudes.iter() {
            let prob = amp.norm_sqr() / (self.norm * self.norm);
            if prob > 1e-10 {
                entropy -= prob * prob.ln();
            }
        }
        entropy
    }

    /// Apply quantum gate transformation (rotation)
    pub fn apply_rotation(&mut self, angle: f64) {
        let rotation = Complex::new(angle.cos(), angle.sin());
        for amp in self.amplitudes.iter_mut() {
            *amp *= rotation;
        }
        self.phase = (self.phase + angle) % (2.0 * PI);
    }

    /// Get dimension of quantum state
    pub fn dimension(&self) -> usize {
        self.amplitudes.len()
    }

    /// Get phase
    pub fn phase(&self) -> f64 {
        self.phase
    }

    /// Get normalization factor
    pub fn norm(&self) -> f64 {
        self.norm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_state_creation() {
        let state = QuantumState::from_prefix(0xC0A80100, 24).unwrap(); // 192.168.1.0/24
        assert!(state.dimension() > 0);
        assert!(state.norm() > 0.0);
        assert!(state.phase() >= 0.0 && state.phase() < 2.0 * PI);
    }

    #[test]
    fn test_quantum_state_collapse() {
        let state = QuantumState::from_prefix(0xC0A80100, 24).unwrap();
        let collapsed = state.collapse();
        assert!(collapsed < state.dimension() as u32);
    }

    #[test]
    fn test_entanglement_entropy() {
        let state = QuantumState::from_prefix(0xC0A80100, 24).unwrap();
        let entropy = state.entanglement_entropy();
        assert!(entropy >= 0.0);
    }

    #[test]
    fn test_quantum_rotation() {
        let mut state = QuantumState::from_prefix(0xC0A80100, 24).unwrap();
        let initial_phase = state.phase();
        state.apply_rotation(PI / 4.0);
        assert!((state.phase() - (initial_phase + PI / 4.0)).abs() < 1e-10);
    }
}

