import main
import types

# Test different configuration scenarios

# Scenario 1: Local development setup
local_dev_scenario = {
    config = main.Config {
        local = True
        primary_cloud = "aws"
        area = "frankfurt"
        gitops = "argo"
        monitoring = False
    }
    expected_apps = 1  # Only podinfo should be created (monitoring and external-secrets excluded)
}

# Scenario 2: Production setup with monitoring
prod_scenario = {
    config = main.Config {
        local = False
        primary_cloud = "gcp"
        area = "tel-aviv"
        gitops = "argo"
        monitoring = True
    }
    expected_apps = 3  # All apps: podinfo, monitoring, external-secrets
}

# Scenario 3: Staging without monitoring
staging_scenario = {
    config = main.Config {
        local = False
        primary_cloud = "aws"
        area = "london"
        gitops = "flux"
        monitoring = False
    }
    expected_apps = 2  # podinfo and external-secrets (no monitoring)
}

# Function to test ArgoCD app generation based on config
def test_argocd_generation(test_config: main.Config) -> [main.argoV1.Application]:
    apps = [
        main.create_argocd_app(
            name = "podinfo-${test_config.area}"
            repo_url = "https://github.com/stefanprodan/podinfo"
            path = "kustomize"
            destination_namespace = "podinfo"
        )
        main.create_argocd_app(
            name = "monitoring-stack-${test_config.area}"
            repo_url = "https://github.com/prometheus-community/helm-charts"
            path = "charts/kube-prometheus-stack"
            destination_namespace = "monitoring"
        ) if test_config.monitoring else Undefined
        main.create_argocd_app(
            name = "external-secrets-${test_config.area}"
            repo_url = "https://github.com/external-secrets/external-secrets"
            path = "deploy/charts/external-secrets"
            destination_namespace = "external-secrets-system"
        ) if not test_config.local else Undefined
    ]
    return [app for app in apps if app != Undefined]

# Test scenarios
local_dev_apps = test_argocd_generation(local_dev_scenario.config)
prod_apps = test_argocd_generation(prod_scenario.config)
staging_apps = test_argocd_generation(staging_scenario.config)

# Test environment-specific naming
env_specific_tests = {
    "local_dev_podinfo_name" = "podinfo-frankfurt"
    "prod_podinfo_name" = "podinfo-tel-aviv"
    "staging_podinfo_name" = "podinfo-london"
}

# Assertions
assert len(local_dev_apps) == local_dev_scenario.expected_apps
assert len(prod_apps) == prod_scenario.expected_apps
assert len(staging_apps) == staging_scenario.expected_apps

assert local_dev_apps[0].metadata.name == env_specific_tests.local_dev_podinfo_name
assert prod_apps[0].metadata.name == env_specific_tests.prod_podinfo_name
assert staging_apps[0].metadata.name == env_specific_tests.staging_podinfo_name

# Test labels are correctly set based on config
assert local_dev_apps[0].metadata.labels["environment"] == "frankfurt"
assert local_dev_apps[0].metadata.labels["cloud"] == "aws"
assert local_dev_apps[0].metadata.labels["gitops"] == "argo"

assert prod_apps[0].metadata.labels["environment"] == "tel-aviv"
assert prod_apps[0].metadata.labels["cloud"] == "gcp"

# Test that monitoring app is only present when monitoring is enabled
monitoring_apps = [app for app in prod_apps if "monitoring" in app.metadata.name]
assert len(monitoring_apps) == 1

local_monitoring_apps = [app for app in local_dev_apps if "monitoring" in app.metadata.name]
assert len(local_monitoring_apps) == 0