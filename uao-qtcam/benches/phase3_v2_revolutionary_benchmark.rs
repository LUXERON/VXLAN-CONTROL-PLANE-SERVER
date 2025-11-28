//! Phase 3 V2 Revolutionary Benchmark
//!
//! Tests the revolutionary Phase 3 V2 engine with all 10 postulates

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use uao_qtcam_unified::phase1::Prefix;
use uao_qtcam_unified::phase3::SCRTTv2Engine;
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

fn bench_phase3_v2_single_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase3_v2_single_lookup");
    
    for size in [100, 500, 1000, 5000, 10000].iter() {
        let routes = generate_routes(*size);
        let mut engine = SCRTTv2Engine::new().unwrap();
        
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

fn bench_phase3_v2_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase3_v2_throughput");
    
    for size in [100, 500, 1000, 5000, 10000].iter() {
        let routes = generate_routes(*size);
        let mut engine = SCRTTv2Engine::new().unwrap();
        
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

fn bench_phase3_v2_quantum_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase3_v2_quantum_cache");
    
    let routes = generate_routes(10000);
    let mut engine = SCRTTv2Engine::new().unwrap();
    
    // Insert routes with hot routes (metric < 100)
    for (i, (prefix, next_hop, _)) in routes.iter().enumerate() {
        let metric = if i < 100 { 50 } else { 500 }; // First 100 are hot
        engine.insert(*prefix, next_hop.clone(), metric).unwrap();
    }
    
    // Benchmark hot route lookups (should hit quantum cache)
    group.bench_function("hot_routes", |b| {
        b.iter(|| {
            let result = engine.lookup(black_box("0.0.1.42")).unwrap();
            black_box(result);
        });
    });
    
    // Benchmark cold route lookups
    group.bench_function("cold_routes", |b| {
        b.iter(|| {
            let result = engine.lookup(black_box("255.255.1.42")).unwrap();
            black_box(result);
        });
    });
    
    group.finish();
}

fn bench_phase3_v2_dimensional_folding(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase3_v2_dimensional_folding");
    
    for size in [100, 1000, 10000].iter() {
        let routes = generate_routes(*size);
        let mut engine = SCRTTv2Engine::new().unwrap();
        
        // Insert routes
        for (prefix, next_hop, metric) in routes.iter() {
            engine.insert(*prefix, next_hop.clone(), *metric).unwrap();
        }
        
        // Benchmark dimensional folding lookup
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let result = engine.lookup(black_box("192.168.1.42")).unwrap();
                black_box(result);
            });
        });
    }
    
    group.finish();
}

fn bench_phase3_v2_insert_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("phase3_v2_insert");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut engine = SCRTTv2Engine::new().unwrap();
                let routes = generate_routes(size);
                
                for (prefix, next_hop, metric) in routes.iter() {
                    engine.insert(*prefix, next_hop.clone(), *metric).unwrap();
                }
                
                black_box(engine);
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_phase3_v2_single_lookup,
    bench_phase3_v2_throughput,
    bench_phase3_v2_quantum_cache,
    bench_phase3_v2_dimensional_folding,
    bench_phase3_v2_insert_performance
);

criterion_main!(benches);

