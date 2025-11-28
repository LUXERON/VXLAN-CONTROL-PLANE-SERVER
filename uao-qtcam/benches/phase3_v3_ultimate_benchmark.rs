//! Phase 3 V3 ULTIMATE Benchmark
//!
//! Tests the ultimate Phase 3 V3 engine with ALL 10 postulates

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use uao_qtcam_unified::phase1::Prefix;
use uao_qtcam_unified::phase3::SCRTTv3Engine;
use std::net::Ipv4Addr;

fn generate_routes(count: usize) -> Vec<(Prefix, String, u32)> {
    let mut routes = Vec::new();
    for i in 0..count {
        let ip = Ipv4Addr::new(
            ((i >> 16) & 0xFF) as u8,
            ((i >> 8) & 0xFF) as u8,
            (i & 0xFF) as u8,
            0,
        );
        let prefix = Prefix::from_cidr(&format!("{}/24", ip)).unwrap();
        let next_hop = format!("gateway{}", i % 100);
        let metric = (i % 1000) as u32;
        routes.push((prefix, next_hop, metric));
    }
    routes
}

fn bench_phase3_v3_single_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase3_v3_single_lookup");
    
    for size in [100, 500, 1000, 5000, 10000].iter() {
        let routes = generate_routes(*size);
        let mut engine = SCRTTv3Engine::new().unwrap();
        
        // Insert routes
        for (prefix, next_hop, metric) in routes.iter() {
            engine.insert(*prefix, next_hop.clone(), *metric).unwrap();
        }
        
        // Benchmark lookup
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let result = engine.lookup(black_box("192.168.1.42")).unwrap();
                black_box(result);
            });
        });
    }
    
    group.finish();
}

fn bench_phase3_v3_4_tier_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase3_v3_4_tier_cache");
    
    let routes = generate_routes(10000);
    let mut engine = SCRTTv3Engine::new().unwrap();
    
    // Insert routes with different metrics for different cache tiers
    for (i, (prefix, next_hop, _)) in routes.iter().enumerate() {
        let metric = if i < 100 {
            50 // Tier 1: Quantum cache
        } else if i < 1000 {
            150 // Tier 2: SIMD
        } else if i < 5000 {
            250 // Tier 3: Tensor
        } else {
            350 // Tier 4: Dimensional folding
        };
        engine.insert(*prefix, next_hop.clone(), metric).unwrap();
    }
    
    // Benchmark each tier
    group.bench_function("tier1_quantum_cache", |b| {
        b.iter(|| {
            let result = engine.lookup(black_box("0.0.1.42")).unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("tier2_simd", |b| {
        b.iter(|| {
            let result = engine.lookup(black_box("0.3.1.42")).unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("tier3_tensor", |b| {
        b.iter(|| {
            let result = engine.lookup(black_box("19.136.1.42")).unwrap();
            black_box(result);
        });
    });
    
    group.bench_function("tier4_dimensional_folding", |b| {
        b.iter(|| {
            let result = engine.lookup(black_box("39.16.1.42")).unwrap();
            black_box(result);
        });
    });
    
    group.finish();
}

fn bench_phase3_v3_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase3_v3_throughput");
    
    for size in [100, 1000, 10000].iter() {
        let routes = generate_routes(*size);
        let mut engine = SCRTTv3Engine::new().unwrap();
        
        // Insert routes
        for (prefix, next_hop, metric) in routes.iter() {
            engine.insert(*prefix, next_hop.clone(), *metric).unwrap();
        }
        
        // Generate test IPs
        let test_ips: Vec<String> = (0..1000)
            .map(|i| format!("192.168.{}.{}", (i >> 8) & 0xFF, i & 0xFF))
            .collect();
        
        // Benchmark throughput
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                for ip in &test_ips {
                    let result = engine.lookup(black_box(ip)).unwrap();
                    black_box(result);
                }
            });
        });
    }
    
    group.finish();
}

fn bench_phase3_v3_compression_ratio(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase3_v3_compression");
    
    for size in [1000, 5000, 10000].iter() {
        let routes = generate_routes(*size);
        let mut engine = SCRTTv3Engine::new().unwrap();
        
        // Insert routes
        for (prefix, next_hop, metric) in routes.iter() {
            engine.insert(*prefix, next_hop.clone(), *metric).unwrap();
        }
        
        let stats = engine.stats();
        println!("Size: {}, Compression ratio: {}", size, stats.compression_ratio);
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_phase3_v3_single_lookup,
    bench_phase3_v3_4_tier_cache,
    bench_phase3_v3_throughput,
    bench_phase3_v3_compression_ratio
);

criterion_main!(benches);

