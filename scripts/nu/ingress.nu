
def main [] {
    #print "🌐 Installing Gateway API..."
    #kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.3.0/standard-install.yaml
    #kubectl wait --for=condition=Established crd/gateways.gateway.networking.k8s.io --timeout=60s
    _main
}

def _main [] {
  istio
}

def istio [] {
  print "🌐 Installing Istio..."
  istioctl install --set values.gateways.istio-ingressgateway.enabled=true --set profile=ambient -y
  kubectl wait --for=condition=Ready pod --all -n istio-system --timeout=300s
  kubectl create namespace aris
  kubectl label namespace aris istio.io/dataplane-mode=ambient
}


