# xbps-lsp

`xbps-lsp` is an LSP (language server protocol) server for XBPS package templates.

Features:
- Auto-completions (kind of barebones, but work)
- Variable/method documentation on hover
- `xlint` support (xbps template linter from [xtools](https://github.com/leahneukirchen/xtools)) 
- `update-check` support: shows if there's a new version available

Requirements:
- `xlint` must be installed

## Installation
```sh
git clone https://github.com/bnyro/xbps-lsp && cd xbps-lsp
cargo install --path .
```

## LSP configuration
### Helix Editor

Add the following to `~/.config/helix/languages.toml`:
```toml
[[language]]
name = "xbps-template"
scope = "source.bash"
grammar = "bash"
comment-token = "#"
file-types = [{ glob = "template" }]
language-servers = ["xbps-lsp"]

[language-server.xbps-lsp]
command = "xbps-lsp"
```

### Other editors
Your LSP client should call `xbps-lsp` (without arguments), then it should work out of the box.

## Credits
- The Void Linux [manual](https://github.com/void-linux/void-packages/blob/master/Manual.md) has awesome explanations on how-to-write XBPS package template. `typos-lsp` sources its documentation from the official manual.
- [`xlint`](https://github.com/leahneukirchen/xtools) was the main motivation to build this tool
