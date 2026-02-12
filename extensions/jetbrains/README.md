# LuaNext JetBrains Plugin

Language support for LuaNext in JetBrains IDEs (IntelliJ IDEA, WebStorm, PyCharm, etc.) with LSP integration.

## Features

- Syntax highlighting for `.luax` files
- LSP integration with `luanext-lsp`
- Auto-completion and code navigation
- Real-time type checking and diagnostics
- Code formatting
- Quick fixes and refactoring support
- File templates for new LuaNext files

## Supported IDEs

- IntelliJ IDEA Ultimate/Community (2023.1+)
- WebStorm (2023.1+)
- PyCharm Professional/Community (2023.1+)
- CLion (2023.1+)
- Rider (2023.1+)
- Any JetBrains IDE with LSP support

## Installation

### From JetBrains Marketplace (Coming Soon)

1. Open your JetBrains IDE
2. Go to `Settings/Preferences` → `Plugins`
3. Search for "LuaNext"
4. Click `Install`
5. Restart the IDE

### From Disk

1. Build the plugin (see Building section)
2. Go to `Settings/Preferences` → `Plugins`
3. Click the gear icon ⚙️ → `Install Plugin from Disk...`
4. Select the generated `.zip` file from `build/distributions/`
5. Restart the IDE

## Building from Source

### Prerequisites

- JDK 17 or later
- Gradle 8.0 or later

### Build Steps

```bash
cd editors/jetbrains
./gradlew buildPlugin
```

The plugin will be built to `build/distributions/luanext-<version>.zip`

### Development

To run the plugin in a sandboxed IDE instance:

```bash
./gradlew runIde
```

## Configuration

### Language Server Path

Go to `Settings/Preferences` → `Languages & Frameworks` → `LuaNext`:

- **Language Server Executable**: Path to `luanext-lsp` (default: searches in PATH)
- **Check on Save**: Enable type checking when saving files
- **Strict Null Checks**: Enable strict null checking
- **Format on Save**: Automatically format files on save
- **Inlay Hints**: Show type and parameter hints inline

### File Associations

The plugin automatically associates `.luax` files with LuaNext. To change this:

1. Go to `Settings/Preferences` → `Editor` → `File Types`
2. Find "LuaNext" in the list
3. Add or remove file patterns

## Features in Detail

### Syntax Highlighting

Full syntax highlighting for:
- Type annotations (`: string`, `: number`, etc.)
- TypeScript-style keywords (`interface`, `type`, `namespace`, `enum`)
- Generics (`<T>`, `Array<string>`)
- Decorators (`@decorator`)
- Import/export statements

### Code Completion

- Context-aware completion for variables, functions, and types
- Auto-import suggestions
- Snippet completion for common patterns
- Parameter hints

### Code Navigation

- Go to Definition (Ctrl+B / Cmd+B)
- Find Usages (Alt+F7 / Opt+F7)
- Go to Type Definition
- View Type Hierarchy
- Navigate to related files

### Refactoring

- Rename symbols (Shift+F6)
- Extract variable/function
- Inline variable
- Move declaration

### Code Analysis

- Real-time error highlighting
- Type mismatch warnings
- Unused variable detection
- Quick fixes and intentions

### File Templates

Create new LuaNext files with pre-filled templates:

1. Right-click in Project view
2. `New` → `LuaNext File`
3. Choose template: Module, Interface, Enum, etc.

## Troubleshooting

### Language Server Not Starting

1. **Check LSP executable**:
   ```bash
   which luanext-lsp
   ```

2. **View IDE logs**:
   - Go to `Help` → `Show Log in Finder/Explorer`
   - Look for errors related to "luanext" or "LSP"

3. **Check plugin installation**:
   - `Settings/Preferences` → `Plugins`
   - Verify "LuaNext" is installed and enabled

### No Syntax Highlighting

1. Verify the file is recognized as LuaNext:
   - Look at the file icon in the project tree
   - Check the language indicator in the bottom-right of the editor

2. Reassociate the file type:
   - Right-click the file → `Associate with File Type...` → `LuaNext`

### Code Completion Not Working

1. Ensure the language server is running:
   - Check `Tools` → `LSP Consoles` → `luanext`

2. Invalidate caches and restart:
   - `File` → `Invalidate Caches...` → Select all → `Invalidate and Restart`

### Performance Issues

1. Increase IDE memory:
   - `Help` → `Edit Custom VM Options`
   - Increase `-Xmx` value (e.g., `-Xmx4096m` for 4GB)

2. Exclude large directories:
   - Right-click directory in Project view → `Mark Directory as` → `Excluded`

## Development

### Project Structure

```
editors/jetbrains/
├── src/main/
│   ├── java/com/luanext/plugin/
│   │   ├── LuaNextLanguage.java         # Language definition
│   │   ├── LuaNextFileType.java         # File type definition
│   │   ├── LuaNextSyntaxHighlighter.java # Syntax highlighter
│   │   ├── LuaNextLspServerDescriptor.java # LSP configuration
│   │   └── ...
│   └── resources/
│       ├── META-INF/plugin.xml          # Plugin descriptor
│       ├── fileTypes/luanext.svg        # File icon
│       └── icons/                       # UI icons
├── build.gradle.kts                     # Build configuration
└── README.md                            # This file
```

### Testing

Run tests:
```bash
./gradlew test
```

### Debugging

1. Start the IDE with the debugger:
   ```bash
   ./gradlew runIde --debug-jvm
   ```

2. Attach your debugger to port 5005

## Contributing

See the main LuaNext [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

MIT
