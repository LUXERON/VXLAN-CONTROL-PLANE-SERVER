//! Performance benchmarks for UAO-QTCAM
//! 
//! Comprehensive benchmarks comparing Phase 1 (AHGF) vs Phase 2 (QAGFHG)
//! Target: Validate 10 ns latency and 1,000x speedup vs hardware TCAM

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use uao_qtcam_unified::unified::{TCAMEngine, Route, PhaseStrategy};
use uao_qtcam_unified::phase1::Prefix;
use std::time::Duration;

/// Generate realistic routing table with various prefix lengths
fn generate_routing_table(size: usize) -> Vec<Route> {
    let mut routes = Vec::with_capacity(size);

    // Common prefix lengths in real routing tables
    let prefix_lengths = [8, 16, 24, 32];

    for i in 0..size {
        let prefix_len = prefix_lengths[i % prefix_lengths.len()];
        let addr = ((i as u32) << (32 - prefix_len)) & (!0u32 << (32 - prefix_len));

        let prefix = Prefix::new(addr, prefix_len).unwrap();
        let next_hop = format!("gateway_{}", i % 10);
        let metric = (i % 100) as u32;

        routes.push(Route::new(prefix, next_hop, metric));
    }

    routes
}

/// Generate realistic IP addresses for lookup testing
fn generate_test_ips(count: usize) -> Vec<String> {
    let mut ips = Vec::with_capacity(count);
    
    for i in 0..count {
        let a = (i % 256) as u8;
        let b = ((i / 256) % 256) as u8;
        let c = ((i / 65536) % 256) as u8;
        let d = ((i / 16777216) % 256) as u8;
        
        ips.push(format!("{}.{}.{}.{}", a, b, c, d));
    }
    
    ips
}

/// Benchmark: Single lookup latency (Phase 1 vs Phase 2)
fn bench_single_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_lookup_latency");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(1000);
    
    // Setup routing table
    let routes = generate_routing_table(1000);
    let test_ips = generate_test_ips(100);
    
    // Phase 1 benchmark
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine_p1 = rt.block_on(async {
        let engine = TCAMEngine::with_strategy(PhaseStrategy::Phase1Only).unwrap();
        for route in &routes {
            engine.insert(route.clone()).await.unwrap();
        }
        engine
    });
    
    group.bench_function("Phase1-AHGF", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                for ip in &test_ips {
                    black_box(engine_p1.lookup(ip).await.unwrap());
                }
            });
        });
    });
    
    // Phase 2 benchmark
    let engine_p2 = rt.block_on(async {
        let engine = TCAMEngine::with_strategy(PhaseStrategy::Phase2Only).unwrap();
        for route in &routes {
            engine.insert(route.clone()).await.unwrap();
        }
        engine
    });
    
    group.bench_function("Phase2-QAGFHG", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                for ip in &test_ips {
                    black_box(engine_p2.lookup(ip).await.unwrap());
                }
            });
        });
    });
    
    group.finish();
}

/// Benchmark: Throughput (lookups per second)
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(100);
    
    let routes = generate_routing_table(5000);
    let test_ips = generate_test_ips(10000);
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Phase 1 throughput
    let engine_p1 = rt.block_on(async {
        let engine = TCAMEngine::with_strategy(PhaseStrategy::Phase1Only).unwrap();
        for route in &routes {
            engine.insert(route.clone()).await.unwrap();
        }
        engine
    });
    
    group.bench_function("Phase1-AHGF-10K-lookups", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                for ip in &test_ips {
                    black_box(engine_p1.lookup(ip).await.unwrap());
                }
            });
        });
    });
    
    // Phase 2 throughput
    let engine_p2 = rt.block_on(async {
        let engine = TCAMEngine::with_strategy(PhaseStrategy::Phase2Only).unwrap();
        for route in &routes {
            engine.insert(route.clone()).await.unwrap();
        }
        engine
    });
    
    group.bench_function("Phase2-QAGFHG-10K-lookups", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                for ip in &test_ips {
                    black_box(engine_p2.lookup(ip).await.unwrap());
                }
            });
        });
    });
    
    group.finish();
}

/// Benchmark: Scalability (varying routing table sizes)
fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    let test_ips = generate_test_ips(100);
    let rt = tokio::runtime::Runtime::new().unwrap();

    for size in [100, 500, 1000, 5000, 10000].iter() {
        let routes = generate_routing_table(*size);

        // Phase 1
        let engine_p1 = rt.block_on(async {
            let engine = TCAMEngine::with_strategy(PhaseStrategy::Phase1Only).unwrap();
            for route in &routes {
                engine.insert(route.clone()).await.unwrap();
            }
            engine
        });

        group.bench_with_input(
            BenchmarkId::new("Phase1", size),
            size,
            |b, _| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                b.iter(|| {
                    rt.block_on(async {
                        for ip in &test_ips {
                            black_box(engine_p1.lookup(ip).await.unwrap());
                        }
                    });
                });
            },
        );

        // Phase 2
        let engine_p2 = rt.block_on(async {
            let engine = TCAMEngine::with_strategy(PhaseStrategy::Phase2Only).unwrap();
            for route in &routes {
                engine.insert(route.clone()).await.unwrap();
            }
            engine
        });

        group.bench_with_input(
            BenchmarkId::new("Phase2", size),
            size,
            |b, _| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                b.iter(|| {
                    rt.block_on(async {
                        for ip in &test_ips {
                            black_box(engine_p2.lookup(ip).await.unwrap());
                        }
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Insert performance
fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert_performance");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    let routes = generate_routing_table(1000);
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Phase 1 insert
    group.bench_function("Phase1-AHGF-1K-inserts", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let engine = TCAMEngine::with_strategy(PhaseStrategy::Phase1Only).unwrap();
                for route in &routes {
                    black_box(engine.insert(route.clone()).await.unwrap());
                }
            });
        });
    });

    // Phase 2 insert
    group.bench_function("Phase2-QAGFHG-1K-inserts", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let engine = TCAMEngine::with_strategy(PhaseStrategy::Phase2Only).unwrap();
                for route in &routes {
                    black_box(engine.insert(route.clone()).await.unwrap());
                }
            });
        });
    });

    group.finish();
}

/// Benchmark: Mixed workload (inserts + lookups)
fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(50);

    let initial_routes = generate_routing_table(1000);
    let new_routes = generate_routing_table(100);
    let test_ips = generate_test_ips(500);
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Phase 1 mixed
    group.bench_function("Phase1-AHGF-mixed", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let engine = TCAMEngine::with_strategy(PhaseStrategy::Phase1Only).unwrap();

                // Initial inserts
                for route in &initial_routes {
                    engine.insert(route.clone()).await.unwrap();
                }

                // Mixed operations
                for i in 0..100 {
                    // 5 lookups
                    for j in 0..5 {
                        black_box(engine.lookup(&test_ips[i * 5 + j]).await.unwrap());
                    }
                    // 1 insert
                    black_box(engine.insert(new_routes[i].clone()).await.unwrap());
                }
            });
        });
    });

    // Phase 2 mixed
    group.bench_function("Phase2-QAGFHG-mixed", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let engine = TCAMEngine::with_strategy(PhaseStrategy::Phase2Only).unwrap();

                // Initial inserts
                for route in &initial_routes {
                    engine.insert(route.clone()).await.unwrap();
                }

                // Mixed operations
                for i in 0..100 {
                    // 5 lookups
                    for j in 0..5 {
                        black_box(engine.lookup(&test_ips[i * 5 + j]).await.unwrap());
                    }
                    // 1 insert
                    black_box(engine.insert(new_routes[i].clone()).await.unwrap());
                }
            });
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_lookup,
    bench_throughput,
    bench_scalability,
    bench_insert,
    bench_mixed_workload
);
criterion_main!(benches);

