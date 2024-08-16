RPK
---

A lightweight, cross-platform cli package manager.

### Installation

Download the latest release from the [releases page](https://github.com/wfxr/rpk/releases)
or just run the following command:

```bash
curl -fsSL https://raw.githubusercontent.com/wfxr/rpk/main/scripts/install \
    | bash -s -- --to ~/bin
```

You can port the configuration from remote at the same time by specifying the `--init` option:

```bash
# Useful when you want to quickly install `rpk` and restore packages on a new environment (like container).
curl -fsSL https://raw.githubusercontent.com/wfxr/rpk/main/scripts/install \
    | bash -s -- --to ~/bin --init https://raw.githubusercontent.com/wfxr/rpk/main/demo/packages.toml
```

### Usage

To access the packages installed by rpk, you need to run the following command or add it to your shell rc file:

```bash
source <(rpk env)
```

There are several ways to add a package using `rpk`. The most simple way is to using `rpk add`:

```
$ rpk add sharkdp/fd
```

If you don't remember the exact repo name, you can run `rpk find <keyword>` and install it interactively:

```
$ rpk find ripgrep
? Select a package
> ★ 47026    BurntSushi/ripgrep                    ripgrep recursively searches directories for a regex pattern while respecting your gitignore
  ★ 6478     phiresky/ripgrep-all                  rga: ripgrep, but also search in PDFs, E-Books, Office documents, zip, tar.gz, etc.
  ★ 540      jremmen/vim-ripgrep                   Use RipGrep in Vim and display results in a quickfix list
  ★ 317      learnbyexample/learn_gnugrep_ripgrep  Example based guide to mastering GNU grep and ripgrep
  ★ 108      microsoft/vscode-ripgrep              For consuming the ripgrep binary from microsoft/ripgrep-prebuilt in a Node project
  ★ 468      dajva/rg.el                           Emacs search tool based on ripgrep
  ★ 711      Wilfred/deadgrep                      fast, friendly searching with ripgrep and Emacs
  ★ 101      cosmicexplorer/helm-rg                ripgrep is nice
  ★ 1413     Gruntfuggly/todo-tree                 Use ripgrep to find TODO tags and display the results in a tree view
  ★ 149      chinanf-boy/ripgrep-zh                中文翻译:<BurntSushi/ripgrep> 一个面向行的搜索工具 ❤️  校对 ✅
[↑↓ to move, enter to select, type to filter]
```

You can also edit the configuration file directly and run `rpk sync`. The config file is located at `~/.config/rpk/packages.toml` by default. Here is a [sample](demo/packages.toml):

```toml
# You can manage rpk by itself
[pkgs.rpk]
repo = "wfxr/rpk"
desc = "A lightweight, cross-platform cli package manager."

[pkgs.fzf]
repo = "junegunn/fzf"
desc = ":cherry_blossom: A command-line fuzzy finder"

[pkgs.fd]
repo = "sharkdp/fd"
desc = "A simple, fast and user-friendly alternative to 'find'"

[pkgs.rg]
repo = "BurntSushi/ripgrep"
desc = "ripgrep recursively searches directories for a regex pattern while respecting your gitignore"

[pkgs.eza]
repo = "eza-community/eza"
desc = "A modern alternative to ls"

[pkgs.bat]
repo = "sharkdp/bat"
desc = "A cat(1) clone with wings."
```

### Credits

`rpk` is inspired by [sheldon](https://github.com/rossmacarthur/sheldon), an awesome shell plugin manager I'm currently using.

### License

`rpk` is distributed under the terms of both the MIT License and the Apache License 2.0.

See the [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) files for license details.
