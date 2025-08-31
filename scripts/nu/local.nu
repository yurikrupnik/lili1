#!/usr/bin/env nu
use cluster.nu
source ~/configs-files/scripts/yurik.nu
#use ~/configs-files/scripts/yurik.nu


def "main run" [] {
  cluster create
  main apply cnpg
}
def "main app logsa" [app: string]: nothing -> string {
    # Takes app name, returns log text
    kubectl logs $app
}
def "main apply cnpg" [] {

     print $"\nInstalling (ansi yellow_bold)Cloud-Native PostgreSQL \(CNPG\)(ansi reset)...\n"

    (
        helm upgrade --install cnpg cloudnative-pg
            --repo https://cloudnative-pg.github.io/charts
            --namespace cnpg-system --create-namespace --wait
    )

}
