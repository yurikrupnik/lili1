#!/usr/bin/env nu

echo "Starting zerg-api in background..."
let process = (cargo run &)

sleep 2sec

echo "Testing API endpoints..."
try {
    echo "1. Testing root endpoint:"
    http get http://localhost:8080/ | print
    
    echo "\n2. Testing tasks endpoint:"
    http get http://localhost:8080/tasks | print
    
    echo "\n3. Testing metrics endpoint:"
    http get http://localhost:8080/metrics | print
    
    echo "\n4. Making multiple requests to generate metrics:"
    for i in 1..5 {
        http get http://localhost:8080/tasks | ignore
        http get http://localhost:8080/ | ignore
    }
    
    echo "\n5. Final metrics check:"
    http get http://localhost:8080/metrics | print
    
} catch { |e|
    echo $"Error: ($e)"
}

echo "\nStopping zerg-api..."
ps | where name =~ zerg_api | get pid | each { |pid| kill $pid }