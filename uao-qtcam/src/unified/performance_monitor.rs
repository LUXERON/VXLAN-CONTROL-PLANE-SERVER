//! # Performance Monitor
//!
//! Real-time performance monitoring and metrics collection for the TCAM engine.

use std::time::Instant;
use parking_lot::RwLock;

/// Performance monitor
pub struct PerformanceMonitor {
    metrics: RwLock<Metrics>,
}

#[derive(Debug, Clone, Default)]
struct Metrics {
    start_time: Option<Instant>,
    total_operations: u64,
    total_latency_ns: u64,
    min_latency_ns: u64,
    max_latency_ns: u64,
    p50_latency_ns: u64,
    p95_latency_ns: u64,
    p99_latency_ns: u64,
}

impl PerformanceMonitor {
    /// Create new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: RwLock::new(Metrics {
                start_time: Some(Instant::now()),
                min_latency_ns: u64::MAX,
                ..Default::default()
            }),
        }
    }

    /// Record an operation latency
    pub fn record_latency(&self, latency_ns: u64) {
        let mut metrics = self.metrics.write();
        metrics.total_operations += 1;
        metrics.total_latency_ns += latency_ns;
        metrics.min_latency_ns = metrics.min_latency_ns.min(latency_ns);
        metrics.max_latency_ns = metrics.max_latency_ns.max(latency_ns);
    }

    /// Get average latency in nanoseconds
    pub fn avg_latency_ns(&self) -> f64 {
        let metrics = self.metrics.read();
        if metrics.total_operations == 0 {
            0.0
        } else {
            metrics.total_latency_ns as f64 / metrics.total_operations as f64
        }
    }

    /// Get operations per second
    pub fn ops_per_second(&self) -> f64 {
        let metrics = self.metrics.read();
        if let Some(start_time) = metrics.start_time {
            let elapsed = start_time.elapsed().as_secs_f64();
            if elapsed > 0.0 {
                metrics.total_operations as f64 / elapsed
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Get total operations
    pub fn total_operations(&self) -> u64 {
        self.metrics.read().total_operations
    }

    /// Get min latency
    pub fn min_latency_ns(&self) -> u64 {
        let metrics = self.metrics.read();
        if metrics.min_latency_ns == u64::MAX {
            0
        } else {
            metrics.min_latency_ns
        }
    }

    /// Get max latency
    pub fn max_latency_ns(&self) -> u64 {
        self.metrics.read().max_latency_ns
    }

    /// Reset metrics
    pub fn reset(&self) {
        let mut metrics = self.metrics.write();
        *metrics = Metrics {
            start_time: Some(Instant::now()),
            min_latency_ns: u64::MAX,
            ..Default::default()
        };
    }

    /// Get performance summary
    pub fn summary(&self) -> PerformanceSummary {
        let metrics = self.metrics.read();
        PerformanceSummary {
            total_operations: metrics.total_operations,
            avg_latency_ns: self.avg_latency_ns(),
            min_latency_ns: self.min_latency_ns(),
            max_latency_ns: metrics.max_latency_ns,
            ops_per_second: self.ops_per_second(),
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance summary
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_operations: u64,
    pub avg_latency_ns: f64,
    pub min_latency_ns: u64,
    pub max_latency_ns: u64,
    pub ops_per_second: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_latency() {
        let monitor = PerformanceMonitor::new();
        
        monitor.record_latency(100);
        monitor.record_latency(200);
        monitor.record_latency(150);
        
        assert_eq!(monitor.total_operations(), 3);
        assert_eq!(monitor.avg_latency_ns(), 150.0);
        assert_eq!(monitor.min_latency_ns(), 100);
        assert_eq!(monitor.max_latency_ns(), 200);
    }

    #[test]
    fn test_reset() {
        let monitor = PerformanceMonitor::new();
        
        monitor.record_latency(100);
        assert_eq!(monitor.total_operations(), 1);
        
        monitor.reset();
        assert_eq!(monitor.total_operations(), 0);
    }

    #[test]
    fn test_summary() {
        let monitor = PerformanceMonitor::new();
        
        monitor.record_latency(100);
        monitor.record_latency(200);
        
        let summary = monitor.summary();
        assert_eq!(summary.total_operations, 2);
        assert_eq!(summary.avg_latency_ns, 150.0);
        assert_eq!(summary.min_latency_ns, 100);
        assert_eq!(summary.max_latency_ns, 200);
    }
}

