# use config.nu
use generic.nu *

def "main" [] {
    let cluster_name = (cluster_name)
    print "📦 Creating Kind cluster: '$cluster_name'..."
    let config = http get https://raw.githubusercontent.com/yurikrupnik/gitops/main/cluster/cluster.yaml
    kind create cluster --name $cluster_name
    kubectl cluster-info --context kind-kind
}

def "main delete" [] {
    let cluster_name = (cluster_name)
    print "📦 Deleting Kind cluster: ($cluster_name)..."
    #kind delete cluster
    kind delete cluster --name $cluster_name
}
