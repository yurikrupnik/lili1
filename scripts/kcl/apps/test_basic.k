import main

# Test basic config scenarios

# Dev config - local=true, monitoring=true
dev_config = main.Config {
    local = True
    primary_cloud = "aws"
    area = "frankfurt"
    gitops = "argo"
    monitoring = True
}

# Prod config - local=false, monitoring=true
prod_config = main.Config {
    local = False
    primary_cloud = "gcp"
    area = "tel-aviv"
    gitops = "argo"
    monitoring = True
}

# Test assertions
assert dev_config.local == True
assert dev_config.primary_cloud == "aws"
assert dev_config.area == "frankfurt"
assert dev_config.region == "eu-central-1"

assert prod_config.local == False
assert prod_config.primary_cloud == "gcp"
assert prod_config.area == "tel-aviv"
assert prod_config.region == "me-west1"