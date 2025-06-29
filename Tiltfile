# Tiltfile for local development
# This file configures Tilt to build and deploy the Starcraft APIs locally

# Allow k8s contexts for local development
allow_k8s_contexts(['kind-kind', 'docker-desktop', 'minikube'])

# Load environment variables
load('ext://dotenv', 'dotenv')
dotenv('.env')

# Configuration - simplified for now
skip_dependencies = False

# Set namespace
k8s_namespace('default')

# Deploy dependencies (Redis, PostgreSQL, MongoDB) if not skipped
if not skip_dependencies:
    print("🗄️  Deploying development dependencies...")
    
    # PostgreSQL
    k8s_yaml('dev/postgresql.yaml')
    k8s_resource('postgresql', port_forwards='5432:5432')
    
    # Redis
    k8s_yaml('dev/redis.yaml')
    k8s_resource('redis', port_forwards='6379:6379')
    
    # MongoDB
    k8s_yaml('dev/mongodb.yaml')
    k8s_resource('mongodb', port_forwards='27017:27017')

# Build and deploy Zerg API
print("🚀 Building Zerg API...")
docker_build(
    'zerg-api:dev',
    context='.',
    dockerfile='apps/zerg/api/Dockerfile',
    # Live reload for Rust files
    live_update=[
        sync('apps/zerg/api/src', '/app/apps/zerg/api/src'),
        sync('libs/rust/services/src', '/app/libs/rust/services/src'),
        run('cargo build --release -p zerg_api', trigger=['apps/zerg/api/src', 'libs/rust/services/src'])
    ]
)

# Deploy Zerg API
k8s_yaml(kustomize('manifests/kustomize/overlays/dev'))
k8s_resource(
    'dev-zerg-api-zerg-api',
    port_forwards='3001:8080',
    resource_deps=['postgresql', 'redis', 'mongodb']
)

# Build and deploy Terran API
print("🚀 Building Terran API...")
docker_build(
    'terran-api:dev',
    context='.',
    dockerfile='apps/terran/api/Dockerfile',
    # Live reload for Rust files
    live_update=[
        sync('apps/terran/api/src', '/app/apps/terran/api/src'),
        sync('libs/rust/services/src', '/app/libs/rust/services/src'),
        run('cargo build --release -p terran_api', trigger=['apps/terran/api/src', 'libs/rust/services/src'])
    ]
)

# Deploy Terran API
k8s_yaml(kustomize('manifests/kustomize/terran-api/overlays/dev'))
k8s_resource(
    'dev-terran-api-terran-api',
    port_forwards='3002:8080',
    resource_deps=['postgresql', 'redis', 'mongodb']
)

# Local resource for running cargo commands
local_resource(
    'cargo-check',
    'cargo check --workspace',
    deps=['apps/', 'libs/'],
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL
)

local_resource(
    'cargo-test',
    'cargo test --workspace',
    deps=['apps/', 'libs/'],
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL
)

local_resource(
    'cargo-fmt',
    'cargo fmt --all',
    deps=['apps/', 'libs/'],
    auto_init=False,
    trigger_mode=TRIGGER_MODE_MANUAL
)

# Print helpful information
print("""
🎮 Starcraft APIs Development Environment

Services will be available at:
- Zerg API:    http://localhost:3001
- Terran API:  http://localhost:3002
- PostgreSQL:  localhost:5432
- Redis:       localhost:6379
- MongoDB:     localhost:27017

Manual commands:
- cargo-check: Run cargo check on the workspace
- cargo-test:  Run tests
- cargo-fmt:   Format code

To skip dependencies: tilt up -- --skip-dependencies=true
""")