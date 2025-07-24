import types
import cloud_config
import fluxcd_source_controller.v1 as fluxV1
import argo_cd.v1alpha1 as argoV1

schema App:
    name: str
    enabled: bool = True
    repo_url: str
    path: str = "."
    namespace: str
    chart?: str
    helm_repo?: str
    target_revision: str = "HEAD"
    auto_sync: bool = True
    prune: bool = True
    self_heal: bool = True


_create_flux_git_repository = lambda app: App -> fluxV1.GitRepository {
    fluxV1.GitRepository {
        metadata = {
            name = app.name
            namespace = app.namespace
        }
        spec = {
            interval = "5m0s"
            url = app.repo_url
            ref = {
                branch = "master" if app.target_revision == "HEAD" else app.target_revision
            }
        }
    }
}

_create_flux_helm_repository = lambda app: App -> fluxV1.HelmRepository {
    fluxV1.HelmRepository {
        metadata = {
            name = app.name
            namespace = app.namespace
        }
        spec = {
            interval = "60m0s"
            url = app.helm_repo
        }
    }
}

# For now, let's create basic YAML structures for HelmRelease and Kustomization
# since the specific KCL modules might not be available

_create_flux_helm_release = lambda app: App -> any {
    {
        apiVersion = "helm.toolkit.fluxcd.io/v2beta1"
        kind = "HelmRelease"
        metadata = {
            name = app.name
            namespace = app.namespace
        }
        spec = {
            interval = "5m0s"
            chart = {
                spec = {
                    chart = app.chart
                    version = app.target_revision if app.target_revision != "HEAD" else "*"
                    sourceRef = {
                        kind = "HelmRepository"
                        name = app.name
                        namespace = app.namespace
                    }
                }
            }
        }
    }
}

_create_flux_kustomization = lambda app: App -> any {
    {
        apiVersion = "kustomize.toolkit.fluxcd.io/v1"
        kind = "Kustomization"
        metadata = {
            name = app.name
            namespace = app.namespace
        }
        spec = {
            interval = "5m0s"
            path = app.path
            prune = app.prune
            sourceRef = {
                kind = "GitRepository"
                name = app.name
                namespace = app.namespace
            }
        }
    }
}

_create_argocd_application = lambda config, app: App -> argoV1.Application {
    argoV1.Application {
        metadata = {
            name = app.name
            namespace = "argocd"
            labels = {
                "app.kubernetes.io/name" = app.name
                "generated-by": "gitops.k"
            }
        }
        spec = {
            project = "default"
            source = {
                repoURL = app.repo_url
                targetRevision = app.target_revision
                path = app.path
            } if not app.chart else {
                repoURL = app.helm_repo
                targetRevision = app.target_revision
                chart = app.chart
            }
            destination = {
                server = "https://kubernetes.default.svc"
                namespace = app.namespace
            }
            syncPolicy = {
                automated = {
                    prune = app.prune
                    selfHeal = app.self_heal
                } if app.auto_sync else {}
                syncOptions = [
                    "CreateNamespace=true"
                ]
            }
        }
    }
}

_create_flux_resources = lambda apps: [App] -> [any] {
    # Create all source resources first (GitRepo or HelmRepo)
    source_resources = [
        _create_flux_helm_repository(app) if app.chart and app.helm_repo else _create_flux_git_repository(app)
        for app in apps
    ]
    # Create all workload resources (HelmRelease or Kustomization)
    workload_resources = [
        _create_flux_helm_release(app) if app.chart and app.helm_repo else _create_flux_kustomization(app)
        for app in apps
    ]
    # Combine both lists
    source_resources + workload_resources
}

get_gitops_resources = lambda gitops_type: str, config, apps: [App] -> [any] {
    _create_flux_resources(apps) if gitops_type == "flux" else [
        _create_argocd_application(config, app) for app in apps
    ] if gitops_type == "argo" else []
}

# Instantiate resources when run directly
config = option("config") or {
    local: True
    primary_cloud: "gcp"
    secondary_cloud: "aws"
    gitops: "flux"
    area: "tel-aviv"
}

_applications: [App] = option("applications") or []
_enabled_applications = [app for app in _applications if app.enabled]
_resources = get_gitops_resources(config.gitops, config, _enabled_applications)

# Output just resources when resources_only option is true
_resources_only = option("resources_only") or True
items = _resources if _resources_only else Undefined

