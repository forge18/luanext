# LuaNext Neovim Plugin

Language support for LuaNext in Neovim with LSP integration.

## Features

- Syntax highlighting for `.luax` files
- LSP integration with `luanext-lsp`
- Auto-completion, go-to-definition, find references
- Inline diagnostics and type hints
- Code formatting

## Installation

### Using [lazy.nvim](https://github.com/folke/lazy.nvim)

```lua
{
  "luanext",
  dir = "path/to/luanext/editors/nvim",
  ft = "luanext",
  config = function()
    require("luanext").setup({
      -- Path to luanext-lsp executable
      cmd = { "luanext-lsp" },

      -- LSP server settings
      settings = {
        luanext = {
          checkOnSave = true,
          strictNullChecks = true,
          format = {
            enable = true,
            indentSize = 4,
          },
          inlayHints = {
            typeHints = true,
            parameterHints = true,
          },
        },
      },
    })
  end,
}
```

### Using [packer.nvim](https://github.com/wbthomason/packer.nvim)

```lua
use {
  'luanext',
  config = function()
    require('luanext').setup()
  end
}
```

### Manual Installation

1. Copy the `nvim` directory to your Neovim config directory:
   ```bash
   cp -r editors/nvim ~/.config/nvim/pack/plugins/start/luanext
   ```

2. Add to your `init.lua`:
   ```lua
   require('luanext').setup()
   ```

## Configuration

Default configuration:

```lua
require('luanext').setup({
  -- Path to luanext-lsp executable
  cmd = { 'luanext-lsp' },

  -- Filetypes to attach to
  filetypes = { 'luanext' },

  -- Root directory patterns
  root_dir = function(fname)
    return require('lspconfig.util').root_pattern('luanext.config.yaml', '.git')(fname)
  end,

  -- LSP server settings
  settings = {
    luanext = {
      checkOnSave = true,
      strictNullChecks = true,
      format = {
        enable = true,
        indentSize = 4,
      },
      inlayHints = {
        typeHints = true,
        parameterHints = true,
      },
    },
  },

  -- Additional capabilities (auto-filled from nvim-cmp if available)
  capabilities = nil,

  -- On attach callback
  on_attach = nil,
})
```

## Requirements

- Neovim >= 0.8.0
- `luanext-lsp` executable in PATH or configured via `cmd` option
- Optional: [nvim-cmp](https://github.com/hrsh7th/nvim-cmp) for auto-completion
- Optional: [null-ls.nvim](https://github.com/jose-elias-alvarez/null-ls.nvim) for additional formatting

## Usage

Once installed, the plugin will automatically activate for `.luax` files.

### Commands

- `:LuaNextRestart` - Restart the language server
- `:LuaNextInfo` - Show language server information

### Key Mappings

The plugin uses standard LSP key mappings. Example configuration:

```lua
vim.api.nvim_create_autocmd('LspAttach', {
  group = vim.api.nvim_create_augroup('UserLspConfig', {}),
  callback = function(ev)
    local opts = { buffer = ev.buf }
    vim.keymap.set('n', 'gD', vim.lsp.buf.declaration, opts)
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, opts)
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, opts)
    vim.keymap.set('n', 'gi', vim.lsp.buf.implementation, opts)
    vim.keymap.set('n', '<C-k>', vim.lsp.buf.signature_help, opts)
    vim.keymap.set('n', '<space>rn', vim.lsp.buf.rename, opts)
    vim.keymap.set('n', '<space>ca', vim.lsp.buf.code_action, opts)
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, opts)
    vim.keymap.set('n', '<space>f', function()
      vim.lsp.buf.format { async = true }
    end, opts)
  end,
})
```

## Troubleshooting

### Language server not starting

1. Check that `luanext-lsp` is in your PATH:
   ```bash
   which luanext-lsp
   ```

2. Check LSP logs:
   ```vim
   :LspLog
   ```

3. Verify the plugin is loaded:
   ```vim
   :LuaNextInfo
   ```

### No syntax highlighting

Run `:set filetype?` in a `.luax` file to verify the filetype is set correctly.

## License

MIT
