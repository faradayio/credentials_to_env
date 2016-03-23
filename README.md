# `credentials-to-env`: Fetch secrets from Hashicorp's vault or elsewhere before `exec`ing a program

Do you have a pre-existing program that assumes that it will receive
secrets in either environment variables or files on disk?  Would you like
to convert that program to work with Hashicorp's [Vault][]?

First run:

```sh
cargo install credentials_to_env
```

Then create a file named `Secretfile` explaining where in Vault the
individual secrets can be found:

    # Set environment variables based on Vault secrets.
    DOCKER_HUB_USER secret/docker_hub:user
    DOCKER_HUB_PASSWORD secret/docker_hub:password
    DOCKER_HUB_EMAIL secret/docker_hub:email

    # Create SSL key files based on Vault secrets.
    >$HOME/.docker/ca.pem secret/docker:ca_pem
    >$HOME/.docker/cert.pem secret/docker:cert_pem
    >$HOME/.docker/key.pem secret/docker:key_pem

Finally, prefix the invocation of your program with `credentials-to-env`:

```sh
credentials-to-env myprogram arg1 arg2
```

This will automatically fetch secrets from Vault (or any other backend
supported by [credentials][]) and write them to the specified environment
variables or files.

You can also override `credentials-to-env` by passing in the secrets
yourself, which is handy if you call `credentials-to-env` inside a Docker
container, but want to temporarily override the secrets you'd get from
Vault.

## Building a portable binary

If you want to build a single binary which works on Debian 7.0 and Ubuntu
14.04 (and probably other recent distros), make sure you have Docker
configured, and run:

```sh
cargo clean
make image
make
```

This isn't quite as portable as a typical Go binary, but it should
theoretically work if you have the C library (which is always there) and
you install OpenSSL:

```sh
sudo apt-get install libssl1.0.0
```

[Vault]: https://www.vaultproject.io/
[credentials]: http://docs.randomhacks.net/credentials/
