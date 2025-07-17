def "main create" [
    --providers = [aws azure google kind upcloud]
    --secrets = false
    --gitops: string = "flux"
    --ingress
] {
    let cluster_name = "dev-cluster"

    let config = http get "https://raw.githubusercontent.com/yurikrupnik/gitops/main/cluster/cluster.yaml"

    let temp_file = $"/tmp/kind-config-($env.USER).yaml"
    $config | save $temp_file -f

    print "📦 Creating Kind cluster..."
    kind create cluster --name $cluster_name --config $temp_file
    rm $temp_file
    sleep 20sec

    kubectl cluster-info --context $"kind-($cluster_name)"
    kubectl wait --for=condition=Ready nodes --all --timeout=300s

    nu scripts/nu/ingress.nu

    if $gitops == "flux" {
      let token = (gcloud secrets versions access latest --secret="github-secret" --project="705877191303")
      $env.GITHUB_TOKEN = $token
      flux bootstrap github --token-auth --owner=yurikrupnik --repository=gitops --branch=main --path=clusters/local-cluster --personal --components-extra=image-reflector-controller,image-automation-controller --force
    } else if $gitops == "argo" {
        helm repo add argo https://argoproj.github.io/argo-helm
        helm repo update
        (
            helm upgrade --install argocd argo/argo-cd
                --namespace argocd --create-namespace
                --wait
        )

        mkdir argocd

        {
            apiVersion: argoproj.io/v1alpha1
            kind: Application
            metadata: {
                name: apps
                namespace: argocd
            }
            spec: {
                project: default
                source: {
                    repoURL: $git_url
                    targetRevision: HEAD
                    path: apps
                }
                destination: {
                    server: "https://kubernetes.default.svc"
                    namespace: a-team
                }
                syncPolicy: {
                    automated: {
                        selfHeal: true
                        prune: true
                        allowEmpty: true
                    }
                }
            }
        } | save argocd/app.yaml --force

        if $apply_apps {

            kubectl apply --filename argocd/app.yaml

        }
    }

    #if $observability {
     #   install_observability_stack $gitops
    #}

    #if $secrets {
     #   install_external_secrets $gitops
    #}

    #print_cluster_info $cluster_name $observability $secrets $gitops

}

def "main delete" [] {
    let cluster_name = "dev-cluster"
    print "📦 Deleting Kind cluster..."
    kind delete cluster --name $cluster_name
}
