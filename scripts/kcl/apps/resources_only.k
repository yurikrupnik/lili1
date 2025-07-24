import gitops

config = option("config") or {}
applications = option("applications") or []
enabled_applications = [app for app in applications if app.enabled]

# Output only the resources as items
items = gitops.get_gitops_resources(config.gitops, config, enabled_applications)
