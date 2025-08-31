#!/usr/bin/env nu

print "Starting Zerg Full Stack App..."

let commands = [
    { name: "LangGraph Agent", dir: "ts-agent", cmd: "bun run dev:server" },
    { name: "Rust API", dir: "api", cmd: "cargo run" },
    { name: "SolidJS UI", dir: "ui", cmd: "bun run dev" }
]

print "Starting services:"
for service in $commands {
    print $"  - ($service.name) in ($service.dir)/"
}

print ""
print "Services will be available at:"
print "  - LangGraph Agent: http://localhost:3001"
print "  - Rust API: http://localhost:8080"
print "  - SolidJS UI: http://localhost:4173"
print ""

# Start all services in parallel using background jobs
for service in $commands {
    print $"Starting ($service.name)..."
    cd $service.dir
    ^($service.cmd | split row " " | first) ...($service.cmd | split row " " | skip 1) &
    cd ..
}

print ""
print "All services started! Press Ctrl+C to stop all services."
print ""

# Keep the script running
loop {
    sleep 1sec
}