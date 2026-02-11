# Editor Setup

LuaNext provides a Language Server Protocol (LSP) implementation for rich IDE support. This guide covers setting up VS Code and other editors.

## VS Code

LuaNext includes a full-featured VS Code extension.

### Features

- **Syntax Highlighting** — Color-coded syntax for `.luax` files
- **IntelliSense** — Autocomplete for variables, functions, and types
- **Diagnostics** — Real-time type errors and warnings
- **Go to Definition** — Jump to type and function definitions
- **Find References** — Find all usages of a symbol
- **Hover Information** — View type information on hover
- **Rename Symbol** — Refactor variable and function names
- **Code Formatting** — Auto-format code on save
- **Inlay Hints** — Show inferred types inline
- **Semantic Tokens** — Accurate syntax highlighting based on type information

### Installation

The VS Code extension is included in the LuaNext repository at `editors/vscode/`.

**Option 1: Install from VSIX (Recommended)**

1. Build the extension:

   ```bash
   cd editors/vscode
   npm install
   npm run package
   ```

2. Install the `.vsix` file:

   ```bash
   code --install-extension luanext-*.vsix
   ```

**Option 2: Development Mode**

1. Open the `editors/vscode` folder in VS Code
2. Press `F5` to launch Extension Development Host
3. Open a `.luax` file in the new window

### Configuration

The extension automatically detects `luanext.config.yaml` in your workspace. You can also configure the LSP server path in VS Code settings:

```json
{
  "luanext.lsp.path": "/path/to/luanext-lsp",
  "luanext.lsp.trace.server": "verbose"
}
```

### Verify It's Working

1. Create a new file `test.luax`
2. Type: `const x: number = "string"`
3. You should see a red underline with error: "expected number, found string"
4. Hover over `x` to see inferred type information

## Neovim

LuaNext LSP works with Neovim's built-in LSP client.

### Using nvim-lspconfig

Install the LuaNext LSP binary:

```bash
cargo install --path crates/luanext-lsp
```

Configure in your `init.lua`:

```lua
local lspconfig = require('lspconfig')

lspconfig.luanext = {
  default_config = {
    cmd = { 'luanext-lsp' },
    filetypes = { 'luax' },
    root_dir = lspconfig.util.root_pattern('luanext.config.yaml', '.git'),
    settings = {},
  },
}

lspconfig.luanext.setup{}
```

Add filetype detection for `.luax` files:

```lua
vim.filetype.add({
  extension = {
    luax = 'luax',
  },
})
```

### Manual Setup

If not using `nvim-lspconfig`, configure manually:

```lua
vim.lsp.start({
  name = 'luanext',
  cmd = { 'luanext-lsp' },
  root_dir = vim.fs.dirname(vim.fs.find({'luanext.config.yaml', '.git'}, { upward = true })[1]),
})
```

## Emacs

Use `lsp-mode` with LuaNext:

```elisp
(require 'lsp-mode)

(add-to-list 'lsp-language-id-configuration '(luax-mode . "luanext"))

(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection '("luanext-lsp"))
  :activation-fn (lsp-activate-on "luax")
  :server-id 'luanext))

(add-hook 'luax-mode-hook #'lsp)
```

Define `luax-mode`:

```elisp
(define-derived-mode luax-mode lua-mode "LuaNext"
  "Major mode for LuaNext files.")

(add-to-list 'auto-mode-alist '("\\.luax\\'" . luax-mode))
```

## Sublime Text

Use LSP package for Sublime Text:

1. Install [LSP](https://packagecontrol.io/packages/LSP) package
2. Configure client in `LSP.sublime-settings`:

```json
{
  "clients": {
    "luanext": {
      "command": ["luanext-lsp"],
      "enabled": true,
      "selector": "source.luax"
    }
  }
}
```

1. Create syntax definition for `.luax` files (copy from Lua syntax)

## JetBrains IDEs (IntelliJ, WebStorm, etc.)

1. Install [LSP Support plugin](https://plugins.jetbrains.com/plugin/10209-lsp-support)
2. Go to Settings → Languages & Frameworks → Language Server Protocol → Server Definitions
3. Add new server:
   - Extension: `luax`
   - Command: `luanext-lsp`

## Generic LSP Client

For editors with LSP support:

- **Command:** `luanext-lsp`
- **File extensions:** `.luax`
- **Root markers:** `luanext.config.yaml`, `.git`
- **Initialization options:** `{}`

## LSP Server Capabilities

The LuaNext LSP provides:

| Capability        | Supported | Notes |
|-------------------|-----------|-------|
| Completion        | ✅        |       |
| Hover             | ✅        |       |
| Signature Help    | ✅        |       |
| Go to Definition  | ✅        |       |
| Find References   | ✅        |       |
| Document Symbols  | ✅        |       |
| Workspace Symbols | ✅        |       |
| Rename            | ✅        |       |
| Formatting        | ✅        |       |
| Code Actions      | ✅        |       |
| Diagnostics       | ✅        |       |
| Semantic Tokens   | ✅        |       |
| Inlay Hints       | ✅        |       |

**Completion Features:**

- ✅ Keyword completion (if, function, class, etc.)
- ✅ Type annotation completion (number, string, etc.)
- ✅ Symbol completion (variables, functions from type checker)
- ✅ Decorator completion (@readonly, @sealed, etc.)
- ✅ Member access completion (`obj.` - properties and methods)
- ✅ Method call completion (`obj:` - Lua method call syntax)
- ✅ Built-in methods for primitive types (string.upper, string.sub, etc.)
- ✅ Array methods (insert, remove, length)

## Troubleshooting

### LSP Not Starting

Check that `luanext-lsp` is in your PATH:

```bash
which luanext-lsp
luanext-lsp --version
```

### No Diagnostics

Ensure you have a `luanext.config.yaml` in your project root, or the LSP may not initialize correctly.

### Slow Performance

For large projects, try disabling strict null checks or reducing the scope of files in `luanext.config.yaml`:

```yaml
include:
  - "src/**/*.luax"

exclude:
  - "**/node_modules/**"
  - "**/dist/**"
```

### Enable LSP Logging

**VS Code:**

```json
{
  "luanext.lsp.trace.server": "verbose"
}
```

View logs in Output → LuaNext Language Server.

**Neovim:**

```lua
vim.lsp.set_log_level("debug")
```

View logs: `:LspLog`

## Next Steps

- [Project Setup](project-setup.md) — Configure `luanext.config.yaml`
- [Language Reference](../language/basics.md) — Learn LuaNext syntax
