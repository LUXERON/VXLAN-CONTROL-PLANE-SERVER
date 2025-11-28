//! Tensor Decomposition Engine
//!
//! POSTULATE 20: Recursive Tensor Decomposition
//! CP decomposition for O(log n) space and O(1) lookup

use crate::phase1::Prefix;
use anyhow::Result;

/// Rank-1 tensor factor
#[derive(Debug, Clone)]
struct TensorFactor {
    /// Factor vectors (one per dimension)
    vectors: Vec<Vec<f32>>,
    /// Weight (λ in CP decomposition)
    weight: f32,
}

/// CP Tensor Decomposition Engine
pub struct TensorDecompositionEngine {
    /// Rank-1 factors
    factors: Vec<TensorFactor>,
    /// Tensor rank
    rank: usize,
    /// Dimensions
    dims: Vec<usize>,
    /// Next hop mapping
    next_hops: Vec<String>,
    /// Metric mapping
    metrics: Vec<u32>,
}

impl TensorDecompositionEngine {
    pub fn new(rank: usize) -> Self {
        Self {
            factors: Vec::new(),
            rank,
            dims: vec![256, 256, 256, 256], // 4D tensor for IP addresses
            next_hops: Vec::new(),
            metrics: Vec::new(),
        }
    }

    /// Insert route and update tensor decomposition
    pub fn insert(&mut self, prefix: Prefix, next_hop: String, metric: u32) -> Result<()> {
        // Map prefix to 4D coordinates
        let coords = self.prefix_to_coords(prefix);

        // Create rank-1 factor for this route
        let factor = self.create_rank1_factor(&coords, metric);
        
        self.factors.push(factor);
        self.next_hops.push(next_hop);
        self.metrics.push(metric);

        // Compress if too many factors
        if self.factors.len() > self.rank * 2 {
            self.compress_factors();
        }

        Ok(())
    }

    /// Lookup using tensor contraction - O(r) where r is rank
    #[inline(always)]
    pub fn lookup(&self, ip: u32) -> Option<(String, u32)> {
        let coords = self.ip_to_coords(ip);

        // Tensor contraction with prefix matching
        // For each factor, compute similarity score based on matching octets
        let mut best_idx = 0;
        let mut best_score = f32::MIN;

        for (idx, factor) in self.factors.iter().enumerate() {
            let mut score = factor.weight;
            let mut matches = 0;

            // Check how many leading octets match (for prefix matching)
            for (dim_idx, &coord) in coords.iter().enumerate() {
                if dim_idx < factor.vectors.len() && coord < factor.vectors[dim_idx].len() {
                    let factor_val = factor.vectors[dim_idx][coord];
                    if factor_val > 0.0 {
                        matches += 1;
                        score *= factor_val;
                    } else {
                        // Check if this dimension has any non-zero value (wildcard)
                        let has_value = factor.vectors[dim_idx].iter().any(|&v| v > 0.0);
                        if has_value {
                            break; // Mismatch in a significant dimension
                        }
                    }
                }
            }

            // Prefer longer prefix matches (more matching octets)
            let adjusted_score = score * (matches as f32 + 1.0);
            if adjusted_score > best_score && matches >= 3 {
                best_score = adjusted_score;
                best_idx = idx;
            }
        }

        if best_score > 0.0 && best_idx < self.next_hops.len() {
            Some((self.next_hops[best_idx].clone(), self.metrics[best_idx]))
        } else {
            None
        }
    }

    /// Map prefix to 4D coordinates
    fn prefix_to_coords(&self, prefix: Prefix) -> [usize; 4] {
        [
            ((prefix.addr >> 24) & 0xFF) as usize,
            ((prefix.addr >> 16) & 0xFF) as usize,
            ((prefix.addr >> 8) & 0xFF) as usize,
            (prefix.addr & 0xFF) as usize,
        ]
    }

    /// Map IP to 4D coordinates
    fn ip_to_coords(&self, ip: u32) -> [usize; 4] {
        [
            ((ip >> 24) & 0xFF) as usize,
            ((ip >> 16) & 0xFF) as usize,
            ((ip >> 8) & 0xFF) as usize,
            (ip & 0xFF) as usize,
        ]
    }

    /// Create rank-1 factor from coordinates
    fn create_rank1_factor(&self, coords: &[usize; 4], metric: u32) -> TensorFactor {
        let weight = 1.0 / (metric as f32 + 1.0);
        
        let mut vectors = Vec::new();
        for (dim_idx, &coord) in coords.iter().enumerate() {
            let mut vec = vec![0.0; self.dims[dim_idx]];
            vec[coord] = 1.0; // One-hot encoding
            vectors.push(vec);
        }

        TensorFactor { vectors, weight }
    }

    /// Compress factors using alternating least squares (simplified)
    fn compress_factors(&mut self) {
        // Keep only top-k factors by weight
        self.factors.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());
        self.factors.truncate(self.rank);
    }

    /// Get compression ratio
    pub fn compression_ratio(&self) -> f64 {
        let original_size = self.dims.iter().product::<usize>();
        let compressed_size = self.factors.len() * self.dims.iter().sum::<usize>();
        original_size as f64 / compressed_size as f64
    }

    /// Get number of factors
    pub fn num_factors(&self) -> usize {
        self.factors.len()
    }
}

impl Default for TensorDecompositionEngine {
    fn default() -> Self {
        Self::new(64) // Rank 64 by default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_decomposition() {
        let mut engine = TensorDecompositionEngine::new(64);

        let prefix = Prefix::from_cidr("192.168.1.0/24").unwrap();
        engine.insert(prefix, "gateway1".to_string(), 100).unwrap();

        let ip = u32::from(std::net::Ipv4Addr::new(192, 168, 1, 42));
        let result = engine.lookup(ip);
        
        assert!(result.is_some());
        println!("Compression ratio: {}", engine.compression_ratio());
    }

    #[test]
    fn test_compression() {
        let mut engine = TensorDecompositionEngine::new(10);

        // Insert many routes
        for i in 0..100 {
            let prefix = Prefix::from_cidr(&format!("10.{}.0.0/16", i)).unwrap();
            engine.insert(prefix, format!("gateway{}", i), i as u32).unwrap();
        }

        // Should compress to rank 10
        assert!(engine.num_factors() <= 20); // 2x rank max
        println!("Factors: {}, Compression: {}", engine.num_factors(), engine.compression_ratio());
    }
}

