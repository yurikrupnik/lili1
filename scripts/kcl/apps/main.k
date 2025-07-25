import gitops
import fluxcd_source_controller.v1 as fluxV1
import argo_cd.v1alpha1 as argoV1
import external_secrets.v1 as es

schema Dam:
    secrets: es
    gitops: argoV1 | fluxV1






