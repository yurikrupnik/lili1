import types
import cloud_config
import fluxcd_source_controller.v1 as fluxV1
import argo_cd.v1alpha1 as argoV1
import k8s.api.apps.v1 as apps
import k8s.api.core.v1 as core
import k8s.api.networking.v1 as networking
import k8s.apimachinery.pkg.apis.meta.v1 as metav1
import k8s.api.rbac.v1 as rbac
import k8s.api.batch.v1 as batch

schema App:
    name: str
    service: fluxV1.GitRepository
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
            dependsOn = [{"name" = dep} for dep in app.depends_on] if app.depends_on else []
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
            dependsOn = [{"name" = dep} for dep in app.depends_on] if app.depends_on else []
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
            } | ({"argocd.argoproj.io/sync-options" = "ServerSideApply=true"} if app.name == "kyverno" else {})
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

_create_cert_manager_crds_job = lambda -> [any] {
    namespace = "kube-system"
    name = "cert-manager-crds-installer"

    [
        core.ServiceAccount {
            metadata = metav1.ObjectMeta {
                name = name
                namespace = namespace
            }
        },
        rbac.ClusterRole {
            metadata = metav1.ObjectMeta {
                name = name
            }
            rules = [rbac.PolicyRule {
                apiGroups = ["apiextensions.k8s.io"]
                resources = ["customresourcedefinitions"]
                verbs = ["get", "list", "create", "update", "patch"]
            }]
        },
        rbac.ClusterRoleBinding {
            metadata = metav1.ObjectMeta {
                name = name
            }
            roleRef = rbac.RoleRef {
                apiGroup = "rbac.authorization.k8s.io"
                kind = "ClusterRole"
                name = name
            }
            subjects = [rbac.Subject {
                kind = "ServiceAccount"
                name = name
                namespace = namespace
            }]
        },
        batch.Job {
            metadata = metav1.ObjectMeta {
                name = name
                namespace = namespace
            }
            spec = batch.JobSpec {
                template = core.PodTemplateSpec {
                    spec = core.PodSpec {
                        serviceAccountName = name
                        restartPolicy = "OnFailure"
                        containers = [core.Container {
                            name = "kubectl"
                            image = "bitnami/kubectl:latest"
                            command = ["kubectl", "apply", "-f", "https://github.com/cert-manager/cert-manager/releases/download/v1.18.2/cert-manager.crds.yaml"]
                        }]
                    }
                }
            }
        }
    ]
}

get_gitops_resources = lambda gitops_type: str, apps: [App] -> [any] {
    # Extract unique namespaces from apps
    namespaces = [app.namespace for app in apps if app.namespace]
    unique_namespaces = list({ns: True for ns in namespaces})

    # Create namespace resources
    namespace_resources = [_create_namespace(ns) for ns in unique_namespaces]

    # Check if cert-manager is in the apps list
    has_cert_manager = len([app for app in apps if app.name == "cert-manager"]) > 0

    # Create cert-manager CRDs resources if needed
    cert_manager_crds = _create_cert_manager_crds_job() if has_cert_manager else []

    base_resources = _create_flux_resources(apps) if gitops_type == "flux" else [
        _create_argocd_application(app) for app in apps
    ] if gitops_type == "argo" else []

    namespace_resources + cert_manager_crds + base_resources
}

# Function to create app-specific resources for a single app
_create_app_folder_resources = lambda app: App -> {str: any} {
    # Check if cert-manager is this app for CRDs Job
    has_cert_manager = app.name == "cert-manager"
    cert_manager_crds = _create_cert_manager_crds_job() if has_cert_manager else []

    # Create base resources
    namespace_resource = _create_namespace(app.namespace)
    source_resource = _create_flux_helm_repository(app) if app.chart and app.helm_repo else _create_flux_git_repository(app)
    workload_resource = _create_flux_helm_release(app) if app.chart and app.helm_repo else _create_flux_kustomization(app)

    # Build resources list
    app_resources = [namespace_resource, source_resource, workload_resource]
    if has_cert_manager:
        app_resources = [namespace_resource] + cert_manager_crds + [source_resource, workload_resource]

    # Build kustomization resources list
    kustomization_resources = ["namespace.yaml"]
    if has_cert_manager:
        kustomization_resources = ["namespace.yaml", "service-account.yaml", "cluster-role.yaml", "cluster-role-binding.yaml", "cert-manager-crds.yaml"]

    if app.chart and app.helm_repo:
        kustomization_resources = kustomization_resources + ["helm-repository.yaml", "helm-release.yaml"]
    else:
        kustomization_resources = kustomization_resources + ["git-repository.yaml", "kustomization-resource.yaml"]

    # Create Kustomization file content
    kustomization_file = {
        apiVersion = "kustomize.config.k8s.io/v1beta1"
        kind = "Kustomization"
        resources = kustomization_resources
    }

    {
        "resources": app_resources
        "kustomization": kustomization_file
    }
}

# New function to generate folder structure for FluxCD
generate_flux_folder_structure = lambda apps: [App] -> {str: any} {
    {
        "clusters/second-cluster/{}".format(app.name): _create_app_folder_resources(app)
        for app in apps
    }
}

# Instantiate resources when run directly

# Output just resources when resources_only option is true
_resources_only = option("resources_only") or True

