import main
import argo_cd.v1alpha1 as argoV1

# Test ArgoCD applications with different configurations

# Mock config for testing
mock_config = main.Config {
    local = False
    primary_cloud = "gcp"
    area = "tel-aviv"
    gitops = "argo"
    monitoring = True
}

# Test creating multiple ArgoCD applications
test_apps = [
    main.create_argocd_app(
        name = "frontend-app"
        repo_url = "https://github.com/example/frontend"
        path = "k8s"
        destination_namespace = "frontend"
    )
    main.create_argocd_app(
        name = "backend-api"
        repo_url = "https://github.com/example/backend"
        path = "deployment"
        target_revision = "v1.2.3"
        destination_namespace = "backend"
    )
    main.create_argocd_app(
        name = "database-migration"
        repo_url = "https://github.com/example/db-migrations"
        path = "scripts"
        target_revision = "develop"
        destination_namespace = "database"
    )
]

# Test app with Helm source
helm_app = argoV1.Application {
    metadata = {
        name = "nginx-ingress"
        namespace = "argocd"
        labels = {
            "app.kubernetes.io/name" = "nginx-ingress"
            "environment" = mock_config.area
        }
    }
    spec = {
        project = "default"
        source = {
            repoURL = "https://kubernetes.github.io/ingress-nginx"
            chart = "ingress-nginx"
            targetRevision = "4.8.0"
            helm = {
                values = """
controller:
  service:
    type: LoadBalancer
  metrics:
    enabled: true
"""
            }
        }
        destination = {
            server = "https://kubernetes.default.svc"
            namespace = "ingress-nginx"
        }
        syncPolicy = {
            automated = {
                prune = True
                selfHeal = True
            }
            syncOptions = [
                "CreateNamespace=true"
            ]
        }
    }
}

# Test assertions for the applications
assert len(test_apps) == 3
assert test_apps[0].metadata.name == "frontend-app"
assert test_apps[1].spec.source.targetRevision == "v1.2.3"
assert test_apps[2].spec.destination.namespace == "database"

assert helm_app.spec.source.chart == "ingress-nginx"
assert helm_app.spec.source.targetRevision == "4.8.0"