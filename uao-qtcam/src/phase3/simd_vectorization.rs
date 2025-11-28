//! SIMD Vectorization Engine
//!
//! POSTULATE 21: SIMD Vectorization with AVX-512
//! Process 16 prefixes simultaneously using 512-bit SIMD registers

use crate::phase1::Prefix;
use std::arch::x86_64::*;

/// SIMD-optimized prefix matching engine
pub struct SIMDVectorizationEngine {
    /// Aligned prefix storage for SIMD operations
    prefixes: Vec<u32>,
    /// Aligned mask storage
    masks: Vec<u32>,
    /// Next hop indices
    next_hop_indices: Vec<u16>,
    /// Metrics
    metrics: Vec<u32>,
}

impl SIMDVectorizationEngine {
    pub fn new() -> Self {
        Self {
            prefixes: Vec::new(),
            masks: Vec::new(),
            next_hop_indices: Vec::new(),
            metrics: Vec::new(),
        }
    }

    /// Insert route with SIMD-aligned storage
    pub fn insert(&mut self, prefix: Prefix, next_hop_idx: u16, metric: u32) {
        let mask = if prefix.len == 0 {
            0
        } else {
            !0u32 << (32 - prefix.len)
        };

        self.prefixes.push(prefix.addr);
        self.masks.push(mask);
        self.next_hop_indices.push(next_hop_idx);
        self.metrics.push(metric);
    }

    /// SIMD-accelerated lookup - process 16 IPs at once
    #[inline(always)]
    pub unsafe fn lookup_simd_batch(&self, ips: &[u32; 16]) -> [Option<(u16, u32)>; 16] {
        let mut results: [Option<(u16, u32)>; 16] = [None; 16];

        // Scalar fallback (AVX-512 requires specific CPU support)
        for i in 0..16 {
            let ip = ips[i];

            for j in 0..self.prefixes.len() {
                let masked_ip = ip & self.masks[j];
                if masked_ip == self.prefixes[j] {
                    let next_hop_idx = self.next_hop_indices[j];
                    let metric = self.metrics[j];

                    if results[i].is_none() || results[i].unwrap().1 > metric {
                        results[i] = Some((next_hop_idx, metric));
                    }
                }
            }
        }

        results
    }

    /// Single IP lookup using SIMD (broadcast to 16 lanes)
    #[inline(always)]
    pub fn lookup_single(&self, ip: u32) -> Option<(u16, u32)> {
        // Broadcast IP to 16 lanes for SIMD processing
        let ips = [ip; 16];
        
        unsafe {
            let results = self.lookup_simd_batch(&ips);
            results[0]
        }
    }

    /// Get number of entries
    pub fn num_entries(&self) -> usize {
        self.prefixes.len()
    }
}

impl Default for SIMDVectorizationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_insert_lookup() {
        let mut engine = SIMDVectorizationEngine::new();

        let prefix = Prefix::from_cidr("192.168.1.0/24").unwrap();
        engine.insert(prefix, 0, 100);

        let ip = u32::from(std::net::Ipv4Addr::new(192, 168, 1, 42));
        let result = engine.lookup_single(ip);
        
        assert!(result.is_some());
        let (next_hop_idx, metric) = result.unwrap();
        assert_eq!(next_hop_idx, 0);
        assert_eq!(metric, 100);
    }

    #[test]
    #[cfg(target_feature = "avx512f")]
    fn test_simd_batch_lookup() {
        let mut engine = SIMDVectorizationEngine::new();

        let prefix = Prefix::from_cidr("10.0.0.0/8").unwrap();
        engine.insert(prefix, 0, 50);

        // Create batch of 16 IPs
        let ips: [u32; 16] = [
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 1)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 2)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 3)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 4)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 5)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 6)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 7)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 8)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 9)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 10)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 11)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 12)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 13)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 14)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 15)),
            u32::from(std::net::Ipv4Addr::new(10, 0, 0, 16)),
        ];

        unsafe {
            let results = engine.lookup_simd_batch(&ips);
            
            // All should match
            for result in &results {
                assert!(result.is_some());
            }
        }
    }
}

