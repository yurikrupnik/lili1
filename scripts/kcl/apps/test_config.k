import main
import types

# Test configuration for development environment
test_dev_config = main.Config {
    local = True
    primary_cloud = "aws"
    secondary_cloud = "gcp"
    gitops = "argo"
    area = "frankfurt"
    monitoring = True
}

# Test configuration for production environment
test_prod_config = main.Config {
    local = False
    primary_cloud = "gcp"
    secondary_cloud = "aws"
    gitops = "argo"
    area = "tel-aviv"
    monitoring = True
}

# Test configuration for staging environment
test_staging_config = main.Config {
    local = False
    primary_cloud = "aws"
    secondary_cloud = "gcp"
    gitops = "flux"
    area = "london"
    monitoring = False
}

# Test ArgoCD application creation
test_argocd_app = main.create_argocd_app(
    name = "test-app"
    repo_url = "https://github.com/example/test-repo"
    path = "manifests"
    target_revision = "main"
    destination_namespace = "test-namespace"
)

# Test assertions
assert test_dev_config.local == True
assert test_dev_config.primary_cloud == "aws"
assert test_dev_config.area == "frankfurt"
assert test_dev_config.region == "eu-central-1"

assert test_prod_config.local == False
assert test_prod_config.primary_cloud == "gcp"
assert test_prod_config.area == "tel-aviv"
assert test_prod_config.region == "me-west1"

assert test_argocd_app.metadata.name == "test-app"
assert test_argocd_app.spec.source.repoURL == "https://github.com/example/test-repo"
assert test_argocd_app.spec.destination.namespace == "test-namespace"