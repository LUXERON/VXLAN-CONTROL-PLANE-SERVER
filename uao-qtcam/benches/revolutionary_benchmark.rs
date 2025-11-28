//! Revolutionary Benchmark: Phase 2 V2 vs Phase 1
//!
//! This benchmark tests the REVOLUTIONARY Phase 2 V2 engine against Phase 1
//! to prove exponential speedup through groundbreaking postulates.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use uao_qtcam_unified::phase1::{AHGFEngine, Prefix};
use uao_qtcam_unified::phase2::QAGFHGv2Engine;
use std::time::Duration;

/// Generate test routes
fn generate_routes(count: usize) -> Vec<(Prefix, String, u32)> {
    let mut routes = Vec::new();
    
    // Generate diverse routing prefixes
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

/// Benchmark Phase 1 (AHGF) single lookup
fn bench_phase1_single_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_lookup");
    group.measurement_time(Duration::from_secs(10));
    
    // Setup Phase 1 engine
    let engine = AHGFEngine::new();
    let routes = generate_routes(1000);
    for (prefix, next_hop, metric) in routes {
        engine.insert(prefix, &next_hop, metric).unwrap();
    }
    
    let test_ips = generate_test_ips(100);
    
    group.bench_function("Phase1-AHGF", |b| {
        b.iter(|| {
            for ip in &test_ips {
                black_box(engine.lookup(ip).unwrap());
            }
        });
    });
    
    group.finish();
}

/// Benchmark Phase 2 V2 (Revolutionary) single lookup
fn bench_phase2_v2_single_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_lookup");
    group.measurement_time(Duration::from_secs(10));
    
    // Setup Phase 2 V2 engine
    let engine = QAGFHGv2Engine::new(8);
    let routes = generate_routes(1000);
    for (prefix, next_hop, metric) in routes {
        engine.insert(prefix, next_hop, metric).unwrap();
    }
    
    let test_ips = generate_test_ips(100);
    
    group.bench_function("Phase2-V2-Revolutionary", |b| {
        b.iter(|| {
            for ip in &test_ips {
                black_box(engine.lookup(ip).unwrap());
            }
        });
    });
    
    group.finish();
}

/// Benchmark throughput comparison
fn bench_throughput_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.measurement_time(Duration::from_secs(15));
    
    let routes = generate_routes(5000);
    let test_ips = generate_test_ips(10000);
    
    // Phase 1
    let engine_p1 = AHGFEngine::new();
    for (prefix, next_hop, metric) in &routes {
        engine_p1.insert(*prefix, next_hop, *metric).unwrap();
    }
    
    group.bench_function("Phase1-AHGF-10K-lookups", |b| {
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
    
    group.bench_function("Phase2-V2-Revolutionary-10K-lookups", |b| {
        b.iter(|| {
            for ip in &test_ips {
                black_box(engine_p2v2.lookup(ip).unwrap());
            }
        });
    });
    
    group.finish();
}

/// Benchmark scalability
fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    group.measurement_time(Duration::from_secs(20));
    
    let test_ips = generate_test_ips(100);
    
    for size in [100, 500, 1000, 5000, 10000].iter() {
        let routes = generate_routes(*size);
        
        // Phase 1
        let engine_p1 = AHGFEngine::new();
        for (prefix, next_hop, metric) in &routes {
            engine_p1.insert(*prefix, next_hop, *metric).unwrap();
        }
        
        group.bench_with_input(BenchmarkId::new("Phase1", size), size, |b, _| {
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
        
        group.bench_with_input(BenchmarkId::new("Phase2-V2", size), size, |b, _| {
            b.iter(|| {
                for ip in &test_ips {
                    black_box(engine_p2v2.lookup(ip).unwrap());
                }
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_phase1_single_lookup,
    bench_phase2_v2_single_lookup,
    bench_throughput_comparison,
    bench_scalability
);
criterion_main!(benches);

