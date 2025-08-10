import gitops
import config

# Get the configuration
_gitops = config._config.gitops

# Use the same app loading logic as resources_only.k
_apps: [gitops.App] = option("applications") or []
a = [app for app in _apps if app.enabled]

# Generate folder structure only for flux
folder_structure = gitops.generate_flux_folder_structure(a) if _gitops == "flux" else {}
