# Makefile for Docker operations in the monorepo

# Variables
DOCKER_REGISTRY ?= yurikrupnik
APP_NAME ?= zerg_api
IMAGE_NAME ?= $(DOCKER_REGISTRY)/$(APP_NAME)
TAG ?= latest
PLATFORM ?= linux/amd64,linux/arm64

# Default target
.PHONY: help
help: ## Show this help message
	@echo "🎮 Starcraft APIs Development Commands"
	@echo ""
	@echo "Tilt Development (Recommended):"
	@grep -E '^tilt-.*:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Docker Development:"
	@grep -E '^(dev-|build|push).*:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Rust Development:"
	@grep -E '^(test|fmt|lint|clean-rust).*:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Kubernetes:"
	@grep -E '^(cluster-|deploy-).*:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Other targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -v -E '^(tilt-|dev-|build|push|test|fmt|lint|clean-rust|cluster-|deploy-)' | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Development targets
.PHONY: dev-up
local: ## Start development environment
	nu -c "source ~/.config/nushell/config.nu; main delete temp_files"
	kind create cluster
	sleep 20
	kubectl create namespace argocd
	kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml
	istioctl install --set profile=demo -y
	kubectl label namespace default istio-injection=enabled
	helm upgrade --install kro oci://ghcr.io/kro-run/kro/kro --namespace kro --create-namespace
	helm upgrade --install cnpg cloudnative-pg --repo https://cloudnative-pg.github.io/charts --namespace cnpg-system --create-namespace --wait

	#kubectl apply -f manifests/configs/stage.yaml

# Development targets
.PHONY: dev-up
dev-up: ## Start development environment
	docker-compose -f docker-compose.dev.yml up -d

.PHONY: dev-down
dev-down: ## Stop development environment
	docker-compose -f docker-compose.dev.yml down

.PHONY: dev-logs
dev-logs: ## Show development logs
	docker-compose -f docker-compose.dev.yml logs -f

# Build targets
.PHONY: build
build: ## Build Docker image for specific app
	docker build \
		-f apps/$(APP_NAME)/Dockerfile \
		-t $(IMAGE_NAME):$(TAG) \
		--build-arg APP_NAME=$(APP_NAME) \
		.

.PHONY: build-secure
build-secure: ## Build secure multi-arch Docker image
	docker buildx build \
		-f docker/rust.secure.Dockerfile \
		-t $(IMAGE_NAME):$(TAG) \
		--platform $(PLATFORM) \
		--build-arg APP_NAME=$(APP_NAME) \
		--load \
		.

.PHONY: build-minimal
build-minimal: ## Build ultra-minimal Docker image (~1.5MB)
	docker build \
		-f docker/rust.minimal.Dockerfile \
		-t $(IMAGE_NAME):$(TAG)-minimal \
		--build-arg APP_NAME=$(APP_NAME) \
		.

.PHONY: build-original
build-original: ## Build using original rust.Dockerfile
	docker build \
		-f rust.Dockerfile \
		-t $(IMAGE_NAME):$(TAG) \
		--build-arg APP_NAME=$(APP_NAME) \
		.

.PHONY: build-all
build-all: ## Build all Rust applications
	@for app in $$(find apps -name "Cargo.toml" -exec dirname {} \; | xargs -I {} basename {}); do \
		echo "Building $$app..."; \
		$(MAKE) build APP_NAME=$$app; \
	done

# Production targets
.PHONY: push
push: build ## Build and push image to registry
	docker push $(IMAGE_NAME):$(TAG)

.PHONY: push-secure
push-secure: ## Build and push secure multi-arch image
	docker buildx build \
		-f docker/rust.secure.Dockerfile \
		-t $(IMAGE_NAME):$(TAG) \
		--platform $(PLATFORM) \
		--build-arg APP_NAME=$(APP_NAME) \
		--push \
		.

# Security targets
.PHONY: scan
scan: ## Scan image for vulnerabilities
	docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
		-v $(HOME)/.cache:/root/.cache \
		aquasec/trivy image $(IMAGE_NAME):$(TAG)

.PHONY: security-check
security-check: build scan ## Build and scan image

# Utility targets
.PHONY: clean
clean: ## Clean up Docker resources
	docker system prune -f
	docker volume prune -f

.PHONY: clean-all
clean-all: ## Clean up all Docker resources (including images)
	docker system prune -af
	docker volume prune -f

.PHONY: run
run: ## Run the application locally
	docker run --rm -p 8080:8080 --name $(APP_NAME) $(IMAGE_NAME):$(TAG)

.PHONY: shell
shell: ## Get shell access to the container
	docker run --rm -it --entrypoint /bin/sh $(IMAGE_NAME):$(TAG)

# NX integration
.PHONY: nx-build
nx-build: ## Build using NX
	npx nx run $(APP_NAME):docker-build

.PHONY: nx-run
nx-run: ## Run using NX
	npx nx run $(APP_NAME):docker-run

# Tilt Development (Recommended)
.PHONY: tilt-up
tilt-up: ## Start Tilt development environment
	@echo "🚀 Starting Tilt development environment..."
	tilt up

.PHONY: tilt-down
tilt-down: ## Stop Tilt development environment
	@echo "🛑 Stopping Tilt development environment..."
	tilt down

.PHONY: tilt-logs
tilt-logs: ## View Tilt logs
	tilt logs

# Rust Development
.PHONY: test
test: ## Run all tests
	@echo "🧪 Running tests..."
	cargo test --workspace

.PHONY: fmt
fmt: ## Format code
	@echo "📝 Formatting code..."
	cargo fmt --all

.PHONY: lint
lint: ## Run clippy linting
	@echo "🔍 Running clippy..."
	cargo clippy --workspace -- -D warnings

.PHONY: clean-rust
clean-rust: ## Clean Rust build artifacts
	@echo "🧹 Cleaning Rust build artifacts..."
	cargo clean

# Cluster Management
.PHONY: cluster-create
cluster-create: ## Create kind cluster for development
	@echo "🏗️  Creating kind cluster..."
	kind create cluster --name starcraft-dev
	@echo "✅ Cluster created. Setting context..."
	kubectl config use-context kind-starcraft-dev

.PHONY: cluster-delete
cluster-delete: ## Delete kind cluster
	@echo "🗑️  Deleting kind cluster..."
	kind delete cluster --name starcraft-dev

# Kubernetes Deployment
.PHONY: deploy-dev
deploy-dev: ## Deploy to dev environment (kubectl)
	@echo "🚀 Deploying to development..."
	kubectl apply -k manifests/kustomize/multi-app/dev

.PHONY: deploy-prod
deploy-prod: ## Deploy to prod environment (kubectl)
	@echo "🚀 Deploying to production..."
	kubectl apply -k manifests/kustomize/multi-app/prod

# Check dependencies
.PHONY: check-deps
check-deps: ## Check development dependencies
	@echo "🔧 Checking dependencies..."
	@command -v tilt >/dev/null 2>&1 || { echo "❌ tilt is not installed. See: https://tilt.dev/"; exit 1; }
	@command -v kubectl >/dev/null 2>&1 || { echo "❌ kubectl is not installed"; exit 1; }
	@command -v kind >/dev/null 2>&1 || { echo "❌ kind is not installed. See: https://kind.sigs.k8s.io/"; exit 1; }
	@command -v cargo >/dev/null 2>&1 || { echo "❌ cargo is not installed. Install Rust: https://rustup.rs/"; exit 1; }
	@echo "✅ All dependencies are installed"

# Full setup for new developers
.PHONY: setup
setup: check-deps cluster-create ## Full setup for new developers
	@echo "🎉 Setup complete! Run 'make tilt-up' to start development."

# Aliases for convenience
.PHONY: dev
dev: tilt-up ## Alias for tilt-up

# Examples:
# make build APP_NAME=zerg_api
# make push APP_NAME=zerg_api TAG=v1.0.0
# make build-secure APP_NAME=zerg_api PLATFORM=linux/amd64
# make tilt-up
# make setup
