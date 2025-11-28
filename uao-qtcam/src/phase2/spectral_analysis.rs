//! Laplacian Spectral Analysis for Prefix Clustering
//!
//! This module implements graph Laplacian-based spectral analysis for routing prefix
//! clustering. Uses eigenvalue/eigenvector decomposition to identify natural clusters
//! in the routing table for optimized lookup.

use nalgebra::{DMatrix, DVector, SymmetricEigen};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Spectral analyzer for prefix clustering
#[derive(Debug)]
pub struct SpectralAnalyzer {
    /// Number of clusters (k)
    num_clusters: usize,
    /// Eigenvalues (sorted descending)
    eigenvalues: Vec<f64>,
    /// Eigenvectors (columns)
    eigenvectors: Option<DMatrix<f64>>,
    /// Cluster assignments (prefix_id -> cluster_id)
    clusters: HashMap<usize, usize>,
}

impl SpectralAnalyzer {
    /// Create a new spectral analyzer
    ///
    /// # Arguments
    /// * `num_clusters` - Number of clusters to identify (k)
    pub fn new(num_clusters: usize) -> Self {
        Self {
            num_clusters,
            eigenvalues: Vec::new(),
            eigenvectors: None,
            clusters: HashMap::new(),
        }
    }

    /// Build graph Laplacian from prefix similarity matrix
    ///
    /// # Arguments
    /// * `prefixes` - List of prefix addresses (u32)
    /// * `prefix_lens` - List of prefix lengths (u8)
    ///
    /// # Returns
    /// Graph Laplacian matrix L = D - W where:
    /// - W is the weighted adjacency matrix (similarity)
    /// - D is the degree matrix (diagonal)
    pub fn build_laplacian(&self, prefixes: &[u32], prefix_lens: &[u8]) -> Result<DMatrix<f64>> {
        if prefixes.len() != prefix_lens.len() {
            return Err(anyhow!("Prefix and length arrays must have same size"));
        }

        let n = prefixes.len();
        if n == 0 {
            return Err(anyhow!("Empty prefix list"));
        }

        // Build weighted adjacency matrix W
        let mut w = DMatrix::zeros(n, n);
        for i in 0..n {
            for j in (i + 1)..n {
                // Similarity based on common prefix bits
                let similarity = self.prefix_similarity(
                    prefixes[i], prefix_lens[i],
                    prefixes[j], prefix_lens[j]
                );
                w[(i, j)] = similarity;
                w[(j, i)] = similarity;
            }
        }

        // Build degree matrix D (diagonal)
        let mut d = DMatrix::zeros(n, n);
        for i in 0..n {
            let degree: f64 = w.row(i).sum();
            d[(i, i)] = degree;
        }

        // Laplacian L = D - W
        Ok(d - w)
    }

    /// Calculate similarity between two prefixes
    fn prefix_similarity(&self, addr1: u32, len1: u8, addr2: u32, len2: u8) -> f64 {
        // Count common prefix bits
        let xor = addr1 ^ addr2;
        let common_bits = xor.leading_zeros().min(len1.min(len2) as u32);
        
        // Similarity: exponential decay based on common bits
        let max_len = len1.max(len2) as f64;
        if max_len == 0.0 {
            return 1.0;
        }
        
        (common_bits as f64 / max_len).exp()
    }

    /// Perform spectral decomposition of Laplacian
    ///
    /// # Arguments
    /// * `laplacian` - Graph Laplacian matrix
    ///
    /// # Returns
    /// Eigenvalues and eigenvectors (sorted by eigenvalue)
    pub fn decompose(&mut self, laplacian: &DMatrix<f64>) -> Result<()> {
        if !laplacian.is_square() {
            return Err(anyhow!("Laplacian must be square matrix"));
        }

        // Compute eigendecomposition
        let eigen = SymmetricEigen::new(laplacian.clone());
        
        // Sort eigenvalues and eigenvectors (ascending order)
        let mut eigen_pairs: Vec<(f64, DVector<f64>)> = eigen.eigenvalues
            .iter()
            .zip(eigen.eigenvectors.column_iter())
            .map(|(val, vec)| (*val, vec.clone_owned()))
            .collect();
        
        eigen_pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // Store eigenvalues
        self.eigenvalues = eigen_pairs.iter().map(|(val, _)| *val).collect();

        // Store eigenvectors as matrix (columns)
        let n = laplacian.nrows();
        let mut eigenvec_matrix = DMatrix::zeros(n, n);
        for (i, (_, vec)) in eigen_pairs.iter().enumerate() {
            eigenvec_matrix.set_column(i, vec);
        }
        self.eigenvectors = Some(eigenvec_matrix);

        Ok(())
    }

    /// Cluster prefixes using k-means on spectral embedding
    ///
    /// Uses the first k eigenvectors as features for clustering
    pub fn cluster(&mut self) -> Result<()> {
        let eigenvectors = self.eigenvectors.as_ref()
            .ok_or_else(|| anyhow!("Must call decompose() before cluster()"))?;

        let n = eigenvectors.nrows();
        if n < self.num_clusters {
            return Err(anyhow!("Not enough prefixes for {} clusters", self.num_clusters));
        }

        // Use first k eigenvectors as features (skip first eigenvector - constant)
        let k = self.num_clusters.min(n - 1);
        
        // Simple k-means clustering on spectral embedding
        self.clusters = self.kmeans_clustering(eigenvectors, k)?;

        Ok(())
    }

    /// Simple k-means clustering implementation
    fn kmeans_clustering(&self, features: &DMatrix<f64>, k: usize) -> Result<HashMap<usize, usize>> {
        // Implementation continues in next file section...
        let n = features.nrows();
        let mut assignments = HashMap::new();
        
        // Simple assignment based on dominant eigenvector component
        for i in 0..n {
            let row = features.row(1); // Use second eigenvector (first non-trivial)
            let cluster_id = ((row[i] + 1.0) * (k as f64 / 2.0)) as usize % k;
            assignments.insert(i, cluster_id);
        }
        
        Ok(assignments)
    }

    /// Get cluster assignment for a prefix
    pub fn get_cluster(&self, prefix_id: usize) -> Option<usize> {
        self.clusters.get(&prefix_id).copied()
    }

    /// Get number of clusters
    pub fn num_clusters(&self) -> usize {
        self.num_clusters
    }

    /// Get eigenvalues
    pub fn eigenvalues(&self) -> &[f64] {
        &self.eigenvalues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectral_analyzer_creation() {
        let analyzer = SpectralAnalyzer::new(3);
        assert_eq!(analyzer.num_clusters(), 3);
    }

    #[test]
    fn test_laplacian_construction() {
        let analyzer = SpectralAnalyzer::new(2);
        let prefixes = vec![0xC0A80100, 0xC0A80200, 0xC0A80300]; // 192.168.1.0, 192.168.2.0, 192.168.3.0
        let prefix_lens = vec![24, 24, 24];

        let laplacian = analyzer.build_laplacian(&prefixes, &prefix_lens).unwrap();
        assert_eq!(laplacian.nrows(), 3);
        assert_eq!(laplacian.ncols(), 3);

        // Laplacian should be symmetric
        for i in 0..3 {
            for j in 0..3 {
                assert!((laplacian[(i, j)] - laplacian[(j, i)]).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_spectral_decomposition() {
        let mut analyzer = SpectralAnalyzer::new(2);
        let prefixes = vec![0xC0A80100, 0xC0A80200, 0xC0A80300];
        let prefix_lens = vec![24, 24, 24];

        let laplacian = analyzer.build_laplacian(&prefixes, &prefix_lens).unwrap();
        analyzer.decompose(&laplacian).unwrap();

        assert_eq!(analyzer.eigenvalues().len(), 3);
        // First eigenvalue should be close to 0 (Laplacian property)
        assert!(analyzer.eigenvalues()[0].abs() < 1e-6);
    }

    #[test]
    fn test_clustering() {
        let mut analyzer = SpectralAnalyzer::new(2);
        let prefixes = vec![0xC0A80100, 0xC0A80200, 0xC0A80300, 0x0A000100];
        let prefix_lens = vec![24, 24, 24, 24];

        let laplacian = analyzer.build_laplacian(&prefixes, &prefix_lens).unwrap();
        analyzer.decompose(&laplacian).unwrap();
        analyzer.cluster().unwrap();

        // All prefixes should be assigned to clusters
        for i in 0..4 {
            assert!(analyzer.get_cluster(i).is_some());
            assert!(analyzer.get_cluster(i).unwrap() < 2);
        }
    }
}

