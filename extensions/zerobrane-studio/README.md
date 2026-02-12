# LuaNext Plugin for ZeroBrane Studio

Language support for LuaNext in ZeroBrane Studio IDE with LSP integration.

## Features

- Syntax highlighting for `.luax` files
- LSP integration with `luanext-lsp`
- Auto-completion and code navigation
- Type checking and diagnostics
- Code outlining and folding
- Quick documentation lookup
- Function signatures and parameter hints

## Installation

### Automatic Installation (Recommended)

1. Open ZeroBrane Studio
2. Go to `Edit` → `Preferences` → `Settings: User`
3. Add the following to install the plugin:

```lua
-- Install LuaNext plugin
path.luanext = '/path/to/luanext/editors/zerobrane-studio'
```

4. Restart ZeroBrane Studio

### Manual Installation

1. Locate your ZeroBrane Studio installation:
   - **Windows**: `C:\Program Files\ZeroBraneStudio\`
   - **macOS**: `/Applications/ZeroBraneStudio.app/Contents/ZeroBraneStudio/`
   - **Linux**: `/opt/zbstudio/` or `~/zbstudio/`

2. Copy the plugin files:
   ```bash
   # Copy to packages directory
   cp -r editors/zerobrane-studio/luanext.lua [ZBS_INSTALL]/packages/

   # Copy spec and API files
   cp editors/zerobrane-studio/spec/luanext.lua [ZBS_INSTALL]/spec/
   cp editors/zerobrane-studio/api/lua/luanext.lua [ZBS_INSTALL]/api/lua/
   ```

3. Restart ZeroBrane Studio

## Configuration

Edit your `cfg/user.lua` file (accessible via `Edit` → `Preferences` → `Settings: User`):

```lua
-- LuaNext configuration
luanext = {
  -- Path to luanext-lsp executable
  lspPath = "luanext-lsp",

  -- Enable/disable features
  enableLSP = true,
  enableTypeChecking = true,
  strictNullChecks = true,

  -- Auto-completion settings
  autoComplete = true,
  showParameterHints = true,
  showTypeHints = true,

  -- Formatting options
  formatOnSave = false,
  indentSize = 4,
  useTabs = false,

  -- Compiler options
  target = "5.4", -- Lua target version
  optimizationLevel = "auto", -- none, minimal, moderate, aggressive, auto
}
```

## Usage

### Opening LuaNext Files

Simply open any `.luax` file in ZeroBrane Studio. The plugin will automatically:
- Apply syntax highlighting
- Start the language server
- Enable type checking

### Commands

Access LuaNext commands from the `Project` menu or via keyboard shortcuts:

- **Compile File** (`Ctrl+Shift+B` / `Cmd+Shift+B`): Compile current file to Lua
- **Type Check** (`Ctrl+T` / `Cmd+T`): Run type checker on current file
- **Show Type Info** (`Ctrl+Shift+T` / `Cmd+Shift+T`): Show type at cursor
- **Go to Definition** (`F12`): Jump to symbol definition
- **Find References** (`Shift+F12`): Find all references to symbol
- **Restart LSP** (`Ctrl+Shift+L` / `Cmd+Shift+L`): Restart language server

### Code Completion

Trigger auto-completion with:
- `Ctrl+Space` / `Cmd+Space`: Manual completion
- Typing `.` or `:`: Auto-completion after member access
- Typing `(`: Parameter hints for functions

### Type Information

- **Hover**: Hover over symbols to see type information
- **Signature Help**: Parameter hints appear when typing function calls
- **Inlay Hints**: Inline type annotations (configurable)

## Features in Detail

### Syntax Highlighting

Full syntax highlighting for LuaNext including:
- Type annotations (`: string`, `: number`, etc.)
- Type keywords (`type`, `interface`, `namespace`, `enum`)
- Generics (`<T>`, `Array<string>`)
- Decorators (`@decorator`)
- Import/export statements
- TypeScript-style operators (`??`, `?.`)

### Code Analysis

Real-time diagnostics for:
- Type errors
- Syntax errors
- Undefined variables
- Unused declarations
- Null safety violations

### Code Navigation

- **Go to Definition**: Jump to where a symbol is defined
- **Find References**: Find all uses of a symbol
- **Symbol Search**: Quick search for types, functions, variables
- **Outline View**: Tree view of file structure

### Snippets

Built-in code snippets for common patterns:
- `interface` - Create interface declaration
- `type` - Create type alias
- `class` - Create class definition
- `function` - Create function with type annotations
- `import` - Import statement
- `export` - Export statement

## Building and Compiling

### Compile Current File

1. Open a `.luax` file
2. Press `Ctrl+Shift+B` / `Cmd+Shift+B`
3. Compiled `.lua` file will be generated in the same directory

### Compile Project

1. Set up a `luanext.config.yaml` in your project root
2. Go to `Project` → `Compile LuaNext Project`
3. All `.luax` files will be compiled according to config

### Watch Mode

Enable watch mode to automatically recompile on save:

```lua
-- In cfg/user.lua
luanext.watchMode = true
```

## Troubleshooting

### Language Server Not Starting

1. **Check LSP executable**:
   ```bash
   which luanext-lsp
   ```

2. **View error output**:
   - Go to `View` → `Output` → `Console`
   - Look for errors starting with `[LuaNext]`

3. **Check plugin installation**:
   - Verify files are in the correct directories
   - Check `Edit` → `Preferences` → `Settings: System` for plugin paths

### No Syntax Highlighting

1. Verify the file extension is `.luax`
2. Check that the plugin is loaded:
   - Look for "LuaNext" in the bottom-right status bar
3. Manually set the file type:
   - `Project` → `Lua Interpreter` → `LuaNext`

### Auto-Completion Not Working

1. Ensure LSP is enabled in settings
2. Check that `luanext-lsp` is in your PATH
3. Restart the language server:
   - `Project` → `Restart LuaNext LSP`

### Performance Issues

For large projects:

1. Exclude large directories from analysis:
   ```lua
   -- In luanext.config.yaml
   exclude:
     - "**/node_modules/**"
     - "**/dist/**"
     - "**/.git/**"
   ```

2. Disable real-time type checking:
   ```lua
   -- In cfg/user.lua
   luanext.enableTypeChecking = false
   ```

3. Use manual type checking instead:
   - Press `Ctrl+T` / `Cmd+T` to check on demand

## Development

### Plugin Structure

```
zerobrane-studio/
├── luanext.lua              # Main plugin file
├── spec/
│   └── luanext.lua          # Language specification
├── api/
│   └── lua/
│       └── luanext.lua      # API definitions
└── README.md                # This file
```

### Testing

To test plugin changes:

1. Edit the plugin files
2. Reload ZeroBrane Studio
3. Check `View` → `Output` → `Console` for errors

### Contributing

See the main LuaNext [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## Resources

- [ZeroBrane Studio Documentation](https://studio.zerobrane.com/documentation)
- [ZeroBrane Studio Plugin API](https://studio.zerobrane.com/doc-plugin)
- [LuaNext Documentation](https://luanext.dev/docs)

## License

MIT
