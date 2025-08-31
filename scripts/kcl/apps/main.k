import gitops
import fluxcd_source_controller.v1 as fluxV1
import argo_cd.v1alpha1 as argoV1
import external_secrets.v1 as es

mixin BaseMixin:
    # _project: str
    name: str
    age: int

#import crossplane
fn1a = lambda name: str -> str {
    "lol"
}

fn2a = lambda name: str -> str {
    name
}
schema Kcla:
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
schema Amm:
    _project?: str
    #secrets: [es.SecretStore | es.ExternalSecret]
    #_get_secrets?: (str) -> [es.SecretStore | es.ExternalSecret]
    #gitops: [argoV1 | fluxV1]
    #_f1: (str) -> str
    #kcl: Kcl

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

amms = Amm {
    _project = "as"
    #_f1 = fn1
    #kcl = Kcl {_src = "github.com/kcl-lang/modules/crossplane-xnetwork-kcl-k"}

}
#gitops = [
# fluxV1.GitRepository {
#}
#]
