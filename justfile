proto-generate:
  protoc --rust_out ./libs/rust/grpc/src/generated --plugin=node_modules/ts-proto/protoc-gen-ts_proto --ts_proto_opt=nestJs=true,addGrpcMetadata=true,addNestjsRestParameter=true --ts_proto_out=./libs/node/grpc/src ./_proto/* --ts_proto_opt=esModuleInterop=true
# daily github actions
cluster-create *args:
    nu ~/configs-files/scripts/nx.nu
    nu ~/private/lili1/scripts/kcl/core/cluster.nu
    just add-tekton
#    nu -c "source ~/private/lili1/scripts/nu/config.nu; main apply kyverno"
#    nu -c "source ~/private/lili1/scripts/nu/cluster.nu; main create {{args}}"
cloud:
  gcloud builds submit --region=REGION --config [CONFIG_FILE_PATH] .

add-tekton:
  kubectl apply --filename https://storage.googleapis.com/tekton-releases/pipeline/latest/release.yaml
  kubectl apply --filename https://storage.googleapis.com/tekton-releases/dashboard/latest/release.yaml
cluster-delete:
    nu -c "source ~/private/lili1/scripts/nu/cluster.nu; main delete"

argo-secret:
    kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d
argo-workflow-secret:
  kubectl get secret argo-workflows-admin-token -n argo-workflows -o jsonpath='{.data.token}' | base64 -d

kcl-a:
  nu -c "source ~/private/lili1/scripts/nu/gitops.nu; main delete"


add-kyverno:
    #nu -c "source ~/private/lili1/scripts/nu/config.nu; main apply kyverno"
    nu -c "source ~/private/lili1/scripts/nu/config.nu; main get github"

get-provider:
  nu -c "source ~/private/lili1/scripts/nu/gitops.nu; main get github"
provider *args:
  nu -c "source ~/private/lili1/scripts/nu/gitops.nu; main get provider {{args}}"
provider *args:
  nu -c "source ~/private/lili1/scripts/nu/gitops.nu; main get provider {{args}}"

gitops:
  kcl scripts/kcl/apps/gitops.k -Y scripts/kcl/apps/kcl.yaml
kyverno:
  nu -c "source ~/private/lili1/scripts/nu/config.nu; main apply kyverno"
config:
  nu -c "source ~/private/lili1/scripts/nu/mew-config.nu; main"

apply:
  kubectl apply -k apps/zerg/api/k8s/base/
  kubectl delete -k apps/zerg/api/k8s/base/

up:
  kcl scripts/kcl/core/main.k -Y scripts/kcl/core/kcl.yaml
  nu ~/configs-files/scripts/setup-shells.nu
