# Quick Start: Testing LuaNext Extension

Get the LuaNext VS Code extension running in under 5 minutes.

## Prerequisites

- VS Code 1.75.0 or higher
- Node.js and npm installed
- Rust toolchain (for building LSP server)

## Step 1: Build the LSP Server

```bash
cd /path/to/luanext
cargo build --release --package luanext-lsp
```

The binary will be at: `target/release/luanext-lsp`

## Step 2: Set up the Extension

```bash
cd editors/vscode
npm install
npm run compile
```

## Step 3: Run the Extension

### Option A: Debug in VS Code (Recommended)

1. Open the extension folder in VS Code:
   ```bash
   code editors/vscode
   ```

2. Press `F5` to start debugging

3. A new "[Extension Development Host]" window opens

4. In the new window, open the test files:
   ```
   File > Open Folder > editors/vscode/test-files
   ```

5. Open `test-basic.luax` - the extension should activate!

### Option B: Install as VSIX

1. Package the extension:
   ```bash
   cd editors/vscode
   npm run package
   ```

2. Install it:
   ```bash
   code --install-extension luanext-0.1.0.vsix
   ```

3. Open any `.luax` file to activate the extension

## Step 4: Verify It's Working

### Check Extension Activation

1. Open `test-basic.luax`
2. Open Output panel: View > Output
3. Select "LuaNext Language Server" from dropdown
4. You should see initialization messages

### Test Basic Features

**Syntax Highlighting:**
- Keywords like `function`, `local`, `const` should be colored
- Strings and comments should be colored

**Auto-Closing:**
- Type `{` → should auto-close with `}`
- Type `"` → should auto-close with `"`

**Indentation:**
- Press Enter after `function foo()` → should auto-indent
- Type `end` → should auto-outdent

**Commands:**
- Press `Ctrl+Shift+P` (Cmd+Shift+P on Mac)
- Type "LuaNext" → should see commands:
  - "LuaNext: Restart Language Server"
  - "LuaNext: Show Output Channel"

## Troubleshooting

### "Failed to start LuaNext Language Server"

The extension can't find the LSP server binary.

**Fix:**
1. Make sure you built it: `cargo build --release --package luanext-lsp`
2. Add to PATH or set absolute path in settings:
   ```json
   {
     "luanext.server.path": "/absolute/path/to/target/release/luanext-lsp"
   }
   ```

### Extension doesn't activate

**Check:**
- File extension is `.luax`
- Open Developer Tools: Help > Toggle Developer Tools
- Look for JavaScript errors in Console tab
- Try reloading: Ctrl+Shift+P > "Developer: Reload Window"

### No syntax highlighting

**Check:**
- File is recognized as LuaNext (bottom-right of VS Code should show "LuaNext")
- If it says "Plain Text", click it and select "LuaNext"
- Reopen the file

### Features not working (completion, hover, etc.)

**Check Output channel:**
1. View > Output
2. Select "LuaNext Language Server"
3. Look for errors

**Enable verbose logging:**
1. Open Settings (Ctrl+,)
2. Search for "luanext trace"
3. Set to "verbose"
4. Restart language server
5. Check Output channel again

## Next Steps

- Read [TESTING.md](./TESTING.md) for comprehensive test checklist
- Try the sample files in `test-files/`
- Report issues at https://github.com/yourusername/luanext/issues

## Common Test Scenarios

### Test Completion

1. Open `test-basic.luax`
2. Type `function` and press Space
3. Type `my` then Ctrl+Space
4. Should see keyword/identifier suggestions

### Test Hover

1. Open `test-basic.luax`
2. Hover over the `function` keyword
3. Should see documentation popup

### Test Go to Definition

1. Open `test-basic.luax`
2. Find the line: `local message = greet("World")`
3. Ctrl+Click on `greet` (or press F12)
4. Should jump to the function definition

### Test Diagnostics

1. Open `test-errors.luax`
2. Should see red squiggles on type errors
3. Hover over them to see error messages
4. Check Problems panel (View > Problems)

## VS Code Keyboard Shortcuts

- `F5` - Start debugging extension
- `Ctrl+Shift+P` - Command palette
- `Ctrl+Space` - Trigger completion
- `F12` - Go to definition
- `Shift+F12` - Find references
- `F2` - Rename symbol
- `Shift+Alt+F` - Format document
- `Ctrl+,` - Open settings

---

**Happy testing!**
