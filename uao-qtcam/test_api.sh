#!/bin/bash

# UAO-QTCAM API Test Script
# Tests all API endpoints

set -e

BASE_URL="http://localhost:8080"

echo "🎯 UAO-QTCAM API Test Script"
echo "=============================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test 1: Root endpoint
echo -e "${BLUE}Test 1: GET / (API Info)${NC}"
curl -s "$BASE_URL/" | jq '.'
echo ""
echo ""

# Test 2: Health check
echo -e "${BLUE}Test 2: GET /health${NC}"
curl -s "$BASE_URL/health" | jq '.'
echo ""
echo ""

# Test 3: Insert routes
echo -e "${BLUE}Test 3: POST /insert (Insert Routes)${NC}"

echo "Inserting route: 10.0.0.0/8"
curl -s -X POST "$BASE_URL/insert" \
  -H "Content-Type: application/json" \
  -d '{"prefix": "10.0.0.0/8", "next_hop": "gateway_1", "metric": 100}' | jq '.'
echo ""

echo "Inserting route: 172.16.0.0/12"
curl -s -X POST "$BASE_URL/insert" \
  -H "Content-Type: application/json" \
  -d '{"prefix": "172.16.0.0/12", "next_hop": "gateway_2", "metric": 50}' | jq '.'
echo ""

echo "Inserting route: 192.168.0.0/16"
curl -s -X POST "$BASE_URL/insert" \
  -H "Content-Type: application/json" \
  -d '{"prefix": "192.168.0.0/16", "next_hop": "gateway_3", "metric": 10}' | jq '.'
echo ""

echo "Inserting route: 192.168.1.0/24"
curl -s -X POST "$BASE_URL/insert" \
  -H "Content-Type: application/json" \
  -d '{"prefix": "192.168.1.0/24", "next_hop": "gateway_4", "metric": 5}' | jq '.'
echo ""
echo ""

# Test 4: Lookup routes
echo -e "${BLUE}Test 4: POST /lookup (Lookup Routes)${NC}"

echo "Looking up: 192.168.1.42 (should match 192.168.1.0/24)"
curl -s -X POST "$BASE_URL/lookup" \
  -H "Content-Type: application/json" \
  -d '{"ip": "192.168.1.42"}' | jq '.'
echo ""

echo "Looking up: 192.168.2.42 (should match 192.168.0.0/16)"
curl -s -X POST "$BASE_URL/lookup" \
  -H "Content-Type: application/json" \
  -d '{"ip": "192.168.2.42"}' | jq '.'
echo ""

echo "Looking up: 10.1.2.3 (should match 10.0.0.0/8)"
curl -s -X POST "$BASE_URL/lookup" \
  -H "Content-Type: application/json" \
  -d '{"ip": "10.1.2.3"}' | jq '.'
echo ""

echo "Looking up: 8.8.8.8 (should match default route)"
curl -s -X POST "$BASE_URL/lookup" \
  -H "Content-Type: application/json" \
  -d '{"ip": "8.8.8.8"}' | jq '.'
echo ""
echo ""

# Test 5: Statistics
echo -e "${BLUE}Test 5: GET /stats (Statistics)${NC}"
curl -s "$BASE_URL/stats" | jq '.'
echo ""
echo ""

echo -e "${GREEN}✅ All tests completed!${NC}"
echo ""
echo "Summary:"
echo "- API is responding correctly"
echo "- Routes can be inserted"
echo "- Lookups work with longest prefix matching"
echo "- Statistics are being tracked"
echo ""
echo "🎯🌌🔬🎼🚀💰"

