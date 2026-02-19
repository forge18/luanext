---
title: Configuration Reference
---

# Configuration Reference

Complete reference for `luanext.config.yaml` configuration file.

## Overview

The `luanext.config.yaml` file configures the LuaNext compiler for your project. Place it in your project root to customize compiler behavior, type checking, output format, and more.

## Configuration File Structure

```yaml
compilerOptions:
  # Type checking options
  strictNullChecks: true
  strictNaming: "error"
  noImplicitUnknown: false
  noExplicitUnknown: false

  # Code generation options
  target: "5.4"
  enableDecorators: true
  allowNonTypedLua: true

  # Output options
  outDir: "dist"
  outFile: null
  sourceMap: false
  noEmit: false

  # Module system
  moduleMode: "require"
  modulePaths:
    - "./?.luax"
    - "./lua_modules/?.luax"
  enforceNamespacePath: false

  # Path aliases (TypeScript-style)
  baseUrl: "."
  paths:
    "@/*": ["src/*"]
    "@components/*": ["src/components/*"]

  # Formatting
  outputFormat: "readable"
  pretty: true
  copyLuaToOutput: false

include:
  - "src/**/*.luax"
  - "lib/**/*.luax"

exclude:
  - "**/node_modules/**"
  - "**/dist/**"
  - "**/*.test.luax"
```

## Compiler Options

### `compilerOptions.strictNullChecks`

**Type:** `boolean`
**Default:** `true`

Enable strict null checking. When enabled, values cannot be `nil` unless explicitly allowed with union types.

```yaml
compilerOptions:
  strictNullChecks: true
```

**With strict null checks:**

```lua
function greet(name: string): void
    print("Hello, " .. name)
end

greet(nil)  -- ❌ Error: Cannot pass nil to string parameter
```

**Without strict null checks:**

```lua
function greet(name: string): void
    print("Hello, " .. (name or "Guest"))
end

greet(nil)  -- ✅ OK (nil treated as valid string)
```

**Recommendation:** Keep enabled for type safety. Use `string | nil` when nil is intentional.

### `compilerOptions.strictNaming`

**Type:** `"off" | "warning" | "error"`
**Default:** `"error"`

Enforce naming conventions for identifiers.

```yaml
compilerOptions:
  strictNaming: "error"  # Violations are errors
  # OR
  strictNaming: "warning"  # Violations are warnings
  # OR
  strictNaming: "off"  # No enforcement
```

**Naming conventions checked:**

- **Constants** — `UPPER_SNAKE_CASE` (`MAX_SIZE`, `DEFAULT_TIMEOUT`)
- **Variables** — `camelCase` or `snake_case` (`userName`, `user_name`)
- **Types/Interfaces** — `PascalCase` (`UserProfile`, `HttpRequest`)
- **Functions** — `camelCase` or `snake_case` (`getUserName`, `get_user_name`)

**Example:**

```lua
const MAX_SIZE: number = 100  -- ✅ OK (UPPER_SNAKE_CASE)
const maxSize: number = 100   -- ❌ Error with strictNaming: "error"

interface UserData  -- ✅ OK (PascalCase)
interface user_data  -- ❌ Error with strictNaming: "error"
```

### `compilerOptions.noImplicitUnknown`

**Type:** `boolean`
**Default:** `false`

Disallow implicit `unknown` types. Requires explicit type annotations.

```yaml
compilerOptions:
  noImplicitUnknown: true
```

**With `noImplicitUnknown: true`:**

```lua
function process(data)  -- ❌ Error: Parameter 'data' has implicit unknown type
    print(data)
end

-- Must annotate explicitly
function process(data: unknown): void  -- ✅ OK
    print(data)
end
```

**Recommendation:** Enable for libraries and shared code. Disable for rapid prototyping.

### `compilerOptions.noExplicitUnknown`

**Type:** `boolean`
**Default:** `false`

Disallow explicit `unknown` types. Forces all types to be known.

```yaml
compilerOptions:
  noExplicitUnknown: true
```

**With `noExplicitUnknown: true`:**

```lua
const data: unknown = getData()  -- ❌ Error: unknown type not allowed

-- Must use specific type
const data: string | number | table = getData()  -- ✅ OK
```

**Recommendation:** Rarely needed. Use only in highly type-strict codebases.

### `compilerOptions.target`

**Type:** `"5.1" | "5.2" | "5.3" | "5.4"`
**Default:** `"5.4"`

Target Lua version for code generation.

```yaml
compilerOptions:
  target: "5.4"  # Latest features
  # OR
  target: "5.1"  # LuaJIT compatible
```

**Affects:**

- **Integers** — Native in 5.3+, type-checked only in 5.1/5.2
- **Bitwise operators** — Native in 5.3+, library calls in 5.1/5.2
- **Integer division (`//`)** — Native in 5.3+, `math.floor(a/b)` in 5.1/5.2
- **Standard library** — Different functions available in different versions

See [Lua Targets Guide](../guides/lua-targets.md) for complete compatibility matrix.

### `compilerOptions.enableDecorators`

**Type:** `boolean`
**Default:** `true`

Enable decorator syntax (`@decorator`).

```yaml
compilerOptions:
  enableDecorators: true
```

**With decorators enabled:**

```lua
@readonly
class Config
    version: string
end
```

**Without decorators:**

```lua
-- Decorator syntax not available
class Config
    readonly version: string  -- Use property modifier instead
end
```

### `compilerOptions.allowNonTypedLua`

**Type:** `boolean`
**Default:** `true`

Allow importing plain `.lua` files without type information.

```yaml
compilerOptions:
  allowNonTypedLua: true
```

**With `allowNonTypedLua: true`:**

```lua
import * as legacy from "./legacy-code.lua"  -- ✅ OK (types treated as unknown)
```

**With `allowNonTypedLua: false`:**

```lua
import * as legacy from "./legacy-code.lua"  -- ❌ Error: Cannot import untyped Lua
```

**Recommendation:** Keep enabled when gradually migrating from Lua. Disable for fully-typed codebases.

### `compilerOptions.outDir`

**Type:** `string | null`
**Default:** `null` (output to same directory as input)

Output directory for compiled Lua files.

```yaml
compilerOptions:
  outDir: "dist"
```

**Directory structure:**

```
project/
├── src/
│   ├── main.luax
│   └── utils/
│       └── helpers.luax
└── dist/                    # Output directory
    ├── main.lua
    └── utils/
        └── helpers.lua
```

**Preserves source structure:** Directory hierarchy from source is maintained in output.

### `compilerOptions.outFile`

**Type:** `string | null`
**Default:** `null` (separate files)

Bundle all output into a single file.

```yaml
compilerOptions:
  outFile: "bundle.lua"
```

**Single file output:**

```
project/
├── src/
│   ├── main.luax
│   └── utils.luax
└── bundle.lua  # All modules in one file
```

**Note:** Uses bundling strategy to combine all modules. Mutually exclusive with separate file output.

### `compilerOptions.sourceMap`

**Type:** `boolean`
**Default:** `false`

Generate source maps (`.lua.map` files).

```yaml
compilerOptions:
  sourceMap: true
```

**Output:**

```
dist/
├── main.lua
└── main.lua.map  # Source map for debugging
```

**Source maps enable:**

- Debugging LuaNext code from generated Lua
- Accurate error locations in stack traces
- IDE integration for breakpoints

### `compilerOptions.noEmit`

**Type:** `boolean`
**Default:** `false`

Type-check only, do not generate output files.

```yaml
compilerOptions:
  noEmit: true
```

**Use cases:**

- CI/CD type checking
- Editor integration (LSP)
- Pre-commit hooks

### `compilerOptions.pretty`

**Type:** `boolean`
**Default:** `true`

Pretty-print diagnostic messages with colors and context.

```yaml
compilerOptions:
  pretty: true
```

**With pretty output:**

```
error[E0001]: Type mismatch
  ┌─ main.luax:5:10
  │
5 │     const x: number = "hello"
  │              ^^^^^^   ^^^^^^^ expected number, found string
  │              │
  │              type annotation
```

**Without pretty output:**

```
main.luax:5:10: error[E0001]: Type mismatch: expected number, found string
```

### `compilerOptions.moduleMode`

**Type:** `"require" | "bundle"`
**Default:** `"require"`

Module code generation mode.

```yaml
compilerOptions:
  moduleMode: "require"  # Separate files with require()
  # OR
  moduleMode: "bundle"   # Inline all modules
```

**`require` mode:**

```lua
-- Generates separate files
local utils = require("utils")
local helper = utils.helper
```

**`bundle` mode:**

```lua
-- Inlines modules
local utils = (function()
    -- utils module code
end)()
local helper = utils.helper
```

### `compilerOptions.modulePaths`

**Type:** `string[]`
**Default:**

```yaml
- "./?.luax"
- "./lua_modules/?.luax"
- "./lua_modules/?/init.luax"
```

Module search paths for resolving imports.

```yaml
compilerOptions:
  modulePaths:
    - "./?.luax"
    - "./lib/?.luax"
    - "./vendor/?/init.luax"
```

**Path patterns:**

- `?` — Replaced with module name
- Example: Importing `"utils"` checks `./utils.luax`, `./lib/utils.luax`, `./vendor/utils/init.luax`

### `compilerOptions.enforceNamespacePath`

**Type:** `boolean`
**Default:** `false`

Enforce that namespace declarations match file paths.

```yaml
compilerOptions:
  enforceNamespacePath: true
```

**With enforcement:**

```lua
-- File: src/MyApp/Utils.luax
namespace MyApp.Utils  -- ✅ OK (matches path)

-- File: src/MyApp/Utils.luax
namespace Other.Name  -- ❌ Error: Namespace doesn't match file path
```

### `compilerOptions.baseUrl`

**Type:** `string | null`
**Default:** `null`

Base directory for resolving path alias replacements. When set, the replacement paths in `paths` are resolved relative to this directory. When `null`, they resolve relative to the project root (where `luanext.config.yaml` is located).

```yaml
compilerOptions:
  baseUrl: "."        # Project root (explicit)
  # OR
  baseUrl: "./src"    # Resolve aliases relative to src/
  # OR
  baseUrl: null       # Default: project root
```

**Example:** With `baseUrl: "./src"` and `paths: { "@/*": ["*"] }`, the import `@/utils` resolves to `src/utils.luax`.

**Note:** `baseUrl` only affects how path alias replacement values are resolved. It does not change how relative imports (`./`, `../`) or package imports work.

### `compilerOptions.paths`

**Type:** `Record<string, string[]>`
**Default:** `{}` (no aliases)

Path alias mappings following TypeScript's `paths` semantics. Maps import patterns to filesystem paths, enabling cleaner imports in large projects.

```yaml
compilerOptions:
  baseUrl: "."
  paths:
    "@/*": ["src/*"]
    "@components/*": ["src/components/*"]
    "@utils": ["src/shared/utils"]
```

**Pattern syntax:**

- Each pattern may contain at most one `*` wildcard
- `*` matches any string and is substituted into the replacement values
- Patterns without `*` are exact matches
- Multiple replacement paths are tried in order until one resolves a file
- A bare `*` catch-all pattern matches any import

**Specificity:** When multiple patterns match an import, the pattern with the longest prefix before `*` wins. For example, `@components/*` wins over `@/*` for the import `@components/Button`.

**Examples:**

```yaml
# Wildcard alias
paths:
  "@/*": ["src/*"]
# import { add } from "@/utils"  →  resolves to src/utils.luax

# Exact alias (no wildcard)
paths:
  "@config": ["src/config/settings"]
# import Config from "@config"  →  resolves to src/config/settings.luax

# Multiple fallback paths
paths:
  "@lib/*": ["src/lib/*", "vendor/lib/*"]
# Tries src/lib/foo.luax first, then vendor/lib/foo.luax

# Multiple alias prefixes
paths:
  "@/*": ["src/*"]
  "@components/*": ["src/ui/components/*"]
  "@utils/*": ["src/shared/utils/*"]
```

**Generated Lua:** In require mode, alias imports are rewritten to relative `require()` paths:

```lua
-- Source: import { Button } from "@components/Button"
-- Generated: local _mod = require("./src/ui/components/Button")
```

**Note:** The `@std/*` prefix is reserved for LuaNext standard library imports and is not affected by path alias configuration.

### `compilerOptions.outputFormat`

**Type:** `"readable" | "compact" | "minified"`
**Default:** `"readable"`

Output format for generated Lua code.

```yaml
compilerOptions:
  outputFormat: "readable"  # Pretty formatting (default)
  # OR
  outputFormat: "compact"   # Minimal whitespace
  # OR
  outputFormat: "minified"  # No unnecessary whitespace
```

**Readable:**

```lua
local function add(a, b)
    return a + b
end
```

**Compact:**

```lua
local function add(a, b)
  return a + b
end
```

**Minified:**

```lua
local function add(a,b)return a+b end
```

### `compilerOptions.copyLuaToOutput`

**Type:** `boolean`
**Default:** `false`

Copy plain `.lua` files to output directory during compilation.

```yaml
compilerOptions:
  copyLuaToOutput: true
  outDir: "dist"
```

**Example:**

```
src/
├── main.luax       # Compiled to dist/main.lua
└── legacy.lua      # Copied to dist/legacy.lua (if copyLuaToOutput: true)
```

**Use case:** Include plain Lua files in output directory without modification.

## Include/Exclude Patterns

### `include`

**Type:** `string[]`
**Default:** `["**/*.luax"]`

Files to include in compilation (glob patterns).

```yaml
include:
  - "src/**/*.luax"
  - "lib/**/*.luax"
  - "tests/**/*.luax"
```

**Glob patterns:**

- `*` — Match any characters except `/`
- `**` — Match any characters including `/` (recursive)
- `?` — Match single character
- `{a,b}` — Match either `a` or `b`

### `exclude`

**Type:** `string[]`
**Default:**

```yaml
- "**/node_modules/**"
- "**/dist/**"
```

Files to exclude from compilation (glob patterns).

```yaml
exclude:
  - "**/node_modules/**"
  - "**/dist/**"
  - "**/*.test.luax"
  - "**/examples/**"
```

**Exclude takes precedence:** Files matching `exclude` are omitted even if they match `include`.

## Configuration Examples

### Development Configuration

Fast compilation, detailed errors:

```yaml
compilerOptions:
  target: "5.4"
  strictNullChecks: true
  outDir: "build"
  sourceMap: true
  pretty: true
  outputFormat: "readable"
  allowNonTypedLua: true

include:
  - "src/**/*.luax"

exclude:
  - "**/node_modules/**"
  - "**/build/**"
```

### Production Configuration

Optimized output, strict checking:

```yaml
compilerOptions:
  target: "5.3"
  strictNullChecks: true
  strictNaming: "error"
  noImplicitUnknown: true
  outFile: "app.lua"
  sourceMap: false
  outputFormat: "minified"
  allowNonTypedLua: false

include:
  - "src/**/*.luax"

exclude:
  - "**/node_modules/**"
  - "**/tests/**"
  - "**/*.test.luax"
```

### Library Configuration

Type-safe, portable:

```yaml
compilerOptions:
  target: "5.1"  # LuaJIT compatible
  strictNullChecks: true
  strictNaming: "error"
  noImplicitUnknown: true
  outDir: "dist"
  sourceMap: true
  outputFormat: "readable"
  enableDecorators: false  # Avoid advanced features

include:
  - "src/**/*.luax"

exclude:
  - "**/node_modules/**"
  - "**/examples/**"
  - "**/tests/**"
```

### Gradual Migration Configuration

Permissive for Lua → LuaNext migration:

```yaml
compilerOptions:
  target: "5.1"
  strictNullChecks: false  # Allow nil everywhere
  strictNaming: "off"      # No naming enforcement
  noImplicitUnknown: false
  outDir: "build"
  pretty: true
  allowNonTypedLua: true   # Allow importing .lua files
  copyLuaToOutput: true    # Copy non-typed files

include:
  - "src/**/*.luax"
  - "src/**/*.lua"  # Include legacy Lua

exclude:
  - "**/node_modules/**"
```

### Path Aliases Configuration

Clean imports for large projects:

```yaml
compilerOptions:
  target: "5.4"
  strictNullChecks: true
  outDir: "dist"
  baseUrl: "."
  paths:
    "@/*": ["src/*"]
    "@components/*": ["src/ui/components/*"]
    "@utils/*": ["src/shared/utils/*"]
    "@config": ["src/config/settings"]

include:
  - "src/**/*.luax"

exclude:
  - "**/node_modules/**"
  - "**/dist/**"
```

**Usage:**

```lua
-- Instead of fragile relative paths:
import { Button } from "../../../ui/components/Button"
import { formatDate } from "../../shared/utils/date"

-- Use clean aliases:
import { Button } from "@components/Button"
import { formatDate } from "@utils/date"
import Config from "@config"
```

## CLI Overrides

Command-line options override configuration file values:

```bash
# Config has target: "5.4", CLI overrides to 5.1
luanext --target 5.1 main.luax

# Config has outDir: "build", CLI overrides to "dist"
luanext --out-dir dist main.luax

# Config has sourceMap: false, CLI enables it
luanext --source-map main.luax
```

**Precedence:** CLI > Config File > Defaults

## Configuration Loading

### Automatic Discovery

LuaNext searches for `luanext.config.yaml` in:

1. Current directory
2. Parent directories (up to project root)
3. Stops at first match

```bash
# Automatically finds ./luanext.config.yaml
luanext src/main.luax
```

### Explicit Path

Specify config file path with `--project`:

```bash
luanext --project ./configs/prod.yaml src/main.luax
```

### No Configuration

Without a config file, all defaults are used:

```bash
# Uses all default options
luanext main.luax
```

## Initializing Configuration

### Create Default Config

```bash
luanext --init
```

Creates `luanext.config.yaml` with default settings:

```yaml
compilerOptions:
  strictNullChecks: true
  strictNaming: error
  noImplicitUnknown: false
  noExplicitUnknown: false
  target: '5.4'
  enableDecorators: true
  allowNonTypedLua: true
  copyLuaToOutput: false
  sourceMap: false
  noEmit: false
  pretty: true
  moduleMode: require
  modulePaths:
  - ./?.luax
  - ./lua_modules/?.luax
  - ./lua_modules/?/init.luax
  enforceNamespacePath: false
  outputFormat: readable
include:
- '**/*.luax'
exclude:
- '**/node_modules/**'
- '**/dist/**'
```

## Best Practices

### Use Version Control

Commit `luanext.config.yaml` to version control:

```bash
git add luanext.config.yaml
git commit -m "Add LuaNext configuration"
```

### Separate Dev/Prod Configs

Use different configs for environments:

```
project/
├── luanext.config.yaml       # Development (default)
├── luanext.prod.yaml         # Production
└── luanext.test.yaml         # Testing
```

```bash
# Development
luanext

# Production
luanext --project luanext.prod.yaml

# Testing
luanext --project luanext.test.yaml
```

### Document Custom Settings

Add comments to explain non-standard options:

```yaml
compilerOptions:
  # Using 5.1 for LuaJIT compatibility
  target: "5.1"

  # Disabled during migration from Lua
  strictNullChecks: false

  # Required by third-party library
  allowNonTypedLua: true
```

### Validate Configuration

Test config changes with `--no-emit`:

```bash
# Check configuration without generating files
luanext --no-emit src/**/*.luax
```

## Troubleshooting

### "Configuration file not found"

**Problem:** `luanext.config.yaml` not found in current or parent directories.

**Solution:**

```bash
# Create default config
luanext --init

# Or specify path explicitly
luanext --project path/to/config.yaml
```

### "Invalid configuration"

**Problem:** YAML syntax error or invalid option value.

**Solution:**

```bash
# Check YAML syntax
yamllint luanext.config.yaml

# Validate against schema
luanext --project luanext.config.yaml --no-emit
```

### Options Not Taking Effect

**Problem:** CLI overrides or incorrect option names.

**Solution:**

- Check CLI flags (they override config)
- Verify option names use `camelCase` (e.g., `strictNullChecks` not `strict_null_checks`)
- Check for typos in YAML keys

## See Also

- [CLI Reference](cli.md) — Command-line options
- [Migrating from Lua](../guides/migrating-from-lua.md) — Migration strategies
- [Lua Targets](../guides/lua-targets.md) — Target version details
