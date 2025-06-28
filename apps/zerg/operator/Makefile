# Zerg Operator Makefile

# Variables
IMAGE_NAME := zerg-operator
IMAGE_TAG := latest
NAMESPACE := zerg-system
KUBECONFIG ?= ~/.kube/config

# Build the operator
.PHONY: build
build:
	cargo build --release

# Run tests
.PHONY: test
test:
	cargo test

# Build Docker image
.PHONY: docker-build
docker-build:
	docker build -t $(IMAGE_NAME):$(IMAGE_TAG) .

# Run the operator locally
.PHONY: run
run:
	RUST_LOG=info cargo run -- --log-level=info --namespace=$(NAMESPACE)

# Install CRDs
.PHONY: install-crds
install-crds:
	kubectl apply -f k8s/crd.yaml

# Uninstall CRDs
.PHONY: uninstall-crds
uninstall-crds:
	kubectl delete -f k8s/crd.yaml

# Create namespace
.PHONY: create-namespace
create-namespace:
	kubectl create namespace $(NAMESPACE) --dry-run=client -o yaml | kubectl apply -f -

# Deploy the operator
.PHONY: deploy
deploy: create-namespace install-crds
	kubectl apply -f k8s/deployment.yaml

# Undeploy the operator
.PHONY: undeploy
undeploy:
	kubectl delete -f k8s/deployment.yaml
	kubectl delete namespace $(NAMESPACE) --ignore-not-found=true

# Apply example configurations
.PHONY: apply-examples
apply-examples:
	kubectl apply -f examples/

# Delete example configurations
.PHONY: delete-examples
delete-examples:
	kubectl delete -f examples/

# View logs
.PHONY: logs
logs:
	kubectl logs -n $(NAMESPACE) -l app.kubernetes.io/name=zerg-operator -f

# Port forward to metrics
.PHONY: port-forward-metrics
port-forward-metrics:
	kubectl port-forward -n $(NAMESPACE) svc/zerg-operator-metrics 8080:8080

# Generate CRD documentation
.PHONY: docs
docs:
	@echo "Generating CRD documentation..."
	@echo "DependencyManager CRD allows you to:"
	@echo "- Install dependencies like external-secrets, crossplane, loki"
	@echo "- Configure GitOps with Flux or ArgoCD"
	@echo "- Setup CI/CD pipelines with Tekton or Argo Workflows"

# Check operator status
.PHONY: status
status:
	kubectl get pods -n $(NAMESPACE) -l app.kubernetes.io/name=zerg-operator
	kubectl get dependencymanagers -A

# Format code
.PHONY: fmt
fmt:
	cargo fmt

# Lint code
.PHONY: lint
lint:
	cargo clippy -- -D warnings

# Clean build artifacts
.PHONY: clean
clean:
	cargo clean

# Full development cycle
.PHONY: dev
dev: fmt lint test build

# CI/CD pipeline
.PHONY: ci
ci: fmt lint test docker-build

# Help
.PHONY: help
help:
	@echo "Available commands:"
	@echo "  build              - Build the operator"
	@echo "  test               - Run tests"
	@echo "  docker-build       - Build Docker image"
	@echo "  run                - Run operator locally"
	@echo "  install-crds       - Install CRDs"
	@echo "  uninstall-crds     - Remove CRDs"
	@echo "  create-namespace   - Create operator namespace"
	@echo "  deploy             - Deploy operator to cluster"
	@echo "  undeploy           - Remove operator from cluster"
	@echo "  apply-examples     - Apply example configurations"
	@echo "  delete-examples    - Delete example configurations"
	@echo "  logs               - View operator logs"
	@echo "  port-forward-metrics - Port forward metrics endpoint"
	@echo "  status             - Check operator status"
	@echo "  fmt                - Format code"
	@echo "  lint               - Lint code"
	@echo "  clean              - Clean build artifacts"
	@echo "  dev                - Full development cycle"
	@echo "  ci                 - CI/CD pipeline"
	@echo "  help               - Show this help"