# Project Setup

For multi-file projects, create a `luanext.config.yaml` configuration file to specify compiler options, input files, and output settings.

## Initialize a New Project

Create a new project with default configuration:

```bash
luanext --init
```

This generates `luanext.config.yaml` in the current directory:

```yaml
compilerOptions:
  strictNullChecks: true
  strictNaming: error
  noImplicitUnknown: false
  noExplicitUnknown: false
  target: "5.4"
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
  - "**/*.luax"

exclude:
  - "**/node_modules/**"
  - "**/dist/**"
```

## Configuration Options

### Compiler Options

#### Type Checking

**`strictNullChecks`** (boolean, default: `true`)

Enable strict null checking. Prevents using nullable values without checking for `nil` first.

```yaml
compilerOptions:
  strictNullChecks: true
```

**`strictNaming`** (string, default: `"error"`)

Enforce naming conventions for variables, functions, types, etc. Options: `"off"`, `"warning"`, `"error"`.

```yaml
compilerOptions:
  strictNaming: error  # or "warning" or "off"
```

**`noImplicitUnknown`** (boolean, default: `false`)

Disallow implicit `unknown` types. Forces explicit type annotations when type cannot be inferred.

```yaml
compilerOptions:
  noImplicitUnknown: true
```

**`noExplicitUnknown`** (boolean, default: `false`)

Disallow explicit use of `unknown` type. Requires more specific types.

```yaml
compilerOptions:
  noExplicitUnknown: true
```

#### Code Generation

**`target`** (string, default: `"5.4"`)

Target Lua version. Options: `"5.1"`, `"5.2"`, `"5.3"`, `"5.4"`.

```yaml
compilerOptions:
  target: "5.4"
```

**`outDir`** (string, optional)

Output directory for compiled Lua files. If not specified, files are written next to source files.

```yaml
compilerOptions:
  outDir: dist
```

**`outFile`** (string, optional)

Bundle all output into a single file.

```yaml
compilerOptions:
  outFile: bundle.lua
```

**`outputFormat`** (string, default: `"readable"`)

Output format for generated Lua code. Options: `"readable"`, `"compact"`, `"minified"`.

```yaml
compilerOptions:
  outputFormat: readable
```

**`sourceMap`** (boolean, default: `false`)

Generate source maps for debugging.

```yaml
compilerOptions:
  sourceMap: true
```

**`noEmit`** (boolean, default: `false`)

Type check only, don't generate output files.

```yaml
compilerOptions:
  noEmit: true
```

#### Module System

**`moduleMode`** (string, default: `"require"`)

Module code generation mode. Options: `"require"` (separate files with `require()` calls), `"bundle"` (bundle all into one file).

```yaml
compilerOptions:
  moduleMode: require  # or "bundle"
```

**`modulePaths`** (array of strings)

Search paths for package imports.

```yaml
compilerOptions:
  modulePaths:
    - ./?.luax
    - ./lua_modules/?.luax
    - ./lua_modules/?/init.luax
```

**`enforceNamespacePath`** (boolean, default: `false`)

Require namespace declarations to match file paths.

```yaml
compilerOptions:
  enforceNamespacePath: true
```

#### Language Features

**`enableDecorators`** (boolean, default: `true`)

Enable decorator syntax (`@decorator`).

```yaml
compilerOptions:
  enableDecorators: true
```

**`allowNonTypedLua`** (boolean, default: `true`)

Allow importing plain `.lua` files without type annotations.

```yaml
compilerOptions:
  allowNonTypedLua: true
```

**`copyLuaToOutput`** (boolean, default: `false`)

Copy plain `.lua` files to output directory during compilation.

```yaml
compilerOptions:
  copyLuaToOutput: true
```

#### Diagnostics

**`pretty`** (boolean, default: `true`)

Pretty-print diagnostics with colors and context.

```yaml
compilerOptions:
  pretty: true
```

### File Patterns

**`include`** (array of glob patterns)

Files to compile. Supports glob wildcards.

```yaml
include:
  - "src/**/*.luax"
  - "lib/**/*.luax"
```

**`exclude`** (array of glob patterns)

Files to exclude from compilation.

```yaml
exclude:
  - "**/node_modules/**"
  - "**/dist/**"
  - "**/*.test.luax"
```

## Example Project Structure

```text
my-project/
├── luanext.config.yaml
├── src/
│   ├── main.luax
│   ├── utils/
│   │   ├── math.luax
│   │   └── string.luax
│   └── models/
│       └── user.luax
├── tests/
│   └── test_math.luax
└── dist/                  # compiled output (if outDir is set)
    ├── main.lua
    ├── utils/
    │   ├── math.lua
    │   └── string.lua
    └── models/
        └── user.lua
```

## Example Configurations

### Development Setup

```yaml
compilerOptions:
  strictNullChecks: true
  target: "5.4"
  outDir: dist
  sourceMap: true
  pretty: true

include:
  - "src/**/*.luax"

exclude:
  - "**/node_modules/**"
  - "**/dist/**"
```

### Production Build

```yaml
compilerOptions:
  strictNullChecks: true
  strictNaming: error
  noImplicitUnknown: true
  target: "5.1"              # for maximum compatibility
  outFile: bundle.lua        # single output file
  outputFormat: minified     # smaller file size
  moduleMode: bundle         # bundle all modules
  sourceMap: false           # no source maps in production

include:
  - "src/**/*.luax"

exclude:
  - "**/tests/**"
  - "**/node_modules/**"
```

### Type-Checking Only

```yaml
compilerOptions:
  strictNullChecks: true
  strictNaming: error
  noImplicitUnknown: true
  noEmit: true               # don't generate output

include:
  - "**/*.luax"
```

## Compiling a Project

Once you have a `luanext.config.yaml`, compile your project:

```bash
# Use configuration file
luanext --project luanext.config.yaml

# Or if luanext.config.yaml is in the current directory:
luanext
```

LuaNext automatically discovers the configuration file and compiles all included files.

## Command-Line Overrides

Command-line arguments override `luanext.config.yaml`:

```bash
# Override target version
luanext --project luanext.config.yaml --target 5.1

# Override output directory
luanext --project luanext.config.yaml --out-dir build

# Enable source maps
luanext --project luanext.config.yaml --source-map
```

## Watch Mode

Enable watch mode to automatically recompile on file changes:

```bash
luanext --project luanext.config.yaml --watch
```

The compiler monitors all files matching the `include` patterns and recompiles when changes are detected.

## Incremental Compilation

LuaNext caches type information and compiled output to speed up rebuilds. The cache is stored in `.luanext-cache/`.

Disable the cache with `--no-cache`:

```bash
luanext --project luanext.config.yaml --no-cache
```

Clear the cache by deleting the `.luanext-cache/` directory.

## Multi-Module Projects

For projects with multiple modules:

```text
my-app/
├── luanext.config.yaml
├── core/
│   ├── init.luax         # exports core API
│   └── internal.luax
├── plugins/
│   ├── plugin-a.luax
│   └── plugin-b.luax
└── main.luax             # imports core and plugins
```

**core/init.luax:**

```lua
import { helper } from "./internal"

export function coreFunction(): void
    helper()
end
```

**main.luax:**

```lua
import { coreFunction } from "./core"
import { pluginA } from "./plugins/plugin-a"

coreFunction()
pluginA()
```

Compile the project:

```bash
luanext --project luanext.config.yaml
```

LuaNext automatically determines the correct compilation order based on imports.

## Next Steps

- [Type System](../language/type-system.md) — Learn about types and type checking
- [Modules](../language/modules.md) — Import and export system
- [Configuration Reference](../reference/configuration.md) — Complete configuration documentation
