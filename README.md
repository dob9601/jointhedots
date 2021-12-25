# Join The Dots
```
jointhedots 
A simple git-based dotfile manager written entirely in Rust!

USAGE:
    jtd <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    help       Print this message or the help of the given subcommand(s)
    install    Install a specified JTD repository
    sync       Sync the currently installed JTD repository with the provided remote repo.
```

## About

jointhedots works by reading a "jtd.yaml" manifest file located within your dotfile repository. The manifest contains a mapping of file to installed location (amongst other things), allowing for JTD to automatically install configurations. `pre_install` and `post_install` commands can also be specified, allowing for additional control over installation.

**WARNING: Be very careful about installing dotfiles via untrusted manifests. The pre_install and post_install blocks allow for (potentially malicious) code execution**

## Roadmap
- Don't overwrite existing configs unless `--force` flag specified
- Allow for installation of specific configs as opposed to all of them
- Prevent syncing when the local dotfiles are from an older version of the repo available upstream

## Example

An example manifest file can be found [here]()

## Download

Grab the latest version [here](https://github.com/dob9601/jointhedots/releases/latest/download/jointhedots)!
