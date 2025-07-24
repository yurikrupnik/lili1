def "main get provider" [
    --providers = [aws azure google kind upcloud]  # List of cloud providers to choose from
] {

    let message = $"
Right now, only providers listed below are supported in this demo.
Please send an email to (ansi yellow_bold)viktor@farcic.com(ansi reset) if you'd like to add additional providers.

(ansi yellow_bold)Select a provider(ansi green_bold)"

    let provider = $providers | input list $message
    print $"(ansi reset)"

    $"PROVIDER=($provider)\n" | save --append .env

    $provider
}
