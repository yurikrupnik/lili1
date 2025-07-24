import types
import cloud_config
import gitops
import fluxcd_source_controller.v1 as fluxV1
import argo_cd.v1alpha1 as argoV1
import external_secrets.v1 as es

schema Dam:
    secrets: es
    gitops: argoV1 | fluxV1
    config: Config{}



schema Config:
    local: bool = True
    primary_cloud: types.CloudType = "gcp"
    secondary_cloud: types.CloudType = "aws"
    gitops: types.GitopsType = "flux"
    area: types.AreaType = "tel-aviv"
    db: types.DatabaseType = "postgres"
    region: types.RegionType = cloud_config._cloud_regions[primary_cloud][area]
    monitoring: bool = True
    observability: bool = True



config: Config = option("config") or Config {}

_applications: [gitops.App] = option("applications") or []


_enabled_applications = [app for app in _applications if app.enabled]
gitops_resources = gitops.get_gitops_resources(config.gitops, config, _enabled_applications)

flux_resources = [
    fluxV1.GitRepository {
        metadata = {
            name = "podinfo"
            namespace = "default"
        }
        spec = {
            interval = "5m0s"
            url = "https://github.com/stefanprodan/podinfo"
            ref = {
                branch = "master"
            }
        }
    }
]

argocd_apps = [
    argoV1.Application {
        metadata = {
            name = "podinfo-${config.area}"
            namespace = "argocd"
            labels = {
                "app.kubernetes.io/name" = "podinfo"
                "environment" = config.area
                "cloud" = config.primary_cloud
                "gitops" = config.gitops
            }
        }
        spec = {
            project = "default"
            source = {
                repoURL = "https://github.com/stefanprodan/podinfo"
                targetRevision = "HEAD"
                path = "kustomize"
            }
            destination = {
                server = "https://kubernetes.default.svc"
                namespace = "podinfo"
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
    argoV1.Application {
        metadata = {
            name = "monitoring-stack-${config.area}"
            namespace = "argocd"
            labels = {
                "app.kubernetes.io/name" = "monitoring"
                "environment" = config.area
                "cloud" = config.primary_cloud
                "gitops" = config.gitops
            }
        }
        spec = {
            project = "default"
            source = {
                repoURL = "https://github.com/prometheus-community/helm-charts"
                targetRevision = "HEAD"
                path = "charts/kube-prometheus-stack"
            }
            destination = {
                server = "https://kubernetes.default.svc"
                namespace = "monitoring"
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
    } if config.monitoring else Undefined
    argoV1.Application {
        metadata = {
            name = "external-secrets-${config.area}"
            namespace = "argocd"
            labels = {
                "app.kubernetes.io/name" = "external-secrets"
                "environment" = config.area
                "cloud" = config.primary_cloud
                "gitops" = config.gitops
            }
        }
        spec = {
            project = "default"
            source = {
                repoURL = "https://github.com/external-secrets/external-secrets"
                targetRevision = "HEAD"
                path = "deploy/charts/external-secrets"
            }
            destination = {
                server = "https://kubernetes.default.svc"
                namespace = "external-secrets-system"
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
    } if not config.local else Undefined
]

filtered_argocd_apps = [app for app in argocd_apps if app != Undefined]

