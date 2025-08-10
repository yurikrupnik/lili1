import k8s.api.apps.v1 as apps
import k8s.api.core.v1 as core
import k8s.api.networking.v1 as networking
import k8s.apimachinery.pkg.apis.meta.v1 as metav1

schema AppConfig:
    name: str
    image: str
    tag: str
    namespace?: str = "default"
    ingress?: str
    replicas?: int = 1
    port?: int = 8080
    serviceType?: str = "ClusterIP"
    env?: {str:str} = {}
    labels?: {str:str} = {}
    app_type?: str = "backend"


create_app = lambda app: AppConfig {
    dev = apps.Deployment {
        metadata = {
            name = app.name
            namespace = app.namespace
            labels = app.labels
        }
        spec = {
            #replicas = 1
            selector = {
                matchLabels = {
                    app = app.name
                }
            }
            template = {
                metadata = {
                    name = app.name
                    labels = {
                        app = app.name
                        version = "v1"
                    }
                }
                spec = {
                    restartPolicy: "Always"
                    containers = [
                        {
                            name: app.name
                            image: app.image
                            imagePullPolicy: "IfNotPresent"
                            resources = {
                              requests = {
                                memory: "32Mi"
                                cpu: "150m"
                              }
                              limits = {
                                 memory: "64Mi"
                                 cpu: "100m"
                              }
                            }

                            ports = [
                                {
                                    containerPort = app.port
                                    protocol = "TCP"
                                    name = "http"
                                }
                            ]
                            env = [core.EnvVar {
                                name = k
                                value = v
                            } for k, v in app.env] if app.env else []
                        }
                    ]
                }
            }
        }
    }

    service = core.Service {
        metadata: {
            name: app.name
        }
        spec: {
            selector: {
                app: app.name
            }
            ports: [
                {
                    protocol: "TCP"
                    port= app.port
                    targetPort= app.port
                }
            ]
            $type: "ClusterIP"
        }
    }
    # Generate Deployment
    # Generate Ingress (conditional)
    ingress = networking.Ingress {
        apiVersion = "networking.k8s.io/v1"
        kind = "Ingress"
        metadata = metav1.ObjectMeta {
            name = "airs"
            namespace = "ars"
            labels = {
                app = "dam"
            }
            annotations = {
                "kubernetes.io/ingress.class" = "nginx"
                "nginx.ingress.kubernetes.io/rewrite-target" = "/"
            }
        }
    }
    [dev, service, ingress]
}

apds = AppConfig {
    name = "zerg-api"
    image = "yurikrupnik/zerg-api"
    tag = "sha-caec71c"
    namespace: "dev"
    replicas: 2
}

create_app_result = create_app(apds)
