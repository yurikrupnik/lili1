#use is

def main [] {
  kind create cluster
  _main
  #istiod
}


# def main [] {
#     #print "🌐 Installing Gateway API..."
#     #kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.3.0/standard-install.yaml
#     #kubectl wait --for=condition=Established crd/gateways.gateway.networking.k8s.io --timeout=60s
#     _main
# }

def _main [] {
  istio
}

# Supported cloud providers (enum-like)
const CLOUD_PROVIDERS = {
    aws: "aws",
    gcp: "gcp",
    local: "local",
    azure: "azure"
}

# Get list of valid provider values
const PROVIDER_VALUES = [$CLOUD_PROVIDERS.aws, $CLOUD_PROVIDERS.gcp, $CLOUD_PROVIDERS.local, $CLOUD_PROVIDERS.azure]

# List available cloud providers
def "main list-providers" [] {
    print "🌩️  Available cloud providers:"
    $PROVIDER_VALUES | each {|provider| print $"  • ($provider)"}
}

def istio [] {
  print "🌐 Installing Istio..."
  istioctl install --set values.gateways.istio-ingressgateway.enabled=true --set profile=ambient -y
  kubectl wait --for=condition=Ready pod --all -n istio-system --timeout=300s
  kubectl create namespace aris
  kubectl label namespace aris istio.io/dataplane-mode=ambient
}
def "main down" [] {
  kind delete cluster
  #kubectl create namespace aris
  #kubectl label namespace aris istio.io/dataplane-mode=ambient
}
