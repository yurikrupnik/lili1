import gitops
import fluxcd_source_controller.v1 as fluxV1
import argo_cd.v1alpha1 as argoV1
import external_secrets.v1 as es


fn1 = lambda name: str -> str {
    "lol"
}

fn2 = lambda name: str -> str {
    name
}
schema Kcl:
    _src?: str = "1"
    apiVersion: "krm.kcl.dev/v1alpha1"
    kind: "KCLInput"
    metadata = {
        name = "basic"
    }
    spec = {
        source = _src
    }

#source = "oci://ghcr.io/kcl-lang/crossplane-xnetwork-kcl-functions"
schema Dam:
    _project?: str
    secrets: [es.SecretStore | es.ExternalSecret]
    _get_secrets?: (str) -> [es.SecretStore | es.ExternalSecret]
    #gitops: [argoV1 | fluxV1]
    _f1: (str) -> str
    kcl: Kcl

# oxr = option("params").oxr | {}
# ocds = option("params").ocds | {}
# _name = oxr.metadata.name
_metadata = lambda resourceName: str -> any {
    {
        name = "aris"
        annotations = {
            "krm.kcl.dev/composition-resource-name" = resourceName
        }
    }
}

cons = Dam {
    _project = "as"
    _f1 = fn1
    kcl = Kcl {_src = "github.com/kcl-lang/modules/crossplane-xnetwork-kcl-k"}
    secrets = [
        es.SecretStore {
            metadata = {
                name = _f1("asd")
                namespace = "dasd"
            }
            spec.provider.gcpsm = {
                projectID = "playground-447016"
                auth = {
                    secretRef = {
                        secretAccessKeySecretRef = {
                            name = "es-secret"
                            key = "creds"
                        }
                    }
                }
            }
        }
        es.ExternalSecret {
            metadata = {
                name = "configs"
                namespace = "crossplane-system"
            }
            spec = {
                refreshInterval = "1m"
                secretStoreRef = {
                    kind = "SecretStore"
                    name = "secret-store"
                }
                target = {
                    name = "config-secrets"
                    creationPolicy = "Owner"
                }
                dataFrom = [
                    {
                        extract = {
                            key = "manager-secrets"
                            version = "latest"
                        }
                    }
                ]
            }
        }
    ]
}
#gitops = [
# fluxV1.GitRepository {
#}
#]
