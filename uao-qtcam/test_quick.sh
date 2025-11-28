#!/bin/bash

echo "=== TEST 1: Health Check ==="
curl -s http://localhost:8080/health | jq '.'
echo ""

echo "=== TEST 2: API Info ==="
curl -s http://localhost:8080/ | jq '.'
echo ""

echo "=== TEST 3: Lookup 192.168.1.42 (should match 192.168.1.0/24) ==="
curl -s -X POST http://localhost:8080/lookup \
  -H "Content-Type: application/json" \
  -d '{"ip": "192.168.1.42"}' | jq '.'
echo ""

echo "=== TEST 4: Lookup 192.168.2.42 (should match 192.168.0.0/16) ==="
curl -s -X POST http://localhost:8080/lookup \
  -H "Content-Type: application/json" \
  -d '{"ip": "192.168.2.42"}' | jq '.'
echo ""

echo "=== TEST 5: Lookup 10.1.2.3 (should match 10.0.0.0/8) ==="
curl -s -X POST http://localhost:8080/lookup \
  -H "Content-Type: application/json" \
  -d '{"ip": "10.1.2.3"}' | jq '.'
echo ""

echo "=== TEST 6: Lookup 8.8.8.8 (should match default route) ==="
curl -s -X POST http://localhost:8080/lookup \
  -H "Content-Type: application/json" \
  -d '{"ip": "8.8.8.8"}' | jq '.'
echo ""

echo "=== TEST 7: Insert New Route ==="
curl -s -X POST http://localhost:8080/insert \
  -H "Content-Type: application/json" \
  -d '{"prefix": "203.0.113.0/24", "next_hop": "test_gateway", "metric": 42}' | jq '.'
echo ""

echo "=== TEST 8: Lookup New Route ==="
curl -s -X POST http://localhost:8080/lookup \
  -H "Content-Type: application/json" \
  -d '{"ip": "203.0.113.100"}' | jq '.'
echo ""

echo "=== TEST 9: Statistics ==="
curl -s http://localhost:8080/stats | jq '.'
echo ""

echo "✅ All tests completed!"

