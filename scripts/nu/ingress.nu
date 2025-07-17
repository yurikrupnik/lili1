
def main [] {
    print "🌐 Installing Gateway API..."
    kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.3.0/standard-install.yaml
    kubectl wait --for=condition=Established crd/gateways.gateway.networking.k8s.io --timeout=60s
}
