local_resource('bun-install', cmd='bun i', deps=['package.json/'], labels=['bun'])
local_resource('nu', cmd='nu ~/configs-files/scripts/generate-shell-configs.nu', labels=['nu'])

k8s_yaml(kustomize('manifests/db'))
k8s_resource("redis-deployment", port_forwards="6379:6379")
k8s_resource("postgres-deployment", port_forwards="5432:5432")
k8s_resource("mongodb-deployment", port_forwards="27017:27017")

#local_resource('bun-install', cmd='bun install', deps=['package.json'], labels=['bun'])

#docker_build(
 # "yurikrupnik/zerg-api",
 # ".",
 # dockerfile="rust.Dockerfile",
 # build_args={"APP_NAME":"zerg_api"},
#)

#k8s_yaml(kustomize('apps/zerg/api/k8s/base'))
