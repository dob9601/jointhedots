# Join The Dots

![Demo](https://user-images.githubusercontent.com/24723950/152683893-eca67fa3-96bd-4c79-9cf4-a1283a73b61d.gif)
```
jointhedots 
A simple git-based dotfile manager written entirely in Rust!

USAGE:
    jtd <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    help           Print this message or the help of the given subcommand(s)
    install        Install a specified JTD repository
    interactive    Interactively install dotfiles
    sync           Sync the currently installed JTD repository with the provided remote repo.
```

## About

jointhedots works by reading a "jtd.yaml" manifest file located within your dotfile repository. The manifest contains a mapping of file to installed location (amongst other things), allowing for JTD to automatically install configurations. `pre_install` and `post_install` commands can also be specified, allowing for additional control over installation.

**WARNING: Be very careful about installing dotfiles via untrusted manifests. The pre_install and post_install blocks allow for (potentially malicious) code execution**. JTD will prompt you to confirm you trust a manifest if it contains install steps.

## Roadmap
| Feature                                                                              | Implemented |       Notes        |
| :---                                                                                 |    :---:    | :---               |
| Sync local changes to dotfiles with remote repo                                      |      ✔      |                    |
| Interactive mode                                                                     |      ✔      |                    |
| Selectively install only some dotfiles                                               |      ✔      |                    |
| JSON Schema for manifest files                                                       |      ✔      |                    |
| Host latest version somewhere that can be curled                                     |      ✔      | `jtd.danielobr.ie` |
| Selectively sync only some dotfile changes                                           |      ✔      |                    |
| Use `git2` as opposed to `Command::new("git")`                                       |      ✔      |                    |
| Prevent syncing when the local dotfiles are from an older version of the remote repo |             |                    |
| Ability to specify which manifest to use in (multiple manifest support)              |      ✔      |                    |
| Support for non-GitHub/GitLab repos                                                  |             |                    |
| Ability to manually specify commit message for JTD sync                              |      ✔      |                    |
| More detailed default commit messages for JTD sync (list the changed files)          |      ✔      |                    |
| Abort syncing if no changes are present in files                                     |             |                    |

## Example

An example manifest file is shown below:
```yaml
nvim:
  file: init.vim
  pre_install:
    - sudo apt install nvim
  target: ~/.config/nvim/init.vim

nvim-coc:
  file: coc-settings.json
  target: ~/.config/nvim/coc-settings.json

kitty:
  file: kitty.conf
  target: ~/.config/kitty/kitty.conf

kitty-theme:
  file: theme.conf
  target: ~/.config/kitty/theme.conf

fish:
  file: config.fish
  target: ~/.config/fish/config.fish
```
The manifest file should be located in the root of the repository and called "jtd.yaml".

A JSON Schema for the manifest is available [here](https://github.com/dob9601/jointhedots/blob/master/src/dotfile_schema.json). This can be used in conjunction with certain plugins to provide language server support for jtd manifests.

## FAQ

*Q: The different platforms I use require differing installation steps, can I target multiple platforms?*

A: Yes! You can write a different manifest for each platform and specify the manifest to use with the `--manifest` flag

## Installation

### Manual
Grab the latest version [here](https://github.com/dob9601/jointhedots/releases/latest/download/jtd) (for x86-64, more targets on the way!)
### Cargo
Install via cargo:
```sh
cargo install jointhedots
```
### Curl (one-time use)
Use the following 1 liner to 1-off run JTD to install your dotfiles
```sh
curl -sL jtd.danielobr.ie | sh
```
