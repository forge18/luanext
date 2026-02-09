-- LuaNext Neovim Plugin
-- Language support for LuaNext with LSP integration

local M = {}

local default_config = {
  -- Path to luanext-lsp executable
  cmd = { 'luanext-lsp' },

  -- Filetypes to attach to
  filetypes = { 'luanext' },

  -- Root directory patterns
  root_dir = nil, -- Will be set in setup if not provided

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
}

-- Merge user config with defaults
local function merge_config(user_config)
  user_config = user_config or {}
  local config = vim.tbl_deep_extend('force', default_config, user_config)

  -- Set root_dir if not provided
  if not config.root_dir then
    local lspconfig_util = require('lspconfig.util')
    config.root_dir = lspconfig_util.root_pattern('luanext.config.yaml', '.git')
  end

  -- Auto-detect capabilities from nvim-cmp if available and not provided
  if not config.capabilities then
    local has_cmp_lsp, cmp_lsp = pcall(require, 'cmp_nvim_lsp')
    if has_cmp_lsp then
      config.capabilities = cmp_lsp.default_capabilities()
    end
  end

  return config
end

-- Setup the LuaNext LSP client
function M.setup(user_config)
  local config = merge_config(user_config)

  -- Register the LSP client configuration
  local lspconfig = require('lspconfig')
  local configs = require('lspconfig.configs')

  -- Define the LuaNext language server if not already defined
  if not configs.luanext then
    configs.luanext = {
      default_config = {
        cmd = config.cmd,
        filetypes = config.filetypes,
        root_dir = config.root_dir,
        settings = config.settings,
        capabilities = config.capabilities,
        on_attach = config.on_attach,
      },
    }
  end

  -- Setup the LSP
  lspconfig.luanext.setup({
    cmd = config.cmd,
    filetypes = config.filetypes,
    root_dir = config.root_dir,
    settings = config.settings,
    capabilities = config.capabilities,
    on_attach = config.on_attach,
  })

  -- Register commands
  vim.api.nvim_create_user_command('LuaNextRestart', function()
    vim.cmd('LspRestart luanext')
  end, { desc = 'Restart LuaNext Language Server' })

  vim.api.nvim_create_user_command('LuaNextInfo', function()
    vim.cmd('LspInfo')
  end, { desc = 'Show LuaNext Language Server Information' })

  -- Set up autocommands for .luax files
  vim.api.nvim_create_autocmd({'BufRead', 'BufNewFile'}, {
    pattern = '*.luax',
    callback = function()
      vim.bo.filetype = 'luanext'
    end,
  })

  vim.notify('LuaNext plugin loaded', vim.log.levels.INFO)
end

return M
