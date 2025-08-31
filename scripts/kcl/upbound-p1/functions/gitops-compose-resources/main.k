
import models.io.k8s.api.apps.v1 as appsv1
import models.io.k8s.api.core.v1 as corev1
import models.io.k8s.api.networking.v1 as networkingv1
import models.com.example.gitops.v1alpha1 as gitopsv1alpha1

oxr = gitopsv1alpha1.Gitops{**option("params").oxr} # observed claim
_ocds = option("params").ocds # observed composed resources
_dxr = option("params").dxr # desired composite resource
dcds = option("params").dcds # desired composed resources

_metadata = lambda name: str -> any {
    { annotations = { "krm.kcl.dev/composition-resource-name" = name }}
}

_labels = {
   "app.kubernetes.io/name": oxr.metadata.name
   #app: oxr.metadata.name
}
_desired_deployment = appsv1.Deployment{
    metadata: _metadata("deployment") | {
        name: oxr.metadata.name
        namespace: oxr.metadata.namespace
        labels: {
            "app.kubernetes.io/name": oxr.metadata.name
        }
    }
    spec: {
        replicas: int(oxr.spec.parameters.replicas)
        selector: {
            matchLabels: _labels | {
              app: oxr.metadata.name
            }
        }
        template: {
            metadata: {
                labels: _labels | {
                  app: oxr.metadata.name
                }
            }
            spec: {
                serviceAccountName: oxr.spec.parameters.serviceAccount
                containers: [{
                    name: oxr.metadata.name
                    image: oxr.spec.parameters.image
                    imagePullPolicy: "Always"
                    ports: [{
                        name: "http"
                        containerPort: int(oxr.spec.parameters.port)
                        protocol: "TCP"
                    }]
                    resources: {
                        requests: {
                            memory: oxr.spec.parameters.resources.requests.memory
                            cpu: oxr.spec.parameters.resources.requests.cpu
                        }
                        limits: {
                            memory: oxr.spec.parameters.resources.limits.memory
                            cpu: oxr.spec.parameters.resources.limits.cpu
                        }
                    }
                }]
                restartPolicy: "Always"
            }
        }
    }
}

observed_deployment = option("params").ocds["deployment"]?.Resource
if any_true([c.type == "Available" and c.status == "True" for c in observed_deployment?.status?.conditions or []]):
  _desired_deployment.metadata.annotations["krm.kcl.dev/ready"] = "True"

if oxr.spec.parameters.service.enabled:
    _desired_service = corev1.Service{
        metadata: _metadata("service") | {
            name: oxr.metadata.name
            namespace: oxr.metadata.namespace
        }
        spec: {
            selector: {
                app: oxr.metadata.name
            }
            ports: [{
                name: "http"
                protocol: "TCP"
                port: 80
                targetPort: "http"
            }]
        }
    }

observed_service = option("params").ocds["service"]?.Resource
if observed_service?.spec?.clusterIP:
  _desired_service.metadata.annotations["krm.kcl.dev/ready"] = "True"

if oxr.spec.parameters.ingress.enabled:
    _desired_ingress = networkingv1.Ingress{
        metadata: _metadata("ingress") | {
            name: oxr.metadata.name
            namespace: oxr.metadata.namespace
            annotations: {
                "kubernetes.io/ingress.class": "alb"
                "alb.ingress.kubernetes.io/scheme": "internet-facing"
                "alb.ingress.kubernetes.io/target-type": "ip"
                "alb.ingress.kubernetes.io/healthcheck-path": "/health"
                "alb.ingress.kubernetes.io/listen-ports": '[{"HTTP": 80}]'
                "alb.ingress.kubernetes.io/target-group-attributes": "stickiness.enabled=true,stickiness.lb_cookie.duration_seconds=60"
            }
        }
        spec: {
            rules: [{
                http: {
                    paths: [{
                        path: "/"
                        pathType: "Prefix"
                        backend: {
                            service: {
                                name: oxr.metadata.name
                                port: {
                                    number: 80
                                }
                            }
                        }
                    }]
                }
            }]
        }
    }


observed_ingress = option("params").ocds["ingress"]?.Resource
if observed_ingress?.status?.loadBalancer?.ingress?[0]?.hostname:
  _desired_ingress.metadata.annotations["krm.kcl.dev/ready"] = "True"

_desired_xr = {
  **option("params").dxr
  status.availableReplicas = observed_deployment?.status?.availableReplicas or 0
  status.url = observed_ingress?.status?.loadBalancer?.ingress?[0]?.hostname or ""
}


items = [
    _desired_deployment,
    _desired_service,
    _desired_ingress,
    _desired_xr
]
