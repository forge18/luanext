# LuaNext IDE Extensions

Official IDE extensions and plugins for LuaNext, providing syntax highlighting, LSP integration, and advanced development features across multiple editors and IDEs.

## Available Extensions

### 🎨 [Visual Studio Code](vscode/)
**Status**: ✅ Production Ready

Full-featured extension with LSP integration for VS Code.

**Features**:
- Syntax highlighting
- Auto-completion
- Type checking
- Go to definition
- Find references
- Inlay hints
- Code formatting

**Installation**: Available on [VS Code Marketplace](https://marketplace.visualstudio.com/) (search "LuaNext")

---

### ⚡ [Neovim](neovim/)
**Status**: ✅ Production Ready

Native Lua plugin with LSP support for Neovim 0.8+.

**Features**:
- Native LSP integration
- Syntax highlighting via TreeSitter-compatible syntax files
- Auto-completion via nvim-cmp
- Type hints and diagnostics
- Standard LSP keybindings

**Installation**: Compatible with lazy.nvim, packer.nvim, or manual installation.

---

### 🧠 [JetBrains IDEs](jetbrains/)
**Status**: 🚧 Beta

Plugin for IntelliJ IDEA, WebStorm, PyCharm, and other JetBrains IDEs.

**Features**:
- Syntax highlighting
- LSP integration via LSP4IJ
- Code navigation
- Refactoring support
- File templates
- Project configuration

**Installation**: From JetBrains Marketplace (coming soon) or build from source.

**Supported IDEs**:
- IntelliJ IDEA Ultimate/Community (2023.1+)
- WebStorm (2023.1+)
- PyCharm Professional/Community (2023.1+)
- CLion (2023.1+)
- Rider (2023.1+)

---

### 🔧 [ZeroBrane Studio](zerobrane-studio/)
**Status**: 🚧 Beta

Lua-based plugin for ZeroBrane Studio IDE.

**Features**:
- Syntax highlighting
- LSP integration
- Type checking
- Code compilation
- Auto-completion
- Function signatures

**Installation**: Copy to ZeroBrane Studio packages directory or configure in settings.

---

## Quick Comparison

| Feature | VS Code | Neovim | JetBrains | ZeroBrane |
|---------|---------|--------|-----------|-----------|
| Syntax Highlighting | ✅ | ✅ | ✅ | ✅ |
| LSP Integration | ✅ | ✅ | ✅ | ✅ |
| Auto-completion | ✅ | ✅ | ✅ | ✅ |
| Type Checking | ✅ | ✅ | ✅ | ✅ |
| Go to Definition | ✅ | ✅ | ✅ | ✅ |
| Find References | ✅ | ✅ | ✅ | ✅ |
| Inlay Hints | ✅ | ✅ | ✅ | ⚠️ |
| Code Formatting | ✅ | ✅ | ✅ | 🚧 |
| Refactoring | ⚠️ | ⚠️ | ✅ | ❌ |
| Debugging | 🚧 | 🚧 | 🚧 | ✅ |
| Project Templates | ❌ | ❌ | ✅ | ❌ |

**Legend**: ✅ Full Support | ⚠️ Partial Support | 🚧 In Development | ❌ Not Available

## Common Features

All extensions provide:

### Language Server Protocol (LSP)
All extensions use the same `luanext-lsp` language server, ensuring consistent behavior across editors:
- Real-time type checking
- Intelligent code completion
- Hover information
- Signature help
- Diagnostics (errors and warnings)

### Syntax Highlighting
Full syntax highlighting for LuaNext including:
- Type annotations
- Type keywords (`type`, `interface`, `namespace`, `enum`)
- Generics
- Decorators
- Import/export statements
- TypeScript-style operators

### Configuration
All extensions support configuration through:
- **Project config**: `luanext.config.yaml` in project root
- **Editor settings**: IDE-specific settings for extension behavior
- **LSP initialization**: Settings passed to language server on startup

## Prerequisites

All extensions require:

1. **LuaNext Compiler**: Install from [releases](https://github.com/yourusername/luanext/releases)
   ```bash
   # macOS/Linux
   curl -fsSL https://luanext.dev/install.sh | sh

   # Windows
   powershell -c "iwr https://luanext.dev/install.ps1 -useb | iex"
   ```

2. **Language Server**: Included with compiler, or install separately:
   ```bash
   cargo install luanext-lsp
   ```

3. **Lua Runtime** (optional, for running compiled code):
   - Lua 5.1, 5.2, 5.3, or 5.4
   - LuaJIT 2.0+

## Installation Guides

### Quick Start

Choose your editor and follow the README in the corresponding directory:

- **VS Code**: [vscode/README.md](vscode/README.md)
- **Neovim**: [neovim/README.md](neovim/README.md)
- **JetBrains**: [jetbrains/README.md](jetbrains/README.md)
- **ZeroBrane Studio**: [zerobrane-studio/README.md](zerobrane-studio/README.md)

### Building from Source

Each extension can be built from source:

```bash
# VS Code
cd editors/vscode
npm install
npm run compile
npm run package

# JetBrains
cd editors/jetbrains
./gradlew buildPlugin

# Neovim & ZeroBrane - no build required (Lua-based)
```

## Configuration Examples

### Project Configuration

Create `luanext.config.yaml` in your project root:

```yaml
compilerOptions:
  target: "auto"              # Lua version: auto, 5.1-5.4
  outDir: "./dist"            # Output directory
  strictNullChecks: true      # Enable strict null checking
  optimizationLevel: "auto"   # none, minimal, moderate, aggressive, auto
  sourceMap: true             # Generate source maps
  outputFormat: "readable"    # readable, compact, minified

include:
  - "src/**/*.luax"

exclude:
  - "**/node_modules/**"
  - "**/dist/**"
```

### Editor-Specific Settings

#### VS Code
```json
{
  "luanext.server.path": "luanext-lsp",
  "luanext.compiler.checkOnSave": true,
  "luanext.format.enable": true,
  "luanext.inlayHints.typeHints": true
}
```

#### Neovim
```lua
require('luanext').setup({
  cmd = { 'luanext-lsp' },
  settings = {
    luanext = {
      checkOnSave = true,
      strictNullChecks = true,
    },
  },
})
```

#### JetBrains
Settings → Languages & Frameworks → LuaNext

#### ZeroBrane Studio
```lua
-- cfg/user.lua
luanext = {
  lspPath = "luanext-lsp",
  enableTypeChecking = true,
  strictNullChecks = true,
}
```

## Troubleshooting

### Language Server Issues

If the language server fails to start:

1. **Verify installation**:
   ```bash
   which luanext-lsp  # Unix
   where luanext-lsp  # Windows
   ```

2. **Check version**:
   ```bash
   luanext-lsp --version
   ```

3. **Test manually**:
   ```bash
   luanext-lsp  # Should start and wait for stdin
   ```

4. **Check logs**:
   - **VS Code**: Output → LuaNext Language Server
   - **Neovim**: `:LspLog`
   - **JetBrains**: Help → Show Log
   - **ZeroBrane**: View → Output → Console

### Extension Not Loading

1. **Verify extension is installed**:
   - Check plugins/extensions list in IDE
   - Ensure extension is enabled

2. **Check file association**:
   - Verify `.luax` files are recognized
   - Manually set file type if needed

3. **Restart IDE**:
   - Some changes require restart
   - Clear caches if available

### Performance Issues

For large projects:

1. **Exclude directories**:
   ```yaml
   # luanext.config.yaml
   exclude:
     - "**/node_modules/**"
     - "**/dist/**"
     - "**/.git/**"
   ```

2. **Disable features temporarily**:
   - Turn off inlay hints
   - Disable format on save
   - Reduce real-time checking

3. **Increase IDE resources**:
   - Allocate more memory to IDE/editor
   - Close unused projects

## Development

### Contributing

We welcome contributions! See individual extension directories for development setup.

General workflow:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

### Testing

Each extension has its own testing approach:
- **VS Code**: `npm test`
- **JetBrains**: `./gradlew test`
- **Neovim**: Manual testing recommended
- **ZeroBrane**: Manual testing recommended

### Releasing

Extensions are released independently:
- **VS Code**: Published to marketplace via CI/CD
- **JetBrains**: Published to JetBrains Marketplace
- **Neovim**: Version tags in git
- **ZeroBrane**: Bundled with main releases

## Support

### Getting Help

- **Documentation**: https://luanext.dev/docs
- **Discord**: https://discord.gg/luanext
- **GitHub Issues**: https://github.com/yourusername/luanext/issues
- **Stack Overflow**: Tag `luanext`

### Reporting Bugs

When reporting bugs, include:
- Editor/IDE name and version
- Extension version
- `luanext-lsp` version
- Minimal reproduction case
- Error messages and logs

## License

All IDE extensions are licensed under MIT. See [LICENSE](../LICENSE) for details.

## Related Projects

- [LuaNext Compiler](../) - Main compiler and toolchain
- [LuaNext LSP](../crates/luanext-lsp/) - Language server
- [LuaNext Syntax](https://github.com/yourusername/luanext-syntax) - TextMate grammars
- [LuaNext Examples](https://github.com/yourusername/luanext-examples) - Example projects

## Roadmap

Upcoming features:
- [ ] Debugging support (DAP protocol)
- [ ] Advanced refactoring operations
- [ ] Code snippets and templates
- [ ] Project scaffolding tools
- [ ] Performance profiler integration
- [ ] Visual AST inspector
- [ ] Real-time collaboration support

---

**Note**: Extensions marked as Beta (🚧) are functional but may have missing features or known issues. We appreciate bug reports and contributions!
