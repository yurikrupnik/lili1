cluster-create *args:
    nu -c "source ~/private/lili1/scripts/nu/cluster.nu; main create --secrets false {{args}}"


cluster-delete:
    nu -c "source ~/private/lili1/scripts/nu/cluster.nu; main delete"
