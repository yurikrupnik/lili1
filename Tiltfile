k8s_yaml(kustomize('apps/zerg/api/k8s/base'))

docker_build(
  "yurikrupnik/zerg-api",
  ".",
  dockerfile="rust.Dockerfile",
  build_args={"APP_NAME":"zerg_api"},
)

