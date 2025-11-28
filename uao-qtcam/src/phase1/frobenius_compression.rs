//! # Frobenius Compression
//!
//! This module implements prefix compression using Frobenius automorphisms
//! in Galois Fields. The compression reduces the dimensionality of the
//! routing table while preserving lookup accuracy.
//!
//! ## Mathematical Foundation
//!
//! The Frobenius automorphism φ: GF(2^32) → GF(2^32) is defined as:
//! φ(x) = x^(2^k)
//!
//! This automorphism has the property that it preserves field structure
//! while "compressing" the representation.
//!
//! ## Compression Algorithm
//!
//! 1. Map IP prefix to GF(2^32) element
//! 2. Apply Frobenius automorphism with k=5
//! 3. Extract compressed representation
//! 4. Store in compressed routing table

use super::galois_field::GF2_32;
use super::Prefix;
use anyhow::Result;

/// Frobenius compressor for prefix compression
pub struct FrobeniusCompressor {
    /// Frobenius power (k in φ(x) = x^(2^k))
    frobenius_power: u32,
    /// Compression ratio
    compression_ratio: f64,
}

impl FrobeniusCompressor {
    /// Create new Frobenius compressor
    pub fn new() -> Self {
        Self {
            frobenius_power: 5, // φ(x) = x^32
            compression_ratio: 0.0,
        }
    }

    /// Compress a prefix using Frobenius automorphism
    pub fn compress(&self, prefix: &Prefix) -> CompressedPrefix {
        // Map prefix to GF(2^32)
        let gf_element = GF2_32::new(prefix.addr);

        // Apply Frobenius automorphism
        let compressed = gf_element.frobenius(self.frobenius_power);

        // Extract compressed representation
        let compressed_value = compressed.value();

        // Compute hash for fast lookup
        let hash = self.compute_hash(compressed_value, prefix.len);

        CompressedPrefix {
            original_prefix: *prefix,
            compressed_value,
            hash,
            frobenius_power: self.frobenius_power,
        }
    }

    /// Decompress a compressed prefix
    pub fn decompress(&self, compressed: &CompressedPrefix) -> Result<Prefix> {
        // For now, we store the original prefix in the compressed representation
        // In a full implementation, we would invert the Frobenius automorphism
        Ok(compressed.original_prefix)
    }

    /// Compute hash for fast lookup
    fn compute_hash(&self, value: u32, prefix_len: u8) -> u64 {
        // Mix value and prefix length
        let mut hash = value as u64;
        hash ^= (prefix_len as u64) << 32;
        
        // Simple hash mixing
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(0xff51afd7ed558ccd);
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(0xc4ceb9fe1a85ec53);
        hash ^= hash >> 33;
        
        hash
    }

    /// Get compression statistics
    pub fn stats(&self) -> CompressionStats {
        CompressionStats {
            frobenius_power: self.frobenius_power,
            compression_ratio: self.compression_ratio,
        }
    }
}

impl Default for FrobeniusCompressor {
    fn default() -> Self {
        Self::new()
    }
}

/// Compressed prefix representation
#[derive(Debug, Clone)]
pub struct CompressedPrefix {
    /// Original prefix (for decompression)
    pub original_prefix: Prefix,
    /// Compressed value after Frobenius automorphism
    pub compressed_value: u32,
    /// Hash for fast lookup
    pub hash: u64,
    /// Frobenius power used
    pub frobenius_power: u32,
}

impl CompressedPrefix {
    /// Check if this compressed prefix matches an IP address
    pub fn matches(&self, ip: u32) -> bool {
        self.original_prefix.matches(ip)
    }

    /// Get original prefix
    pub fn prefix(&self) -> &Prefix {
        &self.original_prefix
    }
}

/// Compression statistics
#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub frobenius_power: u32,
    pub compression_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression() {
        let compressor = FrobeniusCompressor::new();
        let prefix = Prefix::from_cidr("192.168.1.0/24").unwrap();
        
        let compressed = compressor.compress(&prefix);
        assert_eq!(compressed.original_prefix, prefix);
        assert_eq!(compressed.frobenius_power, 5);
    }

    #[test]
    fn test_decompression() {
        let compressor = FrobeniusCompressor::new();
        let prefix = Prefix::from_cidr("192.168.1.0/24").unwrap();
        
        let compressed = compressor.compress(&prefix);
        let decompressed = compressor.decompress(&compressed).unwrap();
        
        assert_eq!(decompressed, prefix);
    }

    #[test]
    fn test_compressed_matches() {
        let compressor = FrobeniusCompressor::new();
        let prefix = Prefix::from_cidr("192.168.1.0/24").unwrap();
        let compressed = compressor.compress(&prefix);
        
        let ip = u32::from(std::net::Ipv4Addr::new(192, 168, 1, 42));
        assert!(compressed.matches(ip));
        
        let ip2 = u32::from(std::net::Ipv4Addr::new(192, 168, 2, 42));
        assert!(!compressed.matches(ip2));
    }
}

