cluster-create *args:
    nu -c "source ~/private/lili1/scripts/nu/cluster.nu; main create {{args}}"


cluster-delete:
    nu -c "source ~/private/lili1/scripts/nu/cluster.nu; main delete"

argo-secret:
    kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d

kcl-a:
  nu -c "source ~/private/lili1/scripts/nu/gitops.nu; main delete"


add-kyverno:
    #nu -c "source ~/private/lili1/scripts/nu/config.nu; main apply kyverno"
    nu -c "source ~/private/lili1/scripts/nu/config.nu; main get github"

get-provider:
  nu -c "source ~/private/lili1/scripts/nu/gitops.nu; main get github"
provider *args:
  nu -c "source ~/private/lili1/scripts/nu/gitops.nu; main get provider {{args}}"

gitops:
  kcl scripts/kcl/apps/gitops.k -Y scripts/kcl/apps/kcl.yaml
kyverno:
  nu -c "source ~/private/lili1/scripts/nu/config.nu; main apply kyverno"

apply:
  kubectl apply -k apps/zerg/api/k8s/base/
  kubectl delete -k apps/zerg/api/k8s/base/
