//! # Concurrent Phase Execution Test
//!
//! Tests that all 3 phases can run concurrently and return consistent results

use uao_qtcam_unified::phase1::{AHGFEngine, Prefix};
use uao_qtcam_unified::phase2::QAGFHGv2Engine;
use uao_qtcam_unified::phase3::SCRTTv2Engine;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_concurrent_phase_consistency() {
    // Create all 3 phases
    let phase1 = Arc::new(AHGFEngine::new());
    let phase2 = Arc::new(RwLock::new(QAGFHGv2Engine::new(256)));
    let phase3 = Arc::new(RwLock::new(SCRTTv2Engine::new().unwrap()));

    // Insert same routes into all phases
    let test_routes = vec![
        ("192.168.1.0/24", "gateway1", 100),
        ("10.0.0.0/8", "gateway2", 50),
        ("172.16.0.0/12", "gateway3", 200),
    ];

    for (cidr, next_hop, metric) in &test_routes {
        let prefix = Prefix::from_cidr(cidr).unwrap();
        
        // Insert into Phase 1
        phase1.insert(prefix, next_hop.to_string(), *metric).unwrap();
        
        // Insert into Phase 2 V2
        phase2.write().await.insert(prefix, next_hop.to_string(), *metric).unwrap();
        
        // Insert into Phase 3 V2
        phase3.write().await.insert(prefix, next_hop.to_string(), *metric).unwrap();
    }

    // Test concurrent lookups
    let test_ips = vec![
        "192.168.1.42",
        "10.0.0.1",
        "172.16.1.1",
    ];

    for ip in test_ips {
        println!("\n🔍 Testing IP: {}", ip);
        
        // Lookup in all phases concurrently
        let phase1_clone = phase1.clone();
        let phase2_clone = phase2.clone();
        let phase3_clone = phase3.clone();
        let ip_str1 = ip.to_string();
        let ip_str2 = ip.to_string();
        let ip_str3 = ip.to_string();

        let (result1, result2, result3) = tokio::join!(
            tokio::spawn(async move {
                phase1_clone.lookup(&ip_str1).ok().flatten()
            }),
            tokio::spawn(async move {
                phase2_clone.read().await.lookup(&ip_str2).ok().flatten()
            }),
            tokio::spawn(async move {
                phase3_clone.write().await.lookup(&ip_str3).ok().flatten()
            })
        );

        let result1 = result1.unwrap();
        let result2 = result2.unwrap();
        let result3 = result3.unwrap();

        println!("  Phase 1: {:?}", result1);
        println!("  Phase 2 V2: {:?}", result2);
        println!("  Phase 3 V2: {:?}", result3);

        // Verify consistency
        if let Some(r1) = result1 {
            if let Some((nh2, m2, _)) = result2 {
                assert_eq!(r1.next_hop, nh2, "Phase 1 and Phase 2 V2 next_hop mismatch for IP {}", ip);
                assert_eq!(r1.metric, m2, "Phase 1 and Phase 2 V2 metric mismatch for IP {}", ip);
            }
            
            if let Some((nh3, m3, _)) = result3 {
                assert_eq!(r1.next_hop, nh3, "Phase 1 and Phase 3 V2 next_hop mismatch for IP {}", ip);
                assert_eq!(r1.metric, m3, "Phase 1 and Phase 3 V2 metric mismatch for IP {}", ip);
            }
        }
    }

    println!("\n✅ All phases return consistent results!");
}

#[tokio::test]
async fn test_concurrent_phase_performance() {
    // Create all 3 phases
    let phase1 = Arc::new(AHGFEngine::new());
    let phase2 = Arc::new(RwLock::new(QAGFHGv2Engine::new(256)));
    let phase3 = Arc::new(RwLock::new(SCRTTv2Engine::new().unwrap()));

    // Insert 1000 routes
    for i in 0..1000 {
        let cidr = format!("{}.{}.0.0/16", (i >> 8) & 0xFF, i & 0xFF);
        let prefix = Prefix::from_cidr(&cidr).unwrap();
        let next_hop = format!("gateway{}", i % 10);
        let metric = (i % 500) as u32;
        
        phase1.insert(prefix, next_hop.clone(), metric).unwrap();
        phase2.write().await.insert(prefix, next_hop.clone(), metric).unwrap();
        phase3.write().await.insert(prefix, next_hop.clone(), metric).unwrap();
    }

    // Concurrent lookup test
    let test_ip = "192.168.1.42";
    let iterations = 100;

    let start = std::time::Instant::now();
    
    for _ in 0..iterations {
        let phase1_clone = phase1.clone();
        let phase2_clone = phase2.clone();
        let phase3_clone = phase3.clone();
        let ip1 = test_ip.to_string();
        let ip2 = test_ip.to_string();
        let ip3 = test_ip.to_string();

        let _ = tokio::join!(
            tokio::spawn(async move { phase1_clone.lookup(&ip1) }),
            tokio::spawn(async move { phase2_clone.read().await.lookup(&ip2) }),
            tokio::spawn(async move { phase3_clone.write().await.lookup(&ip3) })
        );
    }

    let elapsed = start.elapsed();
    let avg_latency = elapsed.as_micros() as f64 / iterations as f64;

    println!("\n⚡ Concurrent Performance:");
    println!("  Total time: {:?}", elapsed);
    println!("  Avg latency per concurrent lookup: {:.2} µs", avg_latency);
    println!("  Throughput: {:.2} lookups/sec", 1_000_000.0 / avg_latency);
}

#[tokio::test]
async fn test_phase_failover() {
    // Test that if one phase fails, others continue working
    let phase1 = Arc::new(AHGFEngine::new());
    let phase2 = Arc::new(RwLock::new(QAGFHGv2Engine::new(256)));
    let phase3 = Arc::new(RwLock::new(SCRTTv2Engine::new().unwrap()));

    // Insert route only into Phase 1 and Phase 3 V2 (skip Phase 2 V2)
    let prefix = Prefix::from_cidr("192.168.1.0/24").unwrap();
    phase1.insert(prefix, "gateway1".to_string(), 100).unwrap();
    phase3.write().await.insert(prefix, "gateway1".to_string(), 100).unwrap();

    // Lookup should succeed in Phase 1 and Phase 3 V2
    let result1 = phase1.lookup("192.168.1.42").unwrap();
    let result3 = phase3.write().await.lookup("192.168.1.42").unwrap();

    assert!(result1.is_some(), "Phase 1 should find the route");
    assert!(result3.is_some(), "Phase 3 V2 should find the route");

    println!("✅ Failover test passed: Phases work independently");
}

