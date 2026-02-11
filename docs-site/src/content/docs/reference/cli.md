---
title: CLI Reference
---

# CLI Reference

Complete command-line interface reference for the LuaNext compiler.

## Synopsis

```bash
luanext [OPTIONS] <FILES>...
luanext --init
luanext --help
luanext --version
```

## Basic Usage

### Compile Single File

```bash
luanext main.luax
```

Compiles `main.luax` to `main.lua` in the same directory.

### Compile Multiple Files

```bash
luanext file1.luax file2.luax file3.luax
```

Compiles each file to corresponding `.lua` output.

### Compile with Glob Patterns

```bash
# Compile all .luax files in src/
luanext "src/**/*.luax"

# Compile multiple patterns
luanext "src/**/*.luax" "tests/**/*.luax"
```

**Note:** Use quotes around glob patterns to prevent shell expansion.

### Specify Output Directory

```bash
luanext main.luax --out-dir dist/
```

Compiles `main.luax` to `dist/main.lua`.

### Bundle to Single File

```bash
luanext "src/**/*.luax" --out-file bundle.lua
```

Compiles all files and concatenates them into `bundle.lua`.

## Options

### Input/Output

#### `<FILES>...`

Input files to compile. Supports glob patterns.

```bash
luanext main.luax utils.luax
luanext "src/**/*.luax"
```

#### `-p, --project <FILE>`

Path to `luanext.config.yaml` configuration file.

```bash
luanext --project ./config/luanext.config.yaml main.luax
```

If not specified, LuaNext looks for `luanext.config.yaml` in the current directory and parent directories.

#### `--out-dir <DIR>`

Output directory for compiled Lua files.

```bash
luanext main.luax --out-dir dist/
```

Creates `dist/main.lua` preserving directory structure.

#### `--out-file <FILE>`

Output file (concatenates all output into a single file).

```bash
luanext "src/**/*.luax" --out-file bundle.lua
```

Generates a single bundled file with all modules.

#### `--no-emit`

Type-check only, do not generate output files.

```bash
luanext main.luax --no-emit
```

Useful for CI/CD type checking without generating output.

### Target and Compatibility

#### `--target <VERSION>`

Target Lua version: `5.1`, `5.2`, `5.3`, or `5.4`.

```bash
luanext main.luax --target 5.1
luanext main.luax --target 5.4
```

**Default:** `5.4`

Affects:

- Integer handling (native in 5.3+, type-checked only in 5.1/5.2)
- Bitwise operators (native in 5.3+, library calls in 5.1/5.2)
- Standard library compatibility

See [Lua Targets](../guides/lua-targets.md) for details.

### Source Maps

#### `--source-map`

Generate source maps (`.lua.map` files).

```bash
luanext main.luax --source-map
```

Creates `main.lua` and `main.lua.map` for debugging.

#### `--inline-source-map`

Inline source map in output file as a comment.

```bash
luanext main.luax --inline-source-map
```

Embeds source map directly in `main.lua`:

```lua
-- ...generated code...
--# sourceMappingURL=data:application/json;base64,...
```

### Watch Mode

#### `-w, --watch`

Watch input files for changes and recompile automatically.

```bash
luanext "src/**/*.luax" --watch --out-dir dist/
```

Monitors files and recompiles on changes. Press `Ctrl+C` to exit.

**Features:**

- Incremental compilation (only changed files)
- Fast rebuild times
- Error recovery (continues watching after errors)

### Project Initialization

#### `--init`

Initialize a new LuaNext project.

```bash
luanext --init
```

Creates:

- `luanext.config.yaml` — Configuration file
- `main.luax` — Sample source file

### Type Checking Options

#### `--no-strict-null-checks`

Disable strict null checking (allow nil anywhere).

```bash
luanext main.luax --no-strict-null-checks
```

By default, LuaNext enforces explicit nil handling. This flag disables that.

#### `--no-implicit-unknown`

Disallow implicit `unknown` types (require explicit type annotations).

```bash
luanext main.luax --no-implicit-unknown
```

Forces all values to have known types or explicit `unknown` annotation.

#### `--strict-naming <LEVEL>`

Strict naming convention enforcement: `error`, `warning`, or `off`.

```bash
luanext main.luax --strict-naming error
luanext main.luax --strict-naming warning
```

- `error` — Naming violations are compilation errors
- `warning` — Naming violations are warnings
- `off` — No naming enforcement (default)

**Conventions checked:**

- Constants: `UPPER_SNAKE_CASE`
- Variables: `camelCase` or `snake_case`
- Types/Interfaces: `PascalCase`
- Functions: `camelCase` or `snake_case`

### Feature Flags

#### `--enable-decorators`

Enable decorator syntax.

```bash
luanext main.luax --enable-decorators
```

Allows using `@decorator` annotations on classes and members.

**Default:** Enabled (add `--no-enable-decorators` to disable)

### Module System

#### `--module-mode <MODE>`

Module code generation mode: `require` or `bundle`.

```bash
luanext main.luax --module-mode require
luanext main.luax --module-mode bundle
```

- `require` — Generate separate files with `require()` calls (default)
- `bundle` — Inline all modules into output file

#### `--module-paths <PATHS>`

Module search paths (comma-separated).

```bash
luanext main.luax --module-paths "lib/,node_modules/"
```

Adds additional directories to module resolution paths.

#### `--enforce-namespace-path`

Enforce that namespace declarations match file paths.

```bash
luanext main.luax --enforce-namespace-path
```

Example: `namespace MyApp.Utils` in `src/MyApp/Utils.luax` is valid,
but `namespace Other.Namespace` would be an error.

### Output Formatting

#### `--format <FORMAT>`

Output format: `readable`, `compact`, or `minified`.

```bash
luanext main.luax --format readable   # Default: formatted with indentation
luanext main.luax --format compact    # Minimal whitespace
luanext main.luax --format minified   # Single line, no whitespace
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

#### `--copy-lua-to-output`

Copy plain `.lua` files to output directory.

```bash
luanext "src/**/*.luax" --copy-lua-to-output --out-dir dist/
```

Copies non-LuaNext `.lua` files from source to output directory.

### Optimization

#### `--optimize`

Enable aggressive optimizations with whole-program analysis.

```bash
luanext main.luax --optimize --out-file optimized.lua
```

**Enabled optimizations:**

- Constant folding and propagation
- Dead code elimination
- Function inlining
- Loop optimizations
- Rich enum optimization
- Table preallocation
- Tail call optimization

**Note:** Requires `--out-file` or single file compilation for whole-program analysis.

#### `--no-optimize`

Disable all optimizations (raw transpilation).

```bash
luanext main.luax --no-optimize
```

Generates Lua code directly from AST without any optimization passes.

#### `--profile-optimizer`

Enable optimizer profiling (logs pass timings).

```bash
luanext main.luax --optimize --profile-optimizer
```

Outputs timing information for each optimization pass:

```
ConstantFoldingPass: 12.3ms
DeadCodeEliminationPass: 8.7ms
InlineExpansionPass: 15.2ms
...
```

#### `--no-parallel-optimization`

Disable parallel optimization (for benchmarking).

```bash
luanext main.luax --optimize --no-parallel-optimization
```

Runs optimization passes sequentially instead of in parallel.

#### `--no-tree-shake`

Disable tree shaking (for debugging).

```bash
luanext main.luax --optimize --no-tree-shake
```

Keeps all code, even unreachable functions/exports.

#### `--no-scope-hoist`

Disable scope hoisting (for debugging).

```bash
luanext main.luax --optimize --no-scope-hoist
```

Preserves module boundaries instead of merging scopes.

### Caching

#### `--no-cache`

Disable incremental compilation cache.

```bash
luanext main.luax --no-cache
```

Forces full recompilation, ignoring `.luanext-cache/` directory.

**Note:** Cache is stored in `.luanext-cache/` in the project root.

#### `--force-full-check`

Force full type check (disable incremental type checking).

```bash
luanext main.luax --force-full-check
```

Re-type-checks all files even if unchanged.

### Reflection

#### `--reflection <MODE>`

Reflection metadata mode: `selective`, `full`, or `none`.

```bash
luanext main.luax --reflection selective  # Default
luanext main.luax --reflection full
luanext main.luax --reflection none
```

- `selective` — Include metadata for decorated items only
- `full` — Include metadata for all types and functions
- `none` — No reflection metadata

**Metadata includes:**

- Type information
- Decorator annotations
- Parameter names
- Property types

### Diagnostics

#### `--pretty`

Pretty print diagnostics (default: enabled).

```bash
luanext main.luax --pretty
```

**With `--pretty`:**

```
error[E0001]: Type mismatch
  ┌─ main.luax:5:10
  │
5 │     const x: number = "hello"
  │              ^^^^^^   ^^^^^^^ expected number, found string
  │              │
  │              type annotation
```

**Without `--pretty` (`--no-pretty`):**

```
main.luax:5:10: error[E0001]: Type mismatch: expected number, found string
```

#### `--diagnostics`

Show diagnostic error codes in output.

```bash
luanext main.luax --diagnostics
```

Includes error codes like `[E0001]`, `[E0042]` in diagnostic messages.

### Information

#### `--help`

Display help information.

```bash
luanext --help
```

#### `--version`

Display version information.

```bash
luanext --version
```

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success (no errors) |
| `1` | Compilation error (type errors, syntax errors) |
| `2` | File not found or I/O error |
| `3` | Configuration error (invalid config file) |

## Configuration File

Instead of command-line options, use `luanext.config.yaml`:

```yaml
compilerOptions:
  target: "5.4"
  outDir: "dist"
  outFile: "bundle.lua"
  sourceMap: true
  inlineSourceMap: false
  noEmit: false
  strictNullChecks: true
  enableDecorators: true
  moduleMode: "require"
  format: "readable"
  optimize: false
  reflection: "selective"
  pretty: true
  copyLuaToOutput: false

include:
  - "src/**/*.luax"
  - "tests/**/*.luax"

exclude:
  - "**/node_modules/**"
  - "**/dist/**"
  - "**/*.test.luax"
```

Command-line options override config file values.

See [Configuration Reference](configuration.md) for complete options.

## Examples

### Basic Compilation

```bash
# Compile single file
luanext main.luax

# Compile to specific directory
luanext main.luax --out-dir build/

# Compile with source maps
luanext main.luax --source-map
```

### Project Compilation

```bash
# Use project configuration
luanext

# Compile entire project
luanext "src/**/*.luax" --out-dir dist/

# Bundle project to single file
luanext "src/**/*.luax" --out-file app.lua
```

### Watch Mode

```bash
# Watch for changes
luanext "src/**/*.luax" --watch --out-dir dist/

# Watch with optimizations
luanext "src/**/*.luax" --watch --optimize --out-file bundle.lua
```

### Type Checking

```bash
# Type-check only (no output)
luanext "src/**/*.luax" --no-emit

# Type-check with strict settings
luanext main.luax --no-emit --no-implicit-unknown --strict-naming error
```

### Optimization

```bash
# Optimize single file
luanext main.luax --optimize --out-file optimized.lua

# Optimize project bundle
luanext "src/**/*.luax" --optimize --out-file app.min.lua --format minified

# Profile optimizer performance
luanext main.luax --optimize --profile-optimizer
```

### Cross-Lua-Version

```bash
# Target Lua 5.1 (LuaJIT compatible)
luanext main.luax --target 5.1 --out-dir dist-5.1/

# Target Lua 5.4 (latest features)
luanext main.luax --target 5.4 --out-dir dist-5.4/
```

## Environment Variables

### `RUST_LOG`

Control logging verbosity.

```bash
# Info level (default)
RUST_LOG=info luanext main.luax

# Debug level (detailed logs)
RUST_LOG=debug luanext main.luax

# Trace level (very verbose)
RUST_LOG=trace luanext main.luax

# Silence logs
RUST_LOG=off luanext main.luax
```

### `NO_COLOR`

Disable colored output.

```bash
NO_COLOR=1 luanext main.luax
```

## Glob Patterns

LuaNext supports standard glob patterns for file selection:

| Pattern | Matches |
|---------|---------|
| `*` | Any characters except `/` |
| `**` | Any characters including `/` (recursive) |
| `?` | Single character |
| `[abc]` | Any character in set |
| `{a,b}` | Either `a` or `b` |

**Examples:**

```bash
# All .luax files in src/
luanext "src/*.luax"

# All .luax files in src/ and subdirectories
luanext "src/**/*.luax"

# Only files matching pattern
luanext "src/**/*{Utils,Helper}.luax"

# Multiple patterns
luanext "src/**/*.luax" "lib/**/*.luax"
```

## Tips and Best Practices

### Use Configuration Files

For projects with complex settings, use `luanext.config.yaml`:

```bash
# Automatically uses luanext.config.yaml
luanext

# Or specify config explicitly
luanext --project ./custom-config.yaml
```

### Watch Mode for Development

Use watch mode during development for fast feedback:

```bash
luanext "src/**/*.luax" --watch --out-dir dist/
```

### Optimization for Production

Enable optimizations for production builds:

```bash
luanext "src/**/*.luax" --optimize --out-file app.lua --format minified
```

### Type-Check in CI/CD

Use `--no-emit` for fast type checking in CI:

```bash
luanext "src/**/*.luax" --no-emit --diagnostics
```

### Debug Optimization Issues

If optimizations cause issues, disable specific passes:

```bash
# Disable tree shaking
luanext main.luax --optimize --no-tree-shake

# Disable scope hoisting
luanext main.luax --optimize --no-scope-hoist

# Disable all optimizations
luanext main.luax --no-optimize
```

## Troubleshooting

### "No input files specified"

**Problem:** No files match the pattern or no files provided.

**Solution:**

```bash
# Check glob pattern matches files
ls src/**/*.luax

# Use quotes around patterns
luanext "src/**/*.luax"
```

### "Configuration file not found"

**Problem:** `luanext.config.yaml` not found.

**Solution:**

```bash
# Initialize project
luanext --init

# Or specify config path
luanext --project path/to/config.yaml
```

### Compilation Errors

**Problem:** Type errors or syntax errors.

**Solution:**

```bash
# Use pretty diagnostics
luanext main.luax --pretty --diagnostics

# Check specific file
luanext problem-file.luax --no-emit
```

### Cache Issues

**Problem:** Stale cache causing incorrect behavior.

**Solution:**

```bash
# Clear cache and recompile
rm -rf .luanext-cache/
luanext main.luax

# Or disable cache
luanext main.luax --no-cache
```

## See Also

- [Configuration Reference](configuration.md) — Config file options
- [Migrating from Lua](../guides/migrating-from-lua.md) — Porting Lua code
- [Lua Targets](../guides/lua-targets.md) — Targeting different Lua versions
