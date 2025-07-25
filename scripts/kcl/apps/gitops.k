import types
import cloud_config
import fluxcd_source_controller.v1 as fluxV1
import argo_cd.v1alpha1 as argoV1

schema App:
    name: str
    enabled: bool = True
    repo_url?: str
    path: str = "."
    namespace: str
    chart?: str
    helm_repo?: str
    target_revision: str = "HEAD"
    auto_sync: bool = True
    prune: bool = True
    self_heal: bool = True
    sync_wave: int = 0
    depends_on: [str] = []


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
            dependsOn = [{"name" = dep} for dep in app.depends_on] if app.depends_on else None
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
            dependsOn = [{"name" = dep} for dep in app.depends_on] if app.depends_on else None
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

_create_argocd_application = lambda app: App -> argoV1.Application {
    argoV1.Application {
        metadata = {
            name = app.name
            namespace = "argocd"
            labels = {
                "app.kubernetes.io/name" = app.name
                "generated-by": "gitops.k"
            }
            annotations = {
                "argocd.argoproj.io/sync-wave" = str(app.sync_wave)
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

_create_namespace = lambda name: str -> any {
    {
        apiVersion = "v1"
        kind = "Namespace"
        metadata = {
            name = name
        }
    }
}

get_gitops_resources = lambda gitops_type: str, apps: [App] -> [any] {
    # Extract unique namespaces from apps
    namespaces = [app.namespace for app in apps if app.namespace]
    unique_namespaces = list({ns: True for ns in namespaces})

    # Create namespace resources
    namespace_resources = [_create_namespace(ns) for ns in unique_namespaces]

    # Check if cert-manager is in the apps list
    has_cert_manager = len([app for app in apps if app.name == "cert-manager"]) > 0

    # Create cert-manager CRDs Job if needed
    cert_manager_crds = [{
        apiVersion = "batch/v1"
        kind = "Job"
        metadata = {
            name = "cert-manager-crds-installer"
            namespace = "kube-system"
        }
        spec = {
            template = {
                spec = {
                    serviceAccountName = "cert-manager-crds-installer"
                    restartPolicy = "OnFailure"
                    containers = [{
                        name = "kubectl"
                        image = "bitnami/kubectl:latest"
                        command = ["kubectl", "apply", "-f", "https://github.com/cert-manager/cert-manager/releases/download/v1.18.2/cert-manager.crds.yaml"]
                    }]
                }
            }
        }
    }] if has_cert_manager else []

    base_resources = _create_flux_resources(apps) if gitops_type == "flux" else [
        _create_argocd_application(app) for app in apps
    ] if gitops_type == "argo" else []

    namespace_resources + cert_manager_crds + base_resources
}

# Instantiate resources when run directly

# Output just resources when resources_only option is true
_resources_only = option("resources_only") or True

