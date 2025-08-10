# use config.nu

def "main" [] {
    #let cluster_name = (config cluster_name)
    print "📦 Deleting Kind cluster..."
    # let config = open https://raw.githubusercontent.com/yurikrupnik/gitops/main/cluster/cluster.yaml
    kind create cluster
    kubectl cluster-info --context kind-kind
}

def "main delete" [] {
    #let cluster_name = (config cluster_name)
    print "📦 Deleting Kind cluster..."
    kind delete cluster
    #kind delete cluster --name $cluster_name
}
