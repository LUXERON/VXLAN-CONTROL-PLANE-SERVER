//! Dimensional Folding for Prefix Space Compression
//!
//! This module implements dimensional reduction using SVD (Singular Value Decomposition)
//! to compress high-dimensional routing prefix space into lower dimensions while
//! preserving distance relationships for fast lookup.

use nalgebra::{DMatrix, DVector, SVD};
use anyhow::{Result, anyhow};

/// Dimensional folder for prefix space compression
#[derive(Debug)]
pub struct DimensionalFolder {
    /// Target dimension (k)
    target_dim: usize,
    /// Singular values
    singular_values: Vec<f64>,
    /// Left singular vectors (U matrix)
    u_matrix: Option<DMatrix<f64>>,
    /// Right singular vectors (V^T matrix)
    vt_matrix: Option<DMatrix<f64>>,
    /// Compression ratio achieved
    compression_ratio: f64,
}

impl DimensionalFolder {
    /// Create a new dimensional folder
    ///
    /// # Arguments
    /// * `target_dim` - Target dimension for compression (k)
    pub fn new(target_dim: usize) -> Self {
        Self {
            target_dim,
            singular_values: Vec::new(),
            u_matrix: None,
            vt_matrix: None,
            compression_ratio: 0.0,
        }
    }

    /// Fold high-dimensional prefix space using SVD
    ///
    /// # Arguments
    /// * `prefix_matrix` - Matrix where each row is a prefix feature vector
    ///
    /// # Returns
    /// Compressed representation in lower dimensions
    pub fn fold(&mut self, prefix_matrix: &DMatrix<f64>) -> Result<DMatrix<f64>> {
        let (rows, cols) = prefix_matrix.shape();
        
        if rows == 0 || cols == 0 {
            return Err(anyhow!("Empty prefix matrix"));
        }

        if self.target_dim > cols.min(rows) {
            return Err(anyhow!("Target dimension {} exceeds matrix dimensions", self.target_dim));
        }

        // Perform SVD: A = U * Σ * V^T
        let svd = SVD::new(prefix_matrix.clone(), true, true);
        
        // Extract components
        let u = svd.u.ok_or_else(|| anyhow!("SVD failed to compute U matrix"))?;
        let vt = svd.v_t.ok_or_else(|| anyhow!("SVD failed to compute V^T matrix"))?;
        let singular_values = svd.singular_values;

        // Store singular values
        self.singular_values = singular_values.iter().copied().collect();

        // Truncate to target dimension
        let u_truncated = u.columns(0, self.target_dim).clone_owned();
        let vt_truncated = vt.rows(0, self.target_dim).clone_owned();
        let sigma_truncated = DMatrix::from_diagonal(
            &DVector::from_iterator(
                self.target_dim,
                singular_values.iter().take(self.target_dim).copied()
            )
        );

        // Store matrices
        self.u_matrix = Some(u_truncated.clone());
        self.vt_matrix = Some(vt_truncated.clone());

        // Calculate compression ratio
        let original_energy: f64 = singular_values.iter().map(|s| s * s).sum();
        let compressed_energy: f64 = singular_values.iter().take(self.target_dim).map(|s| s * s).sum();
        self.compression_ratio = compressed_energy / original_energy;

        // Compressed representation: U_k * Σ_k
        Ok(&u_truncated * &sigma_truncated)
    }

    /// Unfold compressed representation back to original space
    ///
    /// # Arguments
    /// * `compressed` - Compressed prefix matrix
    ///
    /// # Returns
    /// Reconstructed prefix matrix in original dimensions
    pub fn unfold(&self, compressed: &DMatrix<f64>) -> Result<DMatrix<f64>> {
        let vt = self.vt_matrix.as_ref()
            .ok_or_else(|| anyhow!("Must call fold() before unfold()"))?;

        // Reconstruct: (U_k * Σ_k) * V_k^T
        Ok(compressed * vt)
    }

    /// Project a single prefix into compressed space
    ///
    /// # Arguments
    /// * `prefix_vector` - Feature vector for a single prefix
    ///
    /// # Returns
    /// Compressed representation of the prefix
    pub fn project(&self, prefix_vector: &DVector<f64>) -> Result<DVector<f64>> {
        let vt = self.vt_matrix.as_ref()
            .ok_or_else(|| anyhow!("Must call fold() before project()"))?;

        if prefix_vector.len() != vt.ncols() {
            return Err(anyhow!("Prefix vector dimension mismatch"));
        }

        // Project: x_compressed = V_k^T * x
        Ok(vt * prefix_vector)
    }

    /// Get compression ratio (0.0 to 1.0)
    pub fn compression_ratio(&self) -> f64 {
        self.compression_ratio
    }

    /// Get singular values
    pub fn singular_values(&self) -> &[f64] {
        &self.singular_values
    }

    /// Get target dimension
    pub fn target_dim(&self) -> usize {
        self.target_dim
    }

    /// Calculate reconstruction error
    pub fn reconstruction_error(&self, original: &DMatrix<f64>, compressed: &DMatrix<f64>) -> Result<f64> {
        let reconstructed = self.unfold(compressed)?;
        
        if original.shape() != reconstructed.shape() {
            return Err(anyhow!("Shape mismatch in reconstruction"));
        }

        // Frobenius norm of difference
        let diff = original - reconstructed;
        let error = diff.iter().map(|x| x * x).sum::<f64>().sqrt();
        
        Ok(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimensional_folder_creation() {
        let folder = DimensionalFolder::new(5);
        assert_eq!(folder.target_dim(), 5);
    }

    #[test]
    fn test_svd_folding() {
        let mut folder = DimensionalFolder::new(2);

        // Create a simple 4x3 matrix
        let matrix = DMatrix::from_row_slice(4, 3, &[
            1.0, 2.0, 3.0,
            4.0, 5.0, 6.0,
            7.0, 8.0, 9.0,
            10.0, 11.0, 12.0,
        ]);

        let compressed = folder.fold(&matrix).unwrap();

        // Compressed should be 4x2 (rows x target_dim)
        assert_eq!(compressed.nrows(), 4);
        assert_eq!(compressed.ncols(), 2);

        // Compression ratio should be between 0 and 1
        assert!(folder.compression_ratio() > 0.0);
        assert!(folder.compression_ratio() <= 1.0);
    }

    #[test]
    fn test_fold_unfold_reconstruction() {
        let mut folder = DimensionalFolder::new(2);

        let matrix = DMatrix::from_row_slice(3, 4, &[
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
        ]);

        let compressed = folder.fold(&matrix).unwrap();
        let reconstructed = folder.unfold(&compressed).unwrap();

        // Reconstructed should have same shape as original
        assert_eq!(reconstructed.shape(), matrix.shape());

        // Calculate reconstruction error
        let error = folder.reconstruction_error(&matrix, &compressed).unwrap();
        assert!(error >= 0.0);
    }

    #[test]
    fn test_project_single_prefix() {
        let mut folder = DimensionalFolder::new(2);

        let matrix = DMatrix::from_row_slice(3, 4, &[
            1.0, 2.0, 3.0, 4.0,
            5.0, 6.0, 7.0, 8.0,
            9.0, 10.0, 11.0, 12.0,
        ]);

        folder.fold(&matrix).unwrap();

        // Project a single prefix vector
        let prefix_vec = DVector::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
        let projected = folder.project(&prefix_vec).unwrap();

        // Projected should have target dimension
        assert_eq!(projected.len(), 2);
    }

    #[test]
    fn test_singular_values() {
        let mut folder = DimensionalFolder::new(2);

        let matrix = DMatrix::from_row_slice(3, 3, &[
            1.0, 0.0, 0.0,
            0.0, 2.0, 0.0,
            0.0, 0.0, 3.0,
        ]);

        folder.fold(&matrix).unwrap();

        let singular_vals = folder.singular_values();
        assert_eq!(singular_vals.len(), 3);

        // Singular values should be in descending order
        for i in 0..(singular_vals.len() - 1) {
            assert!(singular_vals[i] >= singular_vals[i + 1]);
        }
    }
}

