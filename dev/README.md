# Local Development with Tilt

This directory contains configuration for local development using Tilt, which provides hot reloading and integrated Kubernetes development.

## Prerequisites

1. **Install Tilt**: [https://tilt.dev/](https://tilt.dev/)
   ```bash
   # macOS
   brew install tilt-dev/tap/tilt
   
   # Linux
   curl -fsSL https://raw.githubusercontent.com/tilt-dev/tilt/master/scripts/install.sh | bash
   ```

2. **Kubernetes Cluster**: One of the following:
   - [Kind](https://kind.sigs.k8s.io/) (recommended)
   - [Docker Desktop](https://www.docker.com/products/docker-desktop/) with Kubernetes enabled
   - [Minikube](https://minikube.sigs.k8s.io/)

3. **Docker**: For building container images

## Quick Start

### 1. Setup Kind cluster (recommended)
```bash
# Install kind
brew install kind  # macOS
# or
go install sigs.k8s.io/kind@latest  # Any platform

# Create cluster
kind create cluster --name starcraft-dev

# Verify context
kubectl config current-context  # Should show: kind-starcraft-dev
```

### 2. Start development environment
```bash
# From project root
tilt up

# Open Tilt UI (automatically opens, or visit http://localhost:10350)
```

### 3. Access services
- **Tilt UI**: http://localhost:10350
- **Zerg API**: http://localhost:3001
- **Terran API**: http://localhost:3002
- **PostgreSQL**: localhost:5432 (user: developer, password: password, db: starcraft)
- **Redis**: localhost:6379
- **MongoDB**: localhost:27017 (user: developer, password: password)

## Development Workflow

### Hot Reloading
Tilt watches your Rust source files and automatically:
1. Rebuilds the application when code changes
2. Updates the running container
3. Restarts the service if needed

Watched directories:
- `apps/zerg/api/src/`
- `apps/terran/api/src/`
- `libs/rust/services/src/`

### Manual Commands
Available in Tilt UI or via CLI:

```bash
# Run cargo check
tilt trigger cargo-check

# Run tests
tilt trigger cargo-test

# Format code
tilt trigger cargo-fmt
```

### Development Commands

```bash
# Start with custom namespace
tilt up -- --namespace=development

# Skip dependencies (if you have them running elsewhere)
tilt up -- --skip-dependencies=true

# Stop everything
tilt down

# View logs
tilt logs zerg-api
tilt logs terran-api
```

## Configuration

### Environment Variables
Create a `.env` file in the project root for custom configuration:
```bash
# .env
DATABASE_URL=postgresql://developer:password@localhost:5432/starcraft
REDIS_HOST=redis://localhost:6379
MONGO_URI=mongodb://developer:password@localhost:27017/starcraft
RUST_ENV=development
```

### Database Setup
The PostgreSQL instance creates a database named `starcraft` with user `developer`.

To connect and run migrations:
```bash
# Connect to PostgreSQL
psql -h localhost -U developer -d starcraft

# Run migrations (if you have them)
cargo install sqlx-cli
sqlx migrate run --database-url postgresql://developer:password@localhost:5432/starcraft
```

## Troubleshooting

### Common Issues

1. **Port conflicts**: Change port forwards in `Tiltfile` if ports are in use
2. **Build failures**: Check that Rust toolchain is installed and up to date
3. **Kubernetes context**: Ensure you're using a supported context (kind, docker-desktop, minikube)

### Logs and Debugging
```bash
# View all logs
tilt logs

# View specific service logs
tilt logs zerg-api
tilt logs postgresql

# Restart a service
tilt trigger zerg-api

# Force rebuild
tilt trigger zerg-api --force
```

### Clean Up
```bash
# Stop Tilt
tilt down

# Delete kind cluster
kind delete cluster --name starcraft-dev

# Clean Docker images
docker image prune -f
```

## Performance Tips

1. **Use Kind**: Faster than Docker Desktop for Kubernetes
2. **Exclude target/**: Add `target/` to `.tiltignore` if not already excluded
3. **Incremental builds**: Tilt's live_update feature enables fast incremental rebuilds
4. **Resource limits**: Adjust memory/CPU limits in dev YAML files based on your machine

## IDE Integration

### VS Code
Install these extensions for better development experience:
- Rust Analyzer
- Kubernetes
- Tilt (syntax highlighting for Tiltfile)

### Environment Variables in IDE
Point your IDE to use the same environment variables:
```bash
DATABASE_URL=postgresql://developer:password@localhost:5432/starcraft
REDIS_HOST=redis://localhost:6379
MONGO_URI=mongodb://developer:password@localhost:27017/starcraft
RUST_ENV=development
```