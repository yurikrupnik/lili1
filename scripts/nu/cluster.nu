use config.nu

def "main create" [
    # --providers = [aws azure google kind upcloud]
    --gitops: string = "flux"
    --ingress = true
] {
    let cluster_name = (config cluster_name)

    let config = http get "https://raw.githubusercontent.com/yurikrupnik/gitops/main/cluster/cluster.yaml"

    let temp_file = $"/tmp/kind-config-($env.USER).yaml"
    $config | save $temp_file -f

    print "📦 Creating Kind cluster..."
    kind create cluster --name $cluster_name --config $temp_file
    rm $temp_file

    kubectl cluster-info --context $"kind-($cluster_name)"
    kubectl wait --for=condition=Ready nodes --all --timeout=300s

    [
        { nu scripts/nu/ingress.nu }
        { config main apply kyverno }
        { config main delete temp_files }
    ] | par-each { |task| do $task }

    if $gitops == "flux" { # error  failed to commit component manifests: cannot create empty commit: clean working tree
                                   # error: Recipe `cluster-create` failed on line 2 with exit code 1
      let token = (gcloud secrets versions access latest --secret="github-secret" --project="705877191303")
      $env.GITHUB_TOKEN = $token
      flux bootstrap github --token-auth --owner=yurikrupnik --repository=gitops-v2 --branch=main --path=clusters/first-cluster --personal --components-extra image-reflector-controller,image-automation-controller
      # flux bootstrap github --token-auth --owner=yurikrupnik --repository=gitops-v2 --branch=main --path=clusters/local-cluster --personal --components-extra=image-reflector-controller,image-automation-controller --force
      #flux bootstrap github --token-auth --owner=yurikrupnik --repository=gitops --branch=main --path=clusters/local-cluster --personal --components-extra=image-reflector-controller,image-automation-controller --force

    } else if $gitops == "argo" {
        kubectl create namespace argocd
        kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

        #kubectl apply -k manifests/gitops/argocd
        kubectl wait --for=condition=Available deployment/argocd-server -n argocd --timeout=300s
        let items = (kcl scripts/kcl/apps/resources_only.k -Y scripts/kcl/apps/kcl.yaml | from yaml)
        let app_project = "apiVersion: argoproj.io/v1alpha1
kind: AppProject
metadata:
  name: production
  namespace: argocd
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  description: Production project
  sourceRepos:
    - '*'
  destinations:
    - namespace: '*'
      server: '*'
  clusterResourceWhitelist:
    - group: '*'
      kind: '*'
  namespaceResourceWhitelist:
    - group: '*'
      kind: '*'"
        let yaml_content = ($items.items | each { |item| $item | to yaml } | str join "\n---\n")
        $"($app_project)\n---\n($yaml_content)" | save argocd.yaml -f
        print "Items saved to argocd.yaml"
        main push-to-gitops
        main apply-via-gitops

    }


    #helm repo add cnpg https://cloudnative-pg.github.io/charts
    #(
     # helm upgrade --install cnpg
      #  --namespace cnpg-system
      #  --create-namespace
      #  cnpg/cloudnative-pg
    #)
    #helm repo add grafana https://grafana.github.io/helm-charts
    #helm install my-loki grafana/loki --version 6.32.0
    #if $observability {
     #   install_observability_stack $gitops
    #}

    #if $secrets {
     #   install_external_secrets $gitops
    #}

    #print_cluster_info $cluster_name $observability $secrets $gitops

}

def "main delete" [] {
    let cluster_name = (config cluster_name)
    print "📦 Deleting Kind cluster..."
    kind delete cluster --name $cluster_name
}

# def "main gitops" [
#     --config-file?: string
#     --name?: string
#     --repo?: string
#     --path: string = "."
#     --namespace: string = "default"
#     --chart?: string
#     --helm-repo?: string
#     --gitops-type?: string
#     --cloud?: string
#     --area?: string
#     --revision: string = "HEAD"
#     --local: bool = False
#     --monitoring: bool = false
#     --observability: bool = false
#     --auto-sync: bool = true
#     --prune: bool = true
#     --self-heal: bool = true
#     --dry-run: bool = false
# ] {
#     let base_config = if ($config_file | is-empty) {
#         {
#             local: $local
#             primary_cloud: ($cloud | default "gcp")
#             gitops: ($gitops_type | default "flux")
#             area: ($area | default "tel-aviv")
#             monitoring: $monitoring
#             observability: $observability
#         }
#     } else {
#         open $config_file | get config? | default {}
#     }

#     let apps = if ($config_file | is-empty) {
#         if ($name | is-empty) or ($repo | is-empty) {
#             error make { msg: "Either provide --config-file or both --name and --repo" }
#         }
#         [{
#             name: $name
#             enabled: true
#             repo_url: $repo
#             path: $path
#             namespace: $namespace
#             chart: $chart
#             helm_repo: $helm_repo
#             target_revision: $revision
#             auto_sync: $auto_sync
#             prune: $prune
#             self_heal: $self_heal
#         }]
#     } else {
#         open $config_file | get applications? | default []
#     }

#     let final_config = {
#         config: $base_config
#         applications: $apps
#     }

#     let kcl_config = $"scripts/kcl/apps/generated_config_($env.USER).yaml"
#     $final_config | to yaml | save $kcl_config -f

#     print $"📝 Generated KCL config at ($kcl_config)"
#     print $"🚀 Creating gitops resources for ($base_config.gitops)..."

#     if $dry_run {
#         print "🔍 Dry run - showing generated resources:"
#         kcl run scripts/kcl/apps/main.k -Y $kcl_config
#     } else {
#         kcl run scripts/kcl/apps/main.k -Y $kcl_config | kubectl apply -f -
#         print $"✅ Successfully deployed using ($base_config.gitops)"
#     }

#     if not $dry_run {
#         rm $kcl_config
#     }
# }

def "main push-to-gitops" [] {
    let gitops_repo = "https://github.com/yurikrupnik/gitops-v2"
    let temp_dir = $"/tmp/gitops-v2-($env.USER)"

    print "🔄 Cloning gitops-v2 repository..."

    if ($temp_dir | path exists) {
        rm -rf $temp_dir
    }

    let token = (gcloud secrets versions access latest --secret="github-secret" --project="705877191303")
    let auth_url = $"https://($token)@github.com/yurikrupnik/gitops-v2.git"

    git clone $auth_url $temp_dir

    print "📁 Creating argocd folder and copying argocd.yaml to gitops repository..."
    mkdir $"($temp_dir)/argocd"
    cp argocd.yaml $"($temp_dir)/argocd/argocd.yaml"

    cd $temp_dir

    git add argocd/argocd.yaml

    let git_status = (git status --porcelain)

    if ($git_status | is-empty) {
        print "📝 No changes detected, argocd.yaml is already up to date"
    } else {
        let commit_msg = "feat: update argocd.yaml configuration file

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"

        git commit -m $commit_msg

        print "🚀 Pushing to gitops-v2 repository..."
        git push origin main
        print "✅ Successfully pushed argocd.yaml to gitops-v2 repository"
    }

    cd -
    rm -rf $temp_dir
}

def "main apply-via-gitops" [] {
    print "🔄 Applying ArgoCD configuration via gitops repository..."

    let token = (gh auth token)
    let argocd_url = $"https://($token)@raw.githubusercontent.com/yurikrupnik/gitops-v2/main/argocd/argocd.yaml"

    kubectl apply -f $argocd_url

    print "✅ Successfully applied ArgoCD configuration from gitops repository"
}
