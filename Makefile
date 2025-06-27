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
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

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
	cargo clean
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

# Examples:
# make build APP_NAME=zerg_api
# make push APP_NAME=zerg_api TAG=v1.0.0
# make build-secure APP_NAME=zerg_api PLATFORM=linux/amd64
