# LuaNext VS Code Extension Testing Guide

This guide covers how to test the LuaNext VS Code extension locally before publishing.

## Prerequisites

1. VS Code installed (version 1.75.0 or higher)
2. LuaNext LSP server built:
   ```bash
   cd /path/to/luanext
   cargo build --release --package luanext-lsp
   ```
3. Extension dependencies installed:
   ```bash
   cd editors/vscode
   npm install
   npm run compile
   ```

## Testing Methods

### Method 1: Run Extension in Development Mode (Recommended)

This launches a new VS Code window with the extension loaded for testing.

1. **Open the extension in VS Code:**
   ```bash
   cd editors/vscode
   code .
   ```

2. **Start debugging:**
   - Press `F5` (or Run > Start Debugging)
   - This opens a new VS Code window titled "[Extension Development Host]"

3. **In the Extension Development Host window:**
   - Create a new file: `test.luax`
   - The extension should activate automatically
   - Check the Output panel (View > Output) for "LuaNext Language Server" logs

### Method 2: Install as VSIX Package

This tests the extension as users would install it.

1. **Package the extension:**
   ```bash
   cd editors/vscode
   npm run package
   ```
   This creates `luanext-0.1.0.vsix`

2. **Install the extension:**
   ```bash
   code --install-extension luanext-0.1.0.vsix
   ```
   Or through VS Code UI: Extensions > ... > Install from VSIX

3. **Test with a `.luax` file**

## Test Checklist

### ✅ Extension Activation

- [ ] Extension activates when opening a `.luax` file
- [ ] "LuaNext extension is now active" appears in Debug Console
- [ ] Language server starts without errors
- [ ] Output channel "LuaNext Language Server" is created

### ✅ Language Server Connection

- [ ] LSP server process starts (`luanext-lsp`)
- [ ] No connection errors in Output channel
- [ ] Server initialization completes successfully
- [ ] Server capabilities are advertised correctly

### ✅ Basic Language Features

**Syntax Highlighting:**
- [ ] Keywords highlighted (function, local, if, etc.)
- [ ] Comments highlighted (-- and --[[ ]])
- [ ] Strings highlighted
- [ ] Numbers highlighted

**Auto-Closing Pairs:**
- [ ] `{` auto-closes with `}`
- [ ] `[` auto-closes with `]`
- [ ] `(` auto-closes with `)`
- [ ] `"` auto-closes with `"`
- [ ] `'` auto-closes with `'`

**Indentation:**
- [ ] Press Enter after `function foo()` → auto-indents
- [ ] Type `end` → auto-outdents
- [ ] Press Enter after `{` → auto-indents
- [ ] Type `}` → auto-outdents

**Code Folding:**
- [ ] Function blocks can be folded
- [ ] If/then/end blocks can be folded
- [ ] Multi-line comments can be folded
- [ ] Table literals can be folded

### ✅ LSP Features

**Diagnostics:**
- [ ] Parse errors appear as red squiggles
- [ ] Type errors appear as red squiggles
- [ ] Warnings appear as yellow squiggles
- [ ] Diagnostics update on file change
- [ ] Diagnostics clear when errors fixed

**Completion:**
- [ ] Trigger completion with Ctrl+Space
- [ ] Keyword suggestions appear
- [ ] Type suggestions appear
- [ ] Completion works after `.` (member access)
- [ ] Completion works after `:` (method call)

**Hover Information:**
- [ ] Hover over keywords shows documentation
- [ ] Hover over types shows information
- [ ] Hover over identifiers shows type info
- [ ] Markdown formatting renders correctly

**Go to Definition:**
- [ ] F12 on variable jumps to declaration
- [ ] F12 on function jumps to definition
- [ ] Ctrl+Click works for navigation

**Find References:**
- [ ] Shift+F12 shows reference list
- [ ] All references highlighted in current file
- [ ] Reference count shows correctly

**Rename:**
- [ ] F2 on identifier opens rename box
- [ ] Renaming updates all occurrences
- [ ] Preview shows all changes
- [ ] Rename validates identifier names

**Formatting:**
- [ ] Shift+Alt+F formats document
- [ ] Format on save works (if enabled)
- [ ] Indentation respects settings

**Inlay Hints:**
- [ ] Type hints appear for inferred types
- [ ] Parameter hints appear in function calls
- [ ] Can be toggled in settings

### ✅ Extension Commands

**Restart Language Server:**
- [ ] Command appears in command palette (Ctrl+Shift+P)
- [ ] Command restarts server successfully
- [ ] "Restarting LuaNext Language Server..." message appears
- [ ] Extension reconnects after restart

**Show Output Channel:**
- [ ] Command appears in command palette
- [ ] Opens "LuaNext Language Server" output panel
- [ ] Shows LSP communication logs

### ✅ Extension Settings

Open Settings (Ctrl+,) and search for "luanext":

- [ ] `luanext.trace.server` setting exists
- [ ] `luanext.server.path` setting exists
- [ ] `luanext.compiler.checkOnSave` setting exists
- [ ] `luanext.compiler.strictNullChecks` setting exists
- [ ] `luanext.format.enable` setting exists
- [ ] `luanext.format.indentSize` setting exists
- [ ] `luanext.inlayHints.typeHints` setting exists
- [ ] `luanext.inlayHints.parameterHints` setting exists

**Test changing settings:**
- [ ] Change `luanext.trace.server` to "verbose" → see detailed logs
- [ ] Change `luanext.inlayHints.typeHints` to false → hints disappear
- [ ] Change `luanext.format.indentSize` to 2 → formatting uses 2 spaces

### ✅ Error Handling

**Missing LSP Server:**
- [ ] Set `luanext.server.path` to invalid path
- [ ] Reload window
- [ ] Error message appears: "Failed to start LuaNext Language Server"
- [ ] User can see error in Output channel

**Invalid .luax File:**
- [ ] Open file with syntax errors
- [ ] Red squiggles appear
- [ ] Hover shows error message
- [ ] Problems panel lists errors

## Sample Test Files

### test-basic.luax
```lua
-- Test basic LuaNext features
function greet(name: string): string
    return "Hello, " .. name
end

local message = greet("World")
print(message)
```

### test-types.luax
```lua
-- Test type system
type Point = {
    x: number,
    y: number
}

function distance(p1: Point, p2: Point): number
    local dx = p2.x - p1.x
    local dy = p2.y - p1.y
    return math.sqrt(dx * dx + dy * dy)
end

const origin: Point = { x = 0, y = 0 }
const target: Point = { x = 3, y = 4 }
print(distance(origin, target))  -- Should show 5
```

### test-errors.luax
```lua
-- Test error detection
function add(a: number, b: number): number
    return a + b
end

-- Type error: passing string to number parameter
local result = add(10, "20")  -- Should show red squiggle

-- Parse error: missing 'end'
function broken()
    local x = 1
-- Should show error about missing 'end'
```

## Troubleshooting

### Extension doesn't activate
- Check file extension is `.luax`
- Check Developer Tools (Help > Toggle Developer Tools) for JavaScript errors
- Reload window (Ctrl+Shift+P > "Developer: Reload Window")

### Language server doesn't start
- Verify `luanext-lsp` is in PATH: `which luanext-lsp`
- Check Output channel for error messages
- Try absolute path in `luanext.server.path` setting
- Check LSP server builds: `cargo build --package luanext-lsp`

### Features not working
- Check Output channel for LSP errors
- Enable verbose logging: `luanext.trace.server` = "verbose"
- Restart language server via command palette
- Check if server capabilities are advertised (in Output channel)

### Extension won't package
- Ensure all npm dependencies installed: `npm install`
- Ensure TypeScript compiles: `npm run compile`
- Check for ESLint errors: `npm run lint`
- Install vsce globally if needed: `npm install -g @vscode/vsce`

## Performance Testing

- [ ] Open large `.luax` file (>1000 lines) → should remain responsive
- [ ] Type rapidly → no noticeable lag
- [ ] Save file → diagnostics update quickly
- [ ] Memory usage remains reasonable (check Task Manager/Activity Monitor)
- [ ] CPU usage is low when idle

## Multi-file Testing

- [ ] Open workspace with multiple `.luax` files
- [ ] Go to definition across files works
- [ ] Find references across files works
- [ ] Rename across files works
- [ ] Diagnostics appear in all open files

## Regression Testing

After making changes to the extension:

1. Re-compile: `npm run compile`
2. Re-run all tests above
3. Check for new errors in Output channel
4. Verify no features broke

## Reporting Issues

When reporting bugs, include:
1. VS Code version
2. Extension version
3. LuaNext LSP server version (`luanext-lsp --version`)
4. Steps to reproduce
5. Output channel logs (with `luanext.trace.server` = "verbose")
6. Screenshots if applicable

---

**Last Updated:** 2026-01-03
