<div align="center">
    <h1>Keycli</h1>
    <h3>An environment manager which stores your secrets in your OS keyring</h3>
</div>

# Summary
`keycli` is designed to replace all secrets which are stored in .env files everywhere on your system.

It enables you to store secrets from per project config files and / or cli flags to your OS keyring and to load them into your environment.

For this tool to work properly, you need to have an already working OS keyring.

# Install
`cargo install keycli`

# Usage
```console
$ keycli --help
A env manager which stores your secrets in your OS keyring

Usage: keycli [OPTIONS] <COMMAND>

Commands:
  load        Load secrets to the environment
  unload      Unload the environment
  save        Save secrets to the keyring
  clear       Clear the keyring
  exec        Execute a command with env vars
  shell       Execute a shell with env vars
  init        Create a .keycli.conf from secrets and / or a keycli.tpl
  alias       Generate shell aliases
  completion  Generate shell completion scripts
  help        Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  Turn on verbose output [env: KEYCLI_VERBOSE=]
  -h, --help     Print help
  -V, --version  Print version

Examples:

# Create a .keycli.conf from a keycli.tpl and populate your keyring
keycli init

# Create a .keycli.conf from scratch and populate your keyring
keycli init -a my_app -s PASS -s PASS2 -s PASS3:another_app

# Run a shell with declared env vars
keycli shell

# Load env vars
eval $(keycli load) # Or keycli-load if you installed the alias

# Unload env vars
eval $(keycli unload) # Or keycli-unload if you installed the alias

# Save vars without .keycli.conf file
keycli save -a custom_app -s ZOZO -s ZAZA

# Load vars without .keycli.conf file
keycli load -a custom_app -s ZOZO -s ZAZA

# Install completions and aliases
keycli alias zsh >> ~/.zshrc
keycli completion zsh > ~/.zfunc/_keycli
keycli completion zsh keycli-load > ~/.zfunc/_keycli-load
keycli completion zsh keycli-unload > ~/.zfunc/_keycli-unload

```

Two files are important for `keycli`:
  - `keycli.tpl` is a file meant to be commited and declares the environment variables needed for the project and a suggestion of their paths in the keyring
  - `.keycli.conf` is a file to be kept local and declares the environment variables and paths in your keyring. It will be linked to a version of a `keycli.tpl` if generated with `keycli init`

`.keycli.conf` can be used without `keycli.tpl`.
`keycli.tpl` is only here to suggest variables and keyring paths and to provide `keycli` a mechanism to alert the user if the project requirement changed in terms of environment variables.

Both files shares the same format: 1 secret per line in the form `MY_ENV_VAR:my_app/my_secret_name`.
Lines starting with `#` are ignored.

# Examples
## Custom
> keycli.tpl
```raw
PASS:app/pass
KEY:app/key
```

```bash
$ keycli init
The secret full path is: 'PASS:app/pass'? yes
Input the value of 'PASS:app/pass': [hidden]
INFO PASS was saved to keycli/app/pass
The secret full path is: 'KEY:app/key'? yes
Input the value of 'KEY:app/key': [hidden]
INFO KEY was saved to keycli/app/key
$ keycli exec --env | rg 'PASS|KEY'
KEY=zozo
PASS=zaza
```

## Mise
> keycli.tpl
```raw
PASS:app/pass
KEY:app/key
```

> load_env.sh
```bash
#!/bin/bash
eval $(keycli load)
```

> mise.toml
```toml
[env]
_.source = "./load_env.sh"
```

```bash
$ keycli init
The secret full path is: 'PASS:app/pass'? yes
Input the value of 'PASS:app/pass': [hidden]
INFO PASS was saved to keycli/app/pass
The secret full path is: 'KEY:app/key'? yes
Input the value of 'KEY:app/key': [hidden]
INFO KEY was saved to keycli/app/key
$ mise env | rg 'PASS|KEY'
KEY=zozo
PASS=zaza
```
