# Advanced Configuration Management for Kubernetes Applications
# Uses KCL scripts and dynamic helm repository management

# Configuration constants
const DEFAULT_CLUSTER = "dev-cluster"
const KCL_APPS_PATH = "scripts/kcl/apps"
const KCL_CONFIG_FILE = "scripts/kcl/apps/kcl.yaml"

# Load KCL configuration dynamically
def load_kcl_config [] {
    open $KCL_CONFIG_FILE | get kcl_options | reduce --fold {} {|item, acc|
        $acc | insert $item.key $item.value
    }
}

# Get applications from KCL config
def get_local_applications [] {
    let config = load_kcl_config
    $config.applications
}

# Get applications from KCL config
def get_applications [] {
    let config = load_kcl_config
    $config.applications
}

# Get application by name
def get_app [name] {
    get_applications | where name == $name | first
}

# Dynamic helm repository management
def helm_repo_manager [action, name?, url?] {
    match $action {
        "add" => {
            if ($name | is-empty) or ($url | is-empty) {
                error make {msg: "Name and URL required for adding repo"}
            }
            helm repo add $name $url
        }
        "update" => { helm repo update }
        "list" => { helm repo list }
        "remove" => {
            if ($name | is-empty) {
                error make {msg: "Name required for removing repo"}
            }
            helm repo remove $name
        }
        _ => { error make {msg: $"Unknown action: ($action)"} }
    }
}

# Advanced helm install function with dynamic configuration
def helm_install_app [
    app_name
    --wait = false
    --create-namespace = true
    --dry-run = false
    --timeout = 300sec
    --values = ""
] {
    let app = get_app $app_name
    if ($app | is-empty) {
        error make {msg: $"Application '($app_name)' not found in KCL config"}
    }

    # Add helm repository if specified
    if "helm_repo" in $app {
        helm_repo_manager "add" $app_name $app.helm_repo
        helm_repo_manager "update"
    }

    # Build helm command arguments
    mut args = ["upgrade", "--install", $app_name]

    # Add chart specification
    if "helm_repo" in $app {
        $args = ($args | append $"($app_name)/($app.chart)")
    } else {
        $args = ($args | append $app.chart)
    }

    # Add namespace
    if "namespace" in $app {
        $args = ($args | append ["--namespace", $app.namespace])
    }

    # Add conditional flags
    if $create_namespace {
        $args = ($args | append "--create-namespace")
    }

    if $wait {
        $args = ($args | append "--wait")
        $args = ($args | append ["--timeout", ($timeout | format duration sec)])
    }

    if $dry_run {
        $args = ($args | append "--dry-run")
    }

    # Add target revision if specified
    if "target_revision" in $app {
        $args = ($args | append ["--version", $app.target_revision])
    }

    # Add custom values file if provided
    if not ($values | is-empty) {
        $args = ($args | append ["--values", $values])
    }

    # Execute helm command
    helm ...$args
}

# Execute KCL scripts with parameters
def execute_kcl [
    script
    --config = ""
    --output = "yaml"
    --settings = {}
] {
    mut kcl_args = [$script]

    if not ($config | is-empty) {
        $kcl_args = ($kcl_args | append ["--config", $config])
    }

    $kcl_args = ($kcl_args | append ["--format", $output])

    # Add settings as key-value pairs
    for setting in ($settings | transpose key value) {
        $kcl_args = ($kcl_args | append ["-S", $"($setting.key)=($setting.value)"])
    }

    kcl ...$kcl_args
}

# Batch application deployment with sync waves
def deploy_applications [
    apps
    --wait = true
    --parallel = false
    --max-wave = 10
] {
    let all_apps = get_applications
    let selected_apps = $all_apps | where name in $apps

    # Group by sync wave
    let waves = $selected_apps | group-by {|app|
        if "sync_wave" in $app { $app.sync_wave } else { 0 }
    }

    # Deploy by wave order
    for wave in 0..$max_wave {
        if $wave in ($waves | columns) {
            let wave_apps = $waves | get $wave
            print $"Deploying wave ($wave): ($wave_apps | get name | str join ', ')"

            if $parallel {
                # Deploy in parallel within wave
                $wave_apps | par-each {|app|
                    helm_install_app $app.name --wait=$wait
                }
            } else {
                # Deploy sequentially within wave
                for app in $wave_apps {
                    helm_install_app $app.name --wait=$wait
                }
            }
        }
    }
}

# Generate GitOps manifests using KCL
def generate_gitops_manifests [
    --output-dir = "manifests/gitops"
    --gitops-type = "flux"
] {
    let config = load_kcl_config
    let apps = $config.applications

    mkdir $output_dir

    for app in $apps {
        let manifest = execute_kcl $"($KCL_APPS_PATH)/gitops.k" --settings {
            app_name: $app.name
            namespace: $app.namespace
            repo_url: $app.repo_url
            path: ($app.path? | default ".")
            target_revision: ($app.target_revision? | default "main")
            gitops_type: $gitops_type
        }

        $manifest | save $"($output_dir)/($app.name).yaml"
    }
}

# Cluster management functions
export def cluster_name [] {
    $env.CLUSTER_NAME? | default $DEFAULT_CLUSTER
}

export def "main cluster info" [] {
    {
        name: (cluster_name)
        context: (kubectl config current-context)
        config: (load_kcl_config | get config)
        applications_count: (get_applications | length)
    }
}

# Application management commands
export def "main app list" [] {
    get_applications | select name namespace chart? helm_repo? target_revision?
}

export def "main app show" [name] {
    get_app $name
}

export def "main app install" [
    name
    --wait = false
    --dry-run = false
    --values = ""
] {
    helm_install_app $name --wait=$wait --dry-run=$dry_run --values=$values
}

export def "main app deploy" [
    ...apps
    --wait = true
    --parallel = false
] {
    if ($apps | is-empty) {
        let all_apps = get_applications | get name
        deploy_applications $all_apps --wait=$wait --parallel=$parallel
    } else {
        deploy_applications $apps --wait=$wait --parallel=$parallel
    }
}

# KCL integration commands
export def "main kcl execute" [
    script
    --config = ""
    --output = "yaml"
    ...settings
] {
    let settings_record = $settings | reduce --fold {} {|item, acc|
        let parts = $item | split column "=" key value
        $acc | insert $parts.key.0 $parts.value.0
    }

    execute_kcl $script --config=$config --output=$output --settings=$settings_record
}

export def "main gitops generate" [
    --output-dir = "manifests/gitops"
    --type = "flux"
] {
    generate_gitops_manifests --output-dir=$output_dir --gitops-type=$type
}

# Repository management
export def "main repo add" [name, url] {
    helm_repo_manager "add" $name $url
}

export def "main repo update" [] {
    helm_repo_manager "update"
}

export def "main repo list" [] {
    helm_repo_manager "list"
}

export def "main repo remove" [name] {
    helm_repo_manager "remove" $name
}

# Utility functions
export def "main delete temp_files" [] {
    rm --force .env
    rm --force kubeconfig*.yaml
}

def --env "main get github" [--enable-org = true] {
    mut github_token = ""
    if "GITHUB_TOKEN" in $env {
        $github_token = $env.GITHUB_TOKEN
    } else if "REGISTRY_PASSWORD" in $env {
        $github_token = $env.REGISTRY_PASSWORD
    } else {
        $github_token = input $"(ansi green_bold)Enter GitHub token:(ansi reset) "
    }
    $"export GITHUB_TOKEN=($github_token)\n" | save --append .env

    mut github_org = ""
    if $enable_org {
        if "GITHUB_ORG" in $env {
            $github_org = $env.GITHUB_ORG
        } else if "REGISTRY_USER" in $env {
            $github_org = $env.REGISTRY_USER
        } else {
            $github_org = input $"(ansi green_bold)Enter GitHub user or organization:(ansi reset) "
        }
        $"export GITHUB_ORG=($github_org)\n" | save --append .env
    }

    {org: $github_org, token: $github_token}
}

# Health check functions
export def "main health check" [] {
    let apps = get_applications
    $apps | each {|app|
        let status = try {
            helm status $app.name --namespace $app.namespace | str trim
        } catch {
            "NOT_INSTALLED"
        }

        {
            name: $app.name
            namespace: $app.namespace
            status: $status
        }
    }
}

# Configuration validation
export def "main config validate" [] {
    try {
        let config = load_kcl_config
        print $"✓ KCL configuration loaded successfully"
        print $"✓ Found ($config.applications | length) applications"
        print $"✓ Configuration is valid"
        true
    } catch {|e|
        print $"✗ Configuration validation failed: ($e.msg)"
        false
    }
}
