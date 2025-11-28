#!/bin/bash

# Test script for the example server API
# This script demonstrates how to trigger different log levels

BASE_URL="http://localhost:3000"

echo "=========================================="
echo "Testing Example Server API"
echo "=========================================="
echo ""

# Test 1: Create a user (INFO and DEBUG logs)
echo "1. Creating a user (triggers INFO and DEBUG logs)..."
curl -X POST "$BASE_URL/api/users" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "email": "alice@example.com"
  }' | jq .
echo ""
sleep 1

# Test 2: Create invalid user (triggers WARN logs)
echo "2. Creating user with invalid email (triggers WARN logs)..."
curl -X POST "$BASE_URL/api/users" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "bob",
    "email": "invalid-email"
  }' | jq .
echo ""
sleep 1

# Test 3: Get user by ID that doesn't exist (triggers WARN logs)
echo "3. Getting non-existent user (triggers WARN logs)..."
curl -X GET "$BASE_URL/api/users/non-existent-id" | jq .
echo ""
sleep 1

# Test 4: List users (INFO logs)
echo "4. Listing all users (triggers INFO logs)..."
curl -X GET "$BASE_URL/api/users" | jq .
echo ""
sleep 1

# Test 5: Create a product (INFO logs with structured fields)
echo "5. Creating a product (triggers INFO logs)..."
curl -X POST "$BASE_URL/api/products" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Laptop",
    "price": 999.99,
    "stock": 10
  }' | jq .
echo ""
sleep 1

# Test 6: Search products (DEBUG logs)
echo "6. Searching products (triggers DEBUG logs)..."
curl -X GET "$BASE_URL/api/products?search=Laptop&min_price=500&max_price=1500" | jq .
echo ""
sleep 1

# Test 7: Create invalid product (WARN logs)
echo "7. Creating product with invalid price (triggers WARN logs)..."
curl -X POST "$BASE_URL/api/products" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Invalid Product",
    "price": -10,
    "stock": 5
  }' | jq .
echo ""
sleep 1

# Test 8: Create an order (multiple log levels: DEBUG, INFO, WARN)
echo "8. Creating an order (triggers DEBUG, INFO, and potentially WARN logs)..."
curl -X POST "$BASE_URL/api/orders" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user-123",
    "product_ids": ["prod-1", "prod-2"],
    "total": 1999.98
  }' | jq .
echo ""
sleep 1

# Test 9: Create multiple orders to potentially trigger ERROR (10% failure rate)
echo "9. Creating multiple orders (may trigger ERROR logs - 10% failure rate)..."
for i in {1..5}; do
  echo "  Order attempt $i..."
  curl -X POST "$BASE_URL/api/orders" \
    -H "Content-Type: application/json" \
    -d '{
      "user_id": "user-456",
      "product_ids": ["prod-3"],
      "total": 99.99
    }' | jq -c .
  sleep 0.5
done
echo ""

# Test 10: Invalid order (WARN logs)
echo "10. Creating order with empty user_id (triggers WARN logs)..."
curl -X POST "$BASE_URL/api/orders" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "",
    "product_ids": ["prod-1"],
    "total": 100.00
  }' | jq .
echo ""

echo "=========================================="
echo "API Testing Complete!"
echo "=========================================="
echo ""
echo "Check the tracing UI at: http://localhost:3000/tracing"
echo "Background heartbeat task logs TRACE events every 10 seconds"
echo ""
