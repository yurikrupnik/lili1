use config.nu

def "main create" [
    # --providers = [aws azure google kind upcloud]
    --ingress = false
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
        { config main delete temp_files }
        #{ config main apply kyverno }
    ] | par-each { |task| do $task }

    # Read gitops configuration from kcl.yaml
    let kcl_config = (open scripts/kcl/apps/kcl.yaml)
    let config_value = ($kcl_config.kcl_options | where key == "config" | get value.0)
    let gitops = ($config_value.gitops | default "flux")

    print $"🔧 Using gitops type from kcl.yaml: (ansi cyan)($gitops)(ansi reset)"

    if $gitops == "flux" {
        let token = (gh auth token)
        $env.GITHUB_TOKEN = $token

        # Bootstrap Flux
        flux bootstrap github --token-auth --owner=yurikrupnik --repository=gitops-v2 --branch=main --path=clusters/second-cluster --personal --components-extra image-reflector-controller,image-automation-controller

        # Generate Flux folder structure
        nu scripts/nu/generate_flux_folders.nu

        # Prepare files for gitops push - get all folders in clusters/second-cluster
        let app_folders = (ls clusters/second-cluster | where type == dir | get name)
        let files_to_push = ($app_folders | each { |folder|
            let folder_name = ($folder | path basename)
            $"($folder):clusters/second-cluster/($folder_name)/"
        })

        print $"📦 Pushing ($files_to_push | length) app folders to gitops repository..."

        # Push to gitops repository
        main gitops-push --files $files_to_push --commit-msg "feat: update flux configuration with folder structure"

        # Apply from gitops repository - apply each app's kustomization
        let apply_files = ($app_folders | each { |folder|
            let folder_name = ($folder | path basename)
            $"clusters/second-cluster/($folder_name)/kustomization.yaml"
        })

        #main gitops-apply --files $apply_files

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
        print "ArgoCD resources saved to argocd.yaml"

        # Push to gitops repository
        main gitops-push --files ["argocd.yaml:argocd/argocd.yaml"] --commit-msg "feat: update argocd configuration"

        # Apply from gitops repository
        main gitops-apply --files ["argocd/argocd.yaml"]
    }
}

def "main delete" [] {
    let cluster_name = (config cluster_name)
    print "📦 Deleting Kind cluster..."
    kind delete cluster --name $cluster_name
}

def "main gitops-push" [
    --files: list<string>  # List of files/dirs to push: ["argocd.yaml:argocd/", "clusters/second-cluster/app1:clusters/second-cluster/app1/"]
    --commit-msg: string = "feat: update gitops configuration files"
] {
    let gitops_repo = "https://github.com/yurikrupnik/gitops-v2"
    let temp_dir = $"/tmp/gitops-v2-($env.USER)"

    print "🔄 Cloning gitops-v2 repository..."

    if ($temp_dir | path exists) {
        rm -rf $temp_dir
    }

    let token = (gh auth token)
    let auth_url = $"https://($token)@github.com/yurikrupnik/gitops-v2.git"

    git clone $auth_url $temp_dir

    # Store current directory before changing
    let current_dir = (pwd)

    cd $temp_dir

    # Process each file or directory mapping
    for file_mapping in $files {
        let parts = ($file_mapping | split row ":")
        let source_path_str = ($parts | get 0)
        let target_path = ($parts | get 1)

        let source_path = ($current_dir | path join $source_path_str)

        if ($source_path | path exists) {
            print $"📁 Processing ($source_path_str) -> ($target_path)..."

            # Create target directory if it doesn't exist
            let target_dir = ($target_path | path dirname)
            if $target_dir != "." and not ($target_dir | path exists) {
                mkdir $target_dir
            }

            # Check if source is a directory or file
            if ($source_path | path type) == "dir" {
                # For directories, ensure target parent exists and copy recursively
                let target_parent = ($target_path | path dirname)
                if not ($target_parent | path exists) {
                    mkdir $target_parent
                }
                cp -r $source_path $target_parent
            } else {
                # Copy individual file
                cp $source_path $target_path
            }

            git add $target_path
        } else {
            print $"⚠️  Warning: Source path ($source_path_str) does not exist, skipping..."
        }
    }

    let git_status = (git status --porcelain)

    if ($git_status | is-empty) {
        print "📝 No changes detected, files are already up to date"
    } else {
        git commit -m $"($commit_msg)

🤖 Generated with [Claude Code]\(https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"

        print "🚀 Pushing to gitops-v2 repository..."
        git push origin main
        print "✅ Successfully pushed files to gitops-v2 repository"
    }

    cd -
    rm -rf $temp_dir
}

def "main gitops-apply" [
    --files: list<string>  # List of files to apply: ["argocd/argocd.yaml", "flux/flux.yaml"]
] {
    print "🔄 Applying gitops configurations from repository..."

    let token = (gh auth token)

    for file_path in $files {
        let file_url = $"https://($token)@raw.githubusercontent.com/yurikrupnik/gitops-v2/main/($file_path)"
        print $"📦 Applying ($file_path)..."

        try {
            kubectl apply -f $file_url
        } catch { |err|
            print $"❌ Failed to apply ($file_path): ($err.msg)"
        }
    }

    print "✅ Finished applying gitops configurations from repository"
}
