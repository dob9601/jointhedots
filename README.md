# Jointhedots

[![Release](https://github.com/dob9601/jointhedots/actions/workflows/release.yml/badge.svg)](https://github.com/dob9601/jointhedots/actions/workflows/release.yml)
[![Test](https://github.com/dob9601/jointhedots/actions/workflows/test.yml/badge.svg)](https://github.com/dob9601/jointhedots/actions/workflows/test.yml)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://makeapullrequest.com)

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

## Contents
- [About](#about)
- [Roadmap](#roadmap)
- [Installation](#installation)
- [Example Manifest](#example-manifest)
- [FAQ](#faq)

## About
![Git log example](https://user-images.githubusercontent.com/24723950/160243228-5dce7b66-1c1b-4a7b-96a2-a2bf10feb0d1.png)

jointhedots works by reading a "jtd.yaml" manifest file located within your dotfile repository. The manifest contains a mapping of file to installed location (amongst other things), allowing for JTD to automatically install configurations. `pre_install` and `post_install` commands can also be specified, allowing for additional control over installation.

jtd also allows for pushing your dotfiles back to the remote repo and resolves merge conflicts via git.

These install steps are designed so that they will run once on your first install, store a hash of the steps run and then only run if the hash differs (i.e. you have modified your config with new install steps).

*WARNING:* Be very careful about installing dotfiles via untrusted manifests. The pre\_install and post\_install blocks allow for (potentially malicious) code execution**. JTD will prompt you to confirm you trust a manifest if it contains install steps.

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
| Don't allow `jtd install` if dotfiles are behind remote main (prompt user to sync)   |             |                    |

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

## Example Manifest

An example manifest file is shown below:
```yaml
nvim:
  pre_install:
    - mkdir -p ~/Applications
    - curl -sL -o /tmp/nvim.tar.gz https://github.com/neovim/neovim/releases/latest/download/nvim-linux64.tar.gz
    - tar -xvf /tmp/nvim.tar.gz -C ~/Applications
    - rm /tmp/nvim.tar.gz
    - ln -rfs ~/Applications/nvim-linux64/bin/nvim ~/.local/bin/vim
  file: init.vim
  target: ~/.config/nvim/init.vim

kitty:
  file: kitty.conf
  target: ~/.config/kitty/kitty.conf

kitty-theme:
  file: theme.conf
  target: ~/.config/kitty/theme.conf

fish:
  file: config.fish
  target: ~/.config/fish/config.fish
  post_install:
    - git clone --depth 1 https://github.com/junegunn/fzf.git ~/.fzf
    - ~/.fzf/install --all
```
The manifest file should be located in the root of the repository and called "jtd.yaml".

A JSON Schema for the manifest is available [here](https://github.com/dob9601/jointhedots/blob/master/src/dotfile_schema.json). This can be used in conjunction with certain plugins to provide language server support for jtd manifests.

## FAQ

*Q: The different platforms I use require differing installation steps, can I target multiple platforms?*

A: Yes! You can write a different manifest for each platform and specify the manifest to use with the `--manifest` flag

*Q: Can jointhedots handle secrets*

A: Yes, you could store your secrets as encrypted files in the repository along with a `post_install` step to decrypt them, I'd advise against doing this in a public dotfile repository though.
