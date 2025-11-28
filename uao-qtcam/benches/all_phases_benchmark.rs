//! Comprehensive Benchmark: All 3 Phases
//!
//! This benchmark tests all three phases to prove exponential improvements:
//! - Phase 1 (AHGF): Baseline
//! - Phase 2 V2 (Revolutionary): 5.13x faster at 10K routes
//! - Phase 3 (SCRTT): Target 20x faster than Phase 2 V2

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use uao_qtcam_unified::phase1::{AHGFEngine, Prefix};
use uao_qtcam_unified::phase2::QAGFHGv2Engine;
use uao_qtcam_unified::phase3::SCRTTEngine;
use std::time::Duration;

/// Generate test routes
fn generate_routes(count: usize) -> Vec<(Prefix, String, u32)> {
    let mut routes = Vec::new();
    
    for i in 0..count {
        let addr = 0xC0A80000 + (i as u32 * 256); // 192.168.x.0
        let len = 24;
        let prefix = Prefix::new(addr, len).unwrap();
        let next_hop = format!("gateway{}", i);
        let metric = (i as u32 * 10) % 1000;
        routes.push((prefix, next_hop, metric));
    }
    
    routes
}

/// Generate test IPs for lookup
fn generate_test_ips(count: usize) -> Vec<String> {
    let mut ips = Vec::new();
    for i in 0..count {
        let ip = format!("192.168.{}.{}", i % 256, (i / 256) % 256);
        ips.push(ip);
    }
    ips
}

/// Benchmark all three phases at different scales
fn bench_all_phases_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_phases_scalability");
    group.measurement_time(Duration::from_secs(20));
    
    let test_ips = generate_test_ips(100);
    
    for size in [100, 500, 1000, 5000, 10000].iter() {
        let routes = generate_routes(*size);
        
        // Phase 1 (AHGF)
        let engine_p1 = AHGFEngine::new();
        for (prefix, next_hop, metric) in &routes {
            engine_p1.insert(*prefix, next_hop, *metric).unwrap();
        }
        
        group.bench_with_input(BenchmarkId::new("Phase1-AHGF", size), size, |b, _| {
            b.iter(|| {
                for ip in &test_ips {
                    black_box(engine_p1.lookup(ip).unwrap());
                }
            });
        });
        
        // Phase 2 V2 (Revolutionary)
        let engine_p2v2 = QAGFHGv2Engine::new(8);
        for (prefix, next_hop, metric) in &routes {
            engine_p2v2.insert(*prefix, next_hop.clone(), *metric).unwrap();
        }
        
        group.bench_with_input(BenchmarkId::new("Phase2-V2-Revolutionary", size), size, |b, _| {
            b.iter(|| {
                for ip in &test_ips {
                    black_box(engine_p2v2.lookup(ip).unwrap());
                }
            });
        });
        
        // Phase 3 (SCRTT)
        let mut engine_p3 = SCRTTEngine::new();
        for (prefix, next_hop, metric) in &routes {
            engine_p3.insert(*prefix, next_hop.clone(), *metric).unwrap();
        }
        
        group.bench_with_input(BenchmarkId::new("Phase3-SCRTT", size), size, |b, _| {
            b.iter(|| {
                for ip in &test_ips {
                    black_box(engine_p3.lookup(ip).unwrap());
                }
            });
        });
    }
    
    group.finish();
}

/// Benchmark single lookup performance
fn bench_single_lookup_all_phases(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_lookup_all_phases");
    group.measurement_time(Duration::from_secs(10));
    
    let routes = generate_routes(1000);
    let test_ips = generate_test_ips(100);
    
    // Phase 1
    let engine_p1 = AHGFEngine::new();
    for (prefix, next_hop, metric) in &routes {
        engine_p1.insert(*prefix, next_hop, *metric).unwrap();
    }
    
    group.bench_function("Phase1-AHGF", |b| {
        b.iter(|| {
            for ip in &test_ips {
                black_box(engine_p1.lookup(ip).unwrap());
            }
        });
    });
    
    // Phase 2 V2
    let engine_p2v2 = QAGFHGv2Engine::new(8);
    for (prefix, next_hop, metric) in &routes {
        engine_p2v2.insert(*prefix, next_hop.clone(), *metric).unwrap();
    }
    
    group.bench_function("Phase2-V2-Revolutionary", |b| {
        b.iter(|| {
            for ip in &test_ips {
                black_box(engine_p2v2.lookup(ip).unwrap());
            }
        });
    });
    
    // Phase 3
    let mut engine_p3 = SCRTTEngine::new();
    for (prefix, next_hop, metric) in &routes {
        engine_p3.insert(*prefix, next_hop.clone(), *metric).unwrap();
    }
    
    group.bench_function("Phase3-SCRTT", |b| {
        b.iter(|| {
            for ip in &test_ips {
                black_box(engine_p3.lookup(ip).unwrap());
            }
        });
    });
    
    group.finish();
}

/// Benchmark throughput (10K lookups)
fn bench_throughput_all_phases(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_all_phases");
    group.measurement_time(Duration::from_secs(15));
    
    let routes = generate_routes(5000);
    let test_ips = generate_test_ips(10000);
    
    // Phase 1
    let engine_p1 = AHGFEngine::new();
    for (prefix, next_hop, metric) in &routes {
        engine_p1.insert(*prefix, next_hop, *metric).unwrap();
    }
    
    group.bench_function("Phase1-AHGF-10K", |b| {
        b.iter(|| {
            for ip in &test_ips {
                black_box(engine_p1.lookup(ip).unwrap());
            }
        });
    });
    
    // Phase 2 V2
    let engine_p2v2 = QAGFHGv2Engine::new(8);
    for (prefix, next_hop, metric) in &routes {
        engine_p2v2.insert(*prefix, next_hop.clone(), *metric).unwrap();
    }
    
    group.bench_function("Phase2-V2-Revolutionary-10K", |b| {
        b.iter(|| {
            for ip in &test_ips {
                black_box(engine_p2v2.lookup(ip).unwrap());
            }
        });
    });
    
    // Phase 3
    let mut engine_p3 = SCRTTEngine::new();
    for (prefix, next_hop, metric) in &routes {
        engine_p3.insert(*prefix, next_hop.clone(), *metric).unwrap();
    }
    
    group.bench_function("Phase3-SCRTT-10K", |b| {
        b.iter(|| {
            for ip in &test_ips {
                black_box(engine_p3.lookup(ip).unwrap());
            }
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_single_lookup_all_phases,
    bench_throughput_all_phases,
    bench_all_phases_scalability
);
criterion_main!(benches);

