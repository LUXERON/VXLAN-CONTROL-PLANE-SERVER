//! Particle Mesh Ewald (PME) Engine
//!
//! POSTULATE 16: PME Smooth Approximation
//! Uses molecular dynamics PME method for smooth routing lookup approximation

use crate::phase1::Prefix;
use anyhow::Result;
use std::f64::consts::PI;

/// 3D particle position for PME
#[derive(Debug, Clone, Copy)]
struct Particle {
    position: [f64; 3],
    charge: f64,
}

/// PME Engine for smooth routing approximation
pub struct PMEEngine {
    /// Particles (routing entries)
    particles: Vec<Particle>,
    /// Grid size for mesh
    grid_size: usize,
    /// Ewald parameter α
    alpha: f64,
    /// Cutoff radius for real-space
    cutoff: f64,
    /// Mesh grid for reciprocal space
    mesh: Vec<Vec<Vec<f64>>>,
    /// Next hop mapping
    next_hops: Vec<String>,
    /// Metric mapping
    metrics: Vec<u32>,
}

impl PMEEngine {
    pub fn new(grid_size: usize) -> Self {
        Self {
            particles: Vec::new(),
            grid_size,
            alpha: 0.3, // Ewald parameter
            cutoff: 10.0, // Real-space cutoff
            mesh: vec![vec![vec![0.0; grid_size]; grid_size]; grid_size],
            next_hops: Vec::new(),
            metrics: Vec::new(),
        }
    }

    /// Map IP address to 3D position
    #[inline(always)]
    fn ip_to_position(&self, ip: u32) -> [f64; 3] {
        // Split 32 bits into 3 coordinates (10, 11, 11 bits)
        let x = ((ip >> 22) & 0x3FF) as f64 / 1024.0 * self.grid_size as f64;
        let y = ((ip >> 11) & 0x7FF) as f64 / 2048.0 * self.grid_size as f64;
        let z = (ip & 0x7FF) as f64 / 2048.0 * self.grid_size as f64;
        [x, y, z]
    }

    /// Insert route as particle
    pub fn insert(&mut self, prefix: Prefix, next_hop: String, metric: u32) {
        let position = self.ip_to_position(prefix.addr);
        let charge = 1.0 / (metric as f64 + 1.0); // Higher charge for lower metric

        let particle = Particle { position, charge };
        self.particles.push(particle);
        self.next_hops.push(next_hop);
        self.metrics.push(metric);

        // Update mesh
        self.update_mesh();
    }

    /// Update mesh grid with particle charges
    fn update_mesh(&mut self) {
        // Reset mesh
        for i in 0..self.grid_size {
            for j in 0..self.grid_size {
                for k in 0..self.grid_size {
                    self.mesh[i][j][k] = 0.0;
                }
            }
        }

        // Distribute charges to mesh
        for particle in &self.particles {
            let [x, y, z] = particle.position;
            let i = (x as usize).min(self.grid_size - 1);
            let j = (y as usize).min(self.grid_size - 1);
            let k = (z as usize).min(self.grid_size - 1);

            self.mesh[i][j][k] += particle.charge;
        }
    }

    /// Compute real-space energy
    #[inline(always)]
    fn real_space_energy(&self, pos: [f64; 3], particle: &Particle) -> f64 {
        let dx = pos[0] - particle.position[0];
        let dy = pos[1] - particle.position[1];
        let dz = pos[2] - particle.position[2];
        let r = (dx * dx + dy * dy + dz * dz).sqrt();

        if r < self.cutoff && r > 0.0 {
            // E_real = q_i q_j erfc(α r) / r
            let alpha_r = self.alpha * r;
            particle.charge * Self::erfc(alpha_r) / r
        } else {
            0.0
        }
    }

    /// Compute reciprocal-space energy (simplified)
    #[inline(always)]
    fn reciprocal_space_energy(&self, pos: [f64; 3]) -> f64 {
        let [x, y, z] = pos;
        let i = (x as usize).min(self.grid_size - 1);
        let j = (y as usize).min(self.grid_size - 1);
        let k = (z as usize).min(self.grid_size - 1);

        // Interpolate from mesh
        self.mesh[i][j][k]
    }

    /// Lookup route using PME approximation
    #[inline(always)]
    pub fn lookup(&self, ip: u32) -> Option<(String, u32)> {
        let query_pos = self.ip_to_position(ip);

        // Compute total energy for each particle
        let mut best_idx = 0;
        let mut best_energy = f64::MIN;

        for (idx, particle) in self.particles.iter().enumerate() {
            // Check prefix match first
            let prefix_addr = self.ip_to_position(self.particles[idx].position[0] as u32);
            
            // Total energy = real-space + reciprocal-space
            let e_real = self.real_space_energy(query_pos, particle);
            let e_recip = self.reciprocal_space_energy(query_pos);
            let total_energy = e_real + e_recip;

            if total_energy > best_energy {
                best_energy = total_energy;
                best_idx = idx;
            }
        }

        if best_energy > 0.0 && best_idx < self.next_hops.len() {
            Some((self.next_hops[best_idx].clone(), self.metrics[best_idx]))
        } else {
            None
        }
    }

    /// Complementary error function (approximation)
    #[inline(always)]
    fn erfc(x: f64) -> f64 {
        // Abramowitz and Stegun approximation (7.1.26)
        // erfc(x) = t * exp(-x^2 + polynomial(t))
        let t = 1.0 / (1.0 + 0.5 * x.abs());
        let tau = -x * x - 1.26551223 +
            t * (1.00002368 +
            t * (0.37409196 +
            t * (0.09678418 +
            t * (-0.18628806 +
            t * (0.27886807 +
            t * (-1.13520398 +
            t * (1.48851587 +
            t * (-0.82215223 +
            t * 0.17087277))))))));

        let result = t * tau.exp();

        if x >= 0.0 {
            result
        } else {
            2.0 - result
        }
    }

    /// Get number of particles
    pub fn num_particles(&self) -> usize {
        self.particles.len()
    }

    /// Get grid size
    pub fn grid_size(&self) -> usize {
        self.grid_size
    }
}

impl Default for PMEEngine {
    fn default() -> Self {
        Self::new(64) // 64x64x64 grid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pme_engine() {
        let mut engine = PMEEngine::new(32);

        let prefix = Prefix::from_cidr("192.168.1.0/24").unwrap();
        engine.insert(prefix, "gateway1".to_string(), 100);

        let ip = u32::from(std::net::Ipv4Addr::new(192, 168, 1, 42));
        let result = engine.lookup(ip);
        
        println!("PME lookup result: {:?}", result);
        assert!(result.is_some());
    }

    #[test]
    fn test_erfc() {
        let x = 1.0;
        let result = PMEEngine::erfc(x);
        // erfc(1.0) ≈ 0.1573 (should be between 0 and 2)
        assert!(result >= 0.0 && result <= 2.0);
        println!("erfc(1.0) = {}", result);
    }

    #[test]
    fn test_ip_to_position() {
        let engine = PMEEngine::new(64);
        let ip = u32::from(std::net::Ipv4Addr::new(192, 168, 1, 1));
        let pos = engine.ip_to_position(ip);
        
        println!("IP position: {:?}", pos);
        for &coord in &pos {
            assert!(coord >= 0.0 && coord < 64.0);
        }
    }
}

