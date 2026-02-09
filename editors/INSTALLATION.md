# IDE Extensions Installation Guide

Quick installation guide for all LuaNext IDE extensions.

## Prerequisites

Before installing any extension, ensure you have:

1. **LuaNext Compiler** installed:
   ```bash
   # Check installation
   luanext --version
   ```

2. **Language Server** available in PATH:
   ```bash
   # Check installation
   which luanext-lsp  # Unix/macOS
   where luanext-lsp  # Windows
   ```

If not installed, see the [main README](../README.md) for installation instructions.

---

## Visual Studio Code

### Method 1: From Marketplace (Recommended)

1. Open VS Code
2. Press `Ctrl+Shift+X` (Windows/Linux) or `Cmd+Shift+X` (macOS)
3. Search for "LuaNext"
4. Click **Install**

### Method 2: From VSIX File

1. Download the `.vsix` file from [releases](https://github.com/yourusername/luanext/releases)
2. Open VS Code
3. Press `Ctrl+Shift+P` / `Cmd+Shift+P` → "Extensions: Install from VSIX..."
4. Select the downloaded `.vsix` file

### Method 3: Development Installation

```bash
cd editors/vscode
npm install
npm run compile
code --install-extension .
```

### Verification

1. Open a `.luax` file
2. Check status bar for "LuaNext" indicator
3. Verify syntax highlighting is active

---

## Neovim

### Method 1: Using lazy.nvim (Recommended)

Add to your Neovim configuration:

```lua
-- ~/.config/nvim/lua/plugins/luanext.lua
return {
  dir = "/path/to/luanext/editors/neovim",
  ft = "luanext",
  dependencies = {
    "neovim/nvim-lspconfig",
  },
  config = function()
    require("luanext").setup()
  end,
}
```

### Method 2: Using packer.nvim

```lua
-- ~/.config/nvim/lua/plugins.lua
use {
  '/path/to/luanext/editors/neovim',
  requires = { 'neovim/nvim-lspconfig' },
  config = function()
    require('luanext').setup()
  end
}
```

### Method 3: Manual Installation

```bash
# Copy to Neovim runtime directory
mkdir -p ~/.local/share/nvim/site/pack/luanext/start/
cp -r editors/neovim ~/.local/share/nvim/site/pack/luanext/start/luanext
```

Then add to `init.lua`:
```lua
require('luanext').setup()
```

### Verification

1. Restart Neovim
2. Open a `.luax` file: `nvim test.luax`
3. Check filetype: `:set filetype?` (should show `luanext`)
4. Verify LSP is attached: `:LspInfo`

---

## JetBrains IDEs

### Method 1: From Marketplace (Coming Soon)

1. Open your JetBrains IDE
2. Go to **Settings/Preferences** → **Plugins**
3. Search for "LuaNext"
4. Click **Install**
5. Restart IDE

### Method 2: Install from Disk

1. **Build the plugin**:
   ```bash
   cd editors/jetbrains
   ./gradlew buildPlugin
   ```

   The plugin will be in `build/distributions/luanext-0.1.0.zip`

2. **Install in IDE**:
   - Go to **Settings/Preferences** → **Plugins**
   - Click gear icon ⚙️ → **Install Plugin from Disk...**
   - Select the `.zip` file
   - Restart IDE

### Verification

1. Open a `.luax` file
2. Verify syntax highlighting
3. Check **Tools** → **LSP Consoles** → should see "luanext"
4. Go to **Settings** → **Languages & Frameworks** → should see "LuaNext"

---

## ZeroBrane Studio

### Method 1: Package Directory

1. **Copy plugin files**:
   ```bash
   # Find ZeroBrane installation
   # Windows: C:\Program Files\ZeroBraneStudio\
   # macOS: /Applications/ZeroBraneStudio.app/Contents/ZeroBraneStudio/
   # Linux: /opt/zbstudio/ or ~/zbstudio/

   # Copy main plugin
   cp editors/zerobrane-studio/luanext.lua [ZBS_DIR]/packages/

   # Copy spec file
   cp editors/zerobrane-studio/spec/luanext.lua [ZBS_DIR]/spec/

   # Copy API file
   cp editors/zerobrane-studio/api/lua/luanext.lua [ZBS_DIR]/api/lua/
   ```

2. **Restart ZeroBrane Studio**

### Method 2: User Configuration

1. Open ZeroBrane Studio
2. Go to **Edit** → **Preferences** → **Settings: User**
3. Add:
   ```lua
   path.luanext = '/path/to/luanext/editors/zerobrane-studio'
   ```
4. Restart ZeroBrane Studio

### Verification

1. Open a `.luax` file
2. Check bottom-right status bar for "LuaNext" indicator
3. Verify **Project** menu has LuaNext options
4. Check **View** → **Output** → **Console** for "[LuaNext] Plugin loaded"

---

## Post-Installation Configuration

### Configure Language Server Path

If `luanext-lsp` is not in your PATH, configure the path in your editor:

#### VS Code
```json
{
  "luanext.server.path": "/absolute/path/to/luanext-lsp"
}
```

#### Neovim
```lua
require('luanext').setup({
  cmd = { '/absolute/path/to/luanext-lsp' }
})
```

#### JetBrains
Settings → Languages & Frameworks → LuaNext → Language Server Executable

#### ZeroBrane Studio
```lua
-- cfg/user.lua
luanext = {
  lspPath = '/absolute/path/to/luanext-lsp'
}
```

### Create Project Configuration

Create `luanext.config.yaml` in your project root:

```yaml
compilerOptions:
  target: "auto"
  outDir: "./dist"
  strictNullChecks: true
  optimizationLevel: "auto"

include:
  - "src/**/*.luax"

exclude:
  - "**/node_modules/**"
  - "**/dist/**"
```

---

## Troubleshooting

### Extension Not Loading

**VS Code**:
```bash
# Check extension status
code --list-extensions | grep luanext

# View extension logs
# Command Palette → "Developer: Show Logs" → Extension Host
```

**Neovim**:
```vim
" Check if plugin is loaded
:scriptnames | grep luanext

" View LSP logs
:LspLog
```

**JetBrains**:
- **File** → **Invalidate Caches...**
- Check **Help** → **Show Log in Finder/Explorer**

**ZeroBrane Studio**:
- Check **View** → **Output** → **Console**
- Verify files are in correct directories

### Language Server Not Starting

1. **Verify executable**:
   ```bash
   luanext-lsp --version
   ```

2. **Test manually**:
   ```bash
   luanext-lsp
   # Should start and wait for JSON-RPC input
   # Press Ctrl+C to exit
   ```

3. **Check permissions**:
   ```bash
   ls -la $(which luanext-lsp)
   # Should have execute permissions
   ```

4. **Add to PATH** (if needed):
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export PATH="/path/to/luanext/bin:$PATH"
   ```

### No Syntax Highlighting

1. **Verify file extension**: File must end with `.luax`

2. **Check file type**:
   - **VS Code**: Look at bottom-right corner
   - **Neovim**: `:set filetype?`
   - **JetBrains**: Right-click → Associate with File Type
   - **ZeroBrane**: Check status bar

3. **Reload/restart**: Some editors need restart after installation

### Still Having Issues?

1. Check the detailed README for your editor:
   - [VS Code README](vscode/README.md)
   - [Neovim README](neovim/README.md)
   - [JetBrains README](jetbrains/README.md)
   - [ZeroBrane Studio README](zerobrane-studio/README.md)

2. Search [existing issues](https://github.com/yourusername/luanext/issues)

3. Ask on [Discord](https://discord.gg/luanext)

4. Create a new issue with:
   - Editor name and version
   - Extension version
   - `luanext-lsp` version
   - Error messages and logs

---

## Next Steps

After installation:

1. **Read the documentation**: https://luanext.dev/docs
2. **Try the examples**: https://github.com/yourusername/luanext-examples
3. **Join the community**: https://discord.gg/luanext
4. **Configure your workflow**: See editor-specific READMEs for advanced configuration

Happy coding! 🎉
