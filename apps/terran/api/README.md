# Terran Kubernetes Operator

A Kubernetes operator that executes KCL scripts to generate and deploy resources dynamically. The operator supports event-driven reconciliation via Kafka and manages the lifecycle of generated Kubernetes resources.

## Features

- **KCL Script Execution**: Execute KCL scripts to generate Kubernetes resources dynamically
- **Multiple KCL Sources**: Support for inline scripts, ConfigMaps, Secrets, and Git repositories
- **Event-Driven Architecture**: Kafka integration for event-based reconciliation triggers
- **Resource Management**: Automatic deployment and lifecycle management of generated resources
- **Status Tracking**: Comprehensive status reporting with conditions and phases
- **Owner References**: Automatic cleanup of generated resources when parent is deleted

## Architecture

### Core Components

1. **TerranResource CRD**: Custom resource definition that defines the KCL script configuration
2. **KCL Runner**: Executes KCL scripts and parses the output to extract Kubernetes resources
3. **Reconciler**: Manages the reconciliation loop and resource deployment
4. **Kafka Client**: Handles event publishing and consumption for external triggers
5. **Operator**: Main controller that orchestrates the reconciliation process

### Resource Flow

```
TerranResource → KCL Execution → Resource Generation → Kubernetes Deployment → Status Update → Kafka Events
```

## Custom Resource Definition

The `TerranResource` CRD supports the following specification:

```yaml
apiVersion: terran.io/v1
kind: TerranResource
metadata:
  name: example-app
  namespace: default
spec:
  kcl_config:
    source:
      type: inline|configmap|secret|git
      # ... source-specific configuration
    entry_point: "main"  # Optional
    dependencies: []     # Optional KCL dependencies
  target_namespace: "default"  # Optional
  parameters:           # Optional parameters passed to KCL
    key: "value"
  event_config:         # Optional Kafka event configuration
    topic: "terran-events"
    event_types: ["ReconciliationStarted", "ReconciliationCompleted"]
    consumer_group: "terran-operator"
```

### KCL Source Types

#### Inline Script
```yaml
kcl_config:
  source:
    type: inline
    script: |
      # Your KCL script here
```

#### ConfigMap Reference
```yaml
kcl_config:
  source:
    type: configmap
    name: "my-kcl-scripts"
    key: "main.k"
```

#### Secret Reference
```yaml
kcl_config:
  source:
    type: secret
    name: "my-kcl-secrets"
    key: "script.k"
```

#### Git Repository (Future)
```yaml
kcl_config:
  source:
    type: git
    repository: "https://github.com/user/kcl-scripts.git"
    path: "scripts/main.k"
    branch: "main"  # Optional
```

## Building and Running

### Prerequisites

- Rust 1.70+
- Kubernetes cluster with kubectl access
- Optional: Kafka cluster for event handling

### Build

```bash
cd apps/terran/api
cargo build --release
```

### Local Development

```bash
# Set up your kubeconfig
export KUBECONFIG=~/.kube/config

# Run the operator locally
cargo run
```

### Docker Build

```bash
# Build container
docker build -t terran-operator:latest .

# Run in cluster
kubectl apply -f k8s/
```

## Usage

### 1. Install the CRD

The operator automatically installs the CRD on startup, but you can also install it manually:

```bash
kubectl apply -f crd.yaml
```

### 2. Create a TerranResource

```bash
kubectl apply -f example-terran-resource.yaml
```

### 3. Monitor the Status

```bash
kubectl get terranresources
kubectl describe terranresource example-app
```

### 4. View Generated Resources

The operator will create resources with owner references pointing back to the TerranResource:

```bash
kubectl get all -l "ownerReferences.name=example-app"
```

## Event System

The operator publishes events to Kafka topics for external monitoring and triggering:

### Event Types

- `ReconciliationStarted`: When reconciliation begins
- `ReconciliationCompleted`: When reconciliation succeeds
- `ReconciliationFailed`: When reconciliation fails
- `ResourcesGenerated`: When KCL execution produces resources
- `ResourcesDeployed`: When resources are deployed to the cluster

### Event Format

```json
{
  "event_type": "ReconciliationCompleted",
  "resource_name": "example-app",
  "resource_namespace": "default",
  "timestamp": "2023-11-20T10:30:00Z",
  "data": {
    "phase": "Ready",
    "resources_count": 2
  }
}
```

## Status and Conditions

The operator maintains comprehensive status information:

```yaml
status:
  phase: Ready|Processing|Failed|Pending|Deleting
  last_reconciled: "2023-11-20T10:30:00Z"
  generated_resources:
    - api_version: "v1"
      kind: "ConfigMap"
      name: "example-app-config"
      namespace: "default"
      status: "Created"
  conditions:
    - type: "Ready"
      status: "True"
      last_transition_time: "2023-11-20T10:30:00Z"
      reason: "ReconciliationSuccessful"
      message: "Successfully generated 2 resources"
```

## Development

### Project Structure

```
apps/terran/api/
├── src/
│   ├── main.rs              # Entry point
│   ├── crd.rs               # Custom Resource Definition
│   ├── operator.rs          # Main operator logic
│   ├── reconciler.rs        # Reconciliation logic
│   ├── kcl_runner.rs        # KCL execution
│   └── kafka.rs             # Event handling
├── Cargo.toml
├── README.md
└── example-terran-resource.yaml
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License.