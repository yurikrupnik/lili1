#!/usr/bin/env nu

# Generate FluxCD folder structure with individual files
def main [] {
    print "🚀 Generating FluxCD folder structure..."
    
    # Run KCL to get the folder structure
    print "🔧 Running KCL generation..."
    let kcl_output = (kcl scripts/kcl/apps/flux_folders.k -Y scripts/kcl/apps/kcl.yaml)
    
    if ($kcl_output | is-empty) {
        print "❌ KCL command returned empty output"
        return
    }
    
    let folder_data = ($kcl_output | from yaml)
    
    if ($folder_data == null) {
        print "❌ Failed to parse YAML from KCL output"
        return
    }
    
    # Create base directory if it doesn't exist
    mkdir clusters/second-cluster
    
    # Check if folder_structure exists and handle null case
    if ($folder_data.folder_structure? == null) or ($folder_data.folder_structure | is-empty) {
        print "❌ No folder structure data found. Check KCL configuration."
        return
    }
    
    # Process each app folder
    for app_folder in ($folder_data.folder_structure | columns) {
        print $"📁 Creating folder: ($app_folder)"
        mkdir $app_folder
        
        let app_data = ($folder_data.folder_structure | get $app_folder)
        let resources = ($app_data.resources)
        let kustomization = ($app_data.kustomization)
        
        # Create individual resource files
        mut resource_index = 0
        for resource in $resources {
            let filename = match $resource.kind {
                "Namespace" => "namespace.yaml"
                "HelmRepository" => "helm-repository.yaml" 
                "HelmRelease" => "helm-release.yaml"
                "GitRepository" => "git-repository.yaml"
                "Kustomization" => "kustomization-resource.yaml"
                "Job" => "cert-manager-crds.yaml"
                _ => $"resource-($resource_index).yaml"
            }
            
            let filepath = ($app_folder | path join $filename)
            ($resource | to yaml) | save $filepath -f
            print $"  📄 Created: ($filepath)"
            $resource_index = ($resource_index + 1)
        }
        
        # Create kustomization.yaml file
        let kustomization_path = ($app_folder | path join "kustomization.yaml")
        ($kustomization | to yaml) | save $kustomization_path -f
        print $"  📄 Created: ($kustomization_path)"
    }
    
    print "✅ FluxCD folder structure generated successfully!"
}