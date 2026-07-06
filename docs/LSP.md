# rtex Language Server (LSP)

rtex includes a Language Server Protocol implementation for `.tex` files. Editors get diagnostics, completions, document outlines, and hover help without an external LaTeX installation.

## Build and run

```bash
cargo build --release --features lsp
./target/release/rtex-lsp
```

The server communicates over **stdio** (standard LSP transport).

## Editor setup

### VS Code / Cursor

Install a generic LSP client extension (e.g. [LSP Client](https://marketplace.visualstudio.com/)) or add to `.vscode/settings.json`:

```json
{
  "latex.lsp.server": "./target/release/rtex-lsp"
}
```

For **Cursor**, add to user `settings.json`:

```json
{
  "lsp.servers": {
    "rtex": {
      "command": ["/absolute/path/to/rtex/target/release/rtex-lsp"],
      "filetypes": ["tex", "latex"]
    }
  }
}
```

(Exact key names depend on your LSP extension; adjust to match the extension you use.)

### Neovim (nvim-lspconfig)

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.rtex then
  configs.rtex = {
    default_config = {
      cmd = { '/absolute/path/to/rtex/target/release/rtex-lsp' },
      filetypes = { 'tex', 'plaintex' },
      root_dir = function(fname)
        return vim.fs.dirname(vim.fs.find('.git', { path = fname, upward = true })[1])
      end,
    },
  }
end

lspconfig.rtex.setup({})
```

## Capabilities

| LSP feature | Description |
|-------------|-------------|
| **Diagnostics** | Unclosed braces, environment mismatches, unclosed `$` math, missing `\end{document}` |
| **Completion** | LaTeX commands and environments (trigger: `\`, `{`) |
| **Document symbols** | Section outline and `\label{...}` entries |
| **Hover** | Short docs for supported commands |

Diagnostics are published on open and on every full-buffer change.

## Library API

Analysis functions are available without the `lsp` feature:

```rust
use rtex::{analyze_diagnostics, completions_at, document_symbols, hover_at};

let text = r"\begin{document}\section{Hi}\end{document}";
let diags = analyze_diagnostics(text);
let outline = document_symbols(text);
```

## Limitations

- Diagnostics are static analysis only (not a full TeX engine).
- Macro expansion is applied before outline parsing; diagnostics scan raw source.
- No go-to-definition or PDF preview yet.
