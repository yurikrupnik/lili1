const DEFAULT_CLUSTER = "my-cluster"

# Cluster management functions
export def cluster_ass [] {
    $env.CLUSTER_NAME? | default $DEFAULT_CLUSTER
}

# Cluster management functions
export def cluster_name [] {
    $env.CLUSTER_NAME? | default $DEFAULT_CLUSTER
}
