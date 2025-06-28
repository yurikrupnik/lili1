# Zerg Operator

A Kubernetes operator for managing dependencies, GitOps workflows, and CI/CD pipelines.

## Features

- **Dependency Management**: Install and manage Kubernetes dependencies like:
  - External Secrets Operator
  - Crossplane
  - Loki Stack
  - Prometheus
  - Cert-Manager
  - And more...

- **GitOps Integration**: 
  - FluxCD support
  - ArgoCD support
  - Automated sync policies

- **CI/CD Pipelines**:
  - Tekton pipelines
  - Argo Workflows
  - Git webhook triggers
  - Scheduled pipelines

## Quick Start

### Prerequisites

- Kubernetes cluster (1.20+)
- kubectl configured
- Helm 3.x
- Docker (for building)

### Installation

1. Clone the repository and build the operator:
```bash
make build
make docker-build
```

2. Deploy the operator:
```bash
make deploy
```

3. Apply an example configuration:
```bash
kubectl apply -f examples/basic-setup.yaml
```

### Basic Usage

Create a `DependencyManager` resource to define your platform setup:

```yaml
apiVersion: zerg.io/v1
kind: DependencyManager
metadata:
  name: my-platform
  namespace: zerg-system
spec:
  dependencies:
    - name: external-secrets
      type: operator
      enabled: true
      source:
        repo: https://charts.external-secrets.io
        chart: external-secrets
      version: 0.9.0
      namespace: external-secrets-system
      
    - name: crossplane
      type: operator
      enabled: true
      source:
        repo: https://charts.crossplane.io/stable
        chart: crossplane
      version: 1.14.0
      namespace: crossplane-system
      
  gitops:
    provider: flux
    repository: https://github.com/your-org/gitops-repo
    branch: main
    path: clusters/production
    sync_policy:
      automated: true
      prune: true
      self_heal: true
```

## Configuration

### Supported Dependencies

The operator supports several dependency types:

- **helm**: Install Helm charts
- **kustomize**: Apply Kustomize configurations  
- **yaml**: Apply raw YAML manifests
- **operator**: Install Kubernetes operators

### Built-in Templates

The operator includes templates for common dependencies:

- `external-secrets`: External Secrets Operator
- `crossplane`: Crossplane for infrastructure as code
- `loki`: Loki logging stack
- `prometheus`: Prometheus monitoring
- `cert-manager`: Certificate management

### GitOps Providers

- **FluxCD**: GitOps with Flux v2
- **ArgoCD**: GitOps with ArgoCD

### CI/CD Providers

- **Tekton**: Cloud-native CI/CD
- **Argo Workflows**: Workflow engine

## Development

### Building

```bash
# Format and lint
make fmt
make lint

# Run tests
make test

# Build binary
make build

# Build Docker image
make docker-build
```

### Running Locally

```bash
# Run the operator locally (requires kubectl access)
make run
```

### Debugging

```bash
# View operator logs
make logs

# Check status
make status

# Port forward metrics
make port-forward-metrics
```

## Architecture

The operator consists of several key components:

- **Controller**: Main reconciliation loop
- **Dependency Installer**: Handles installation of various dependency types
- **GitOps Manager**: Manages GitOps configurations
- **Config Manager**: Handles operator configuration

## Examples

See the `examples/` directory for complete configuration examples:

- `basic-setup.yaml`: Basic platform setup with common dependencies
- `argocd-setup.yaml`: Setup using ArgoCD for GitOps

## Monitoring

The operator exposes metrics on port 8080:

- `/metrics`: Prometheus metrics
- `/health`: Health check endpoint  
- `/ready`: Readiness check endpoint

## Security

The operator follows security best practices:

- Runs as non-root user
- Uses read-only root filesystem
- Drops all capabilities
- Uses secure defaults

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `make test`
5. Format code: `make fmt`
6. Submit a pull request

## License

This project is licensed under the MIT License.