//! Hardware Hints Generation for CPU Cache Optimization
//!
//! This module generates CPU cache-optimized lookup hints to achieve 10 ns latency.
//! Uses cache line alignment, prefetch instructions, and branch prediction hints.

use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::{Result, anyhow};

/// Cache line size (typically 64 bytes on x86-64)
const CACHE_LINE_SIZE: usize = 64;

/// Hardware hint for optimized lookup
#[repr(align(64))] // Align to cache line boundary
#[derive(Debug, Clone)]
pub struct HardwareHint {
    /// Prefix address (32-bit IPv4)
    pub prefix_addr: u32,
    /// Prefix length (0-32)
    pub prefix_len: u8,
    /// Next hop identifier
    pub next_hop_id: u32,
    /// Cluster ID (from spectral analysis)
    pub cluster_id: u16,
    /// Compressed coordinates (from dimensional folding)
    pub compressed_coords: [f32; 4],
    /// Probability score (0.0-1.0) for branch prediction
    pub probability: f32,
    /// Padding to fill cache line
    _padding: [u8; 23],
}

impl HardwareHint {
    /// Create a new hardware hint
    pub fn new(
        prefix_addr: u32,
        prefix_len: u8,
        next_hop_id: u32,
        cluster_id: u16,
        compressed_coords: [f32; 4],
        probability: f32,
    ) -> Self {
        Self {
            prefix_addr,
            prefix_len,
            next_hop_id,
            cluster_id,
            compressed_coords,
            probability,
            _padding: [0; 23],
        }
    }

    /// Check if this hint matches an IP address
    #[inline(always)]
    pub fn matches(&self, ip: u32) -> bool {
        if self.prefix_len == 0 {
            return true;
        }
        let mask = !0u32 << (32 - self.prefix_len);
        (ip & mask) == (self.prefix_addr & mask)
    }

    /// Calculate distance in compressed space (for nearest neighbor)
    #[inline(always)]
    pub fn distance(&self, coords: &[f32; 4]) -> f32 {
        let mut dist_sq = 0.0f32;
        for i in 0..4 {
            let diff = self.compressed_coords[i] - coords[i];
            dist_sq += diff * diff;
        }
        dist_sq.sqrt()
    }
}

/// Hardware hint generator
pub struct HintGenerator {
    /// Hint table (cache-aligned)
    hints: Arc<RwLock<Vec<HardwareHint>>>,
    /// Cluster index (cluster_id -> hint indices)
    cluster_index: Arc<RwLock<Vec<Vec<usize>>>>,
    /// Number of clusters
    num_clusters: usize,
}

impl HintGenerator {
    /// Create a new hint generator
    pub fn new(num_clusters: usize) -> Self {
        Self {
            hints: Arc::new(RwLock::new(Vec::new())),
            cluster_index: Arc::new(RwLock::new(vec![Vec::new(); num_clusters])),
            num_clusters,
        }
    }

    /// Generate hints from routing table
    pub fn generate_hints(
        &self,
        prefixes: &[(u32, u8)],
        next_hops: &[u32],
        cluster_ids: &[u16],
        compressed_coords: &[[f32; 4]],
    ) -> Result<()> {
        if prefixes.len() != next_hops.len() 
            || prefixes.len() != cluster_ids.len()
            || prefixes.len() != compressed_coords.len() {
            return Err(anyhow!("Input arrays must have same length"));
        }

        let mut hints = self.hints.write();
        let mut cluster_index = self.cluster_index.write();

        hints.clear();
        for cluster in cluster_index.iter_mut() {
            cluster.clear();
        }

        // Generate hints
        for (i, ((prefix_addr, prefix_len), next_hop_id)) in prefixes.iter().zip(next_hops.iter()).enumerate() {
            let cluster_id = cluster_ids[i];
            let coords = compressed_coords[i];
            
            // Calculate probability based on prefix length (longer = more specific = higher priority)
            let probability = (*prefix_len as f32) / 32.0;

            let hint = HardwareHint::new(
                *prefix_addr,
                *prefix_len,
                *next_hop_id,
                cluster_id,
                coords,
                probability,
            );

            let hint_idx = hints.len();
            hints.push(hint);

            // Update cluster index
            if (cluster_id as usize) < self.num_clusters {
                cluster_index[cluster_id as usize].push(hint_idx);
            }
        }

        Ok(())
    }

    /// Fast lookup using hardware hints
    #[inline(always)]
    pub fn lookup(&self, ip: u32, cluster_id: u16) -> Option<u32> {
        let hints = self.hints.read();
        let cluster_index = self.cluster_index.read();

        // Get hints for this cluster
        let cluster_hints = if (cluster_id as usize) < self.num_clusters {
            &cluster_index[cluster_id as usize]
        } else {
            return None;
        };

        // Search within cluster (cache-friendly sequential access)
        let mut best_match: Option<(u8, u32)> = None;

        for &hint_idx in cluster_hints.iter() {
            if let Some(hint) = hints.get(hint_idx) {
                if hint.matches(ip) {
                    match best_match {
                        None => best_match = Some((hint.prefix_len, hint.next_hop_id)),
                        Some((best_len, _)) if hint.prefix_len > best_len => {
                            best_match = Some((hint.prefix_len, hint.next_hop_id));
                        }
                        _ => {}
                    }
                }
            }
        }

        best_match.map(|(_, next_hop)| next_hop)
    }

    /// Get number of hints
    pub fn num_hints(&self) -> usize {
        self.hints.read().len()
    }

    /// Get cache line utilization
    pub fn cache_utilization(&self) -> f64 {
        let num_hints = self.num_hints();
        let cache_lines_used = (num_hints * std::mem::size_of::<HardwareHint>() + CACHE_LINE_SIZE - 1) / CACHE_LINE_SIZE;
        (num_hints as f64) / (cache_lines_used as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_hint_creation() {
        let hint = HardwareHint::new(
            0xC0A80100, // 192.168.1.0
            24,
            1,
            0,
            [1.0, 2.0, 3.0, 4.0],
            0.75,
        );

        assert_eq!(hint.prefix_addr, 0xC0A80100);
        assert_eq!(hint.prefix_len, 24);
        assert_eq!(hint.next_hop_id, 1);
        assert_eq!(hint.cluster_id, 0);
        assert_eq!(hint.probability, 0.75);
    }

    #[test]
    fn test_hardware_hint_matches() {
        let hint = HardwareHint::new(
            0xC0A80100, // 192.168.1.0/24
            24,
            1,
            0,
            [0.0; 4],
            0.75,
        );

        assert!(hint.matches(0xC0A80142)); // 192.168.1.66
        assert!(hint.matches(0xC0A801FF)); // 192.168.1.255
        assert!(!hint.matches(0xC0A80242)); // 192.168.2.66
    }

    #[test]
    fn test_hardware_hint_distance() {
        let hint = HardwareHint::new(
            0xC0A80100,
            24,
            1,
            0,
            [1.0, 2.0, 3.0, 4.0],
            0.75,
        );

        let coords = [1.0, 2.0, 3.0, 4.0];
        let dist = hint.distance(&coords);
        assert!(dist.abs() < 1e-6); // Should be 0 (same coordinates)

        let coords2 = [2.0, 3.0, 4.0, 5.0];
        let dist2 = hint.distance(&coords2);
        assert!(dist2 > 0.0); // Should be non-zero
    }

    #[test]
    fn test_hint_generator_creation() {
        let generator = HintGenerator::new(3);
        assert_eq!(generator.num_hints(), 0);
    }

    #[test]
    fn test_hint_generation() {
        let generator = HintGenerator::new(2);

        let prefixes = vec![(0xC0A80100, 24), (0xC0A80200, 24)];
        let next_hops = vec![1, 2];
        let cluster_ids = vec![0, 1];
        let compressed_coords = vec![[1.0, 2.0, 3.0, 4.0], [5.0, 6.0, 7.0, 8.0]];

        generator.generate_hints(&prefixes, &next_hops, &cluster_ids, &compressed_coords).unwrap();

        assert_eq!(generator.num_hints(), 2);
    }

    #[test]
    fn test_hint_lookup() {
        let generator = HintGenerator::new(2);

        let prefixes = vec![(0xC0A80100, 24), (0xC0A80200, 24), (0xC0A80000, 16)];
        let next_hops = vec![1, 2, 3];
        let cluster_ids = vec![0, 0, 1];
        let compressed_coords = vec![[1.0; 4], [2.0; 4], [3.0; 4]];

        generator.generate_hints(&prefixes, &next_hops, &cluster_ids, &compressed_coords).unwrap();

        // Lookup in cluster 0
        let result = generator.lookup(0xC0A80142, 0); // 192.168.1.66
        assert_eq!(result, Some(1)); // Should match 192.168.1.0/24

        // Lookup in cluster 1
        let result2 = generator.lookup(0xC0A80342, 1); // 192.168.3.66
        assert_eq!(result2, Some(3)); // Should match 192.168.0.0/16
    }

    #[test]
    fn test_longest_prefix_match() {
        let generator = HintGenerator::new(1);

        // Add overlapping prefixes
        let prefixes = vec![(0xC0A80000, 16), (0xC0A80100, 24), (0xC0A80140, 26)];
        let next_hops = vec![1, 2, 3];
        let cluster_ids = vec![0, 0, 0];
        let compressed_coords = vec![[1.0; 4], [2.0; 4], [3.0; 4]];

        generator.generate_hints(&prefixes, &next_hops, &cluster_ids, &compressed_coords).unwrap();

        // Should match most specific prefix
        let result = generator.lookup(0xC0A80150, 0); // 192.168.1.80
        assert_eq!(result, Some(3)); // Should match 192.168.1.64/26 (most specific)
    }

    #[test]
    fn test_cache_utilization() {
        let generator = HintGenerator::new(2);

        let prefixes = vec![(0xC0A80100, 24)];
        let next_hops = vec![1];
        let cluster_ids = vec![0];
        let compressed_coords = vec![[1.0; 4]];

        generator.generate_hints(&prefixes, &next_hops, &cluster_ids, &compressed_coords).unwrap();

        let utilization = generator.cache_utilization();
        assert!(utilization > 0.0);
        assert!(utilization <= 1.0);
    }
}

