# Config

`luanext.config.yaml` format, CompilerConfig, CompilerOptions, and CLI overrides.

## Overview

LuaNext configuration uses a YAML file (`luanext.config.yaml`) with TypeScript-inspired naming conventions. CLI flags override file configuration via `CliOverrides`.

**File**: `crates/luanext-typechecker/src/cli/config.rs`

## Configuration File

```yaml
# luanext.config.yaml
compilerOptions:
  target: "5.4"
  outDir: "dist"
  sourceMap: true
  strictNullChecks: true
  strictNaming: "error"
  enableDecorators: true
  optimizationLevel: "moderate"
  outputFormat: "readable"
  moduleMode: "require"
  modulePaths:
    - "./?.luax"
    - "./lua_modules/?.luax"
  paths:
    "@/*": ["src/*"]

include:
  - "**/*.luax"
exclude:
  - "**/node_modules/**"
  - "**/dist/**"
```

## CompilerConfig

```rust
struct CompilerConfig {
    compiler_options: CompilerOptions,
    include: Vec<String>,        // glob patterns, default: ["**/*.luax"]
    exclude: Vec<String>,        // glob patterns, default: ["**/node_modules/**", "**/dist/**"]
}
```

### Loading

- `CompilerConfig::from_file(path)` — load from YAML
- `CompilerConfig::init_file(path)` — create default config file
- `config.merge(&overrides)` — apply CLI overrides

## CompilerOptions

All fields use camelCase in YAML (via `#[serde(rename_all = "camelCase")]`).

### Type Checking

| Option | Type | Default | Description |
| ------ | ---- | ------- | ----------- |
| `strictNullChecks` | `bool` | `true` | Enforce null safety |
| `strictNaming` | `StrictLevel` | `"error"` | Naming convention enforcement |
| `noImplicitUnknown` | `bool` | `false` | Disallow implicit unknown types |
| `noExplicitUnknown` | `bool` | `false` | Disallow explicit unknown types |
| `enableDecorators` | `bool` | `true` | Enable decorator syntax |

### Target

| Option | Type | Default | Description |
| ------ | ---- | ------- | ----------- |
| `target` | `LuaVersion` | `"5.4"` | Lua version target |

#### LuaVersion Values

| Value | Enum |
| ----- | ---- |
| `"5.1"` | `Lua51` |
| `"5.2"` | `Lua52` |
| `"5.3"` | `Lua53` |
| `"5.4"` | `Lua54` |
| `"5.5"` | `Lua55` |
| `"jit"` | `LuaJIT` |
| `"auto"` | `Auto` — detect via `lua -v` |

`[NOTE: BEHAVIOR]` `Auto` runs `lua -v` to detect the system Lua version. Falls back to Lua 5.4 if detection fails.

### Output

| Option | Type | Default | Description |
| ------ | ---- | ------- | ----------- |
| `outDir` | `Option<String>` | `None` | Output directory |
| `outFile` | `Option<String>` | `None` | Single output file (bundle) |
| `sourceMap` | `bool` | `false` | Generate source maps |
| `noEmit` | `bool` | `false` | Type check only, no output |
| `outputFormat` | `OutputFormat` | `"readable"` | Whitespace style |
| `pretty` | `bool` | `true` | Pretty-print diagnostics |

#### OutputFormat Values

- `"readable"` — full indentation and newlines
- `"compact"` — minimal whitespace
- `"minified"` — no unnecessary whitespace

### Modules

| Option | Type | Default | Description |
| ------ | ---- | ------- | ----------- |
| `moduleMode` | `ModuleMode` | `"require"` | `require` or `bundle` |
| `modulePaths` | `Vec<String>` | `["./?.luax", ...]` | Module search paths |
| `enforceNamespacePath` | `bool` | `false` | Namespace must match file path |
| `allowNonTypedLua` | `bool` | `true` | Allow importing `.lua` files |
| `copyLuaToOutput` | `bool` | `false` | Copy `.lua` files to outDir |

### Optimization

| Option | Type | Default | Description |
| ------ | ---- | ------- | ----------- |
| `optimizationLevel` | `OptimizationLevel` | `"auto"` | O0-O3 or auto |

#### OptimizationLevel Values

- `"none"` — O0, no optimizations
- `"minimal"` — O1, safe transforms
- `"moderate"` — O2, standard optimizations
- `"aggressive"` — O3, whole-program analysis
- `"auto"` — Minimal in debug, Moderate in release

### Path Aliases

| Option | Type | Default | Description |
| ------ | ---- | ------- | ----------- |
| `baseUrl` | `Option<String>` | `None` | Base directory for path resolution |
| `paths` | `HashMap<String, Vec<String>>` | `{}` | Path alias mappings |

Path alias syntax (TypeScript-compatible):

```yaml
paths:
  "@/*": ["src/*"]           # @/foo → src/foo
  "@components/*": ["src/components/*"]
  "@utils": ["src/shared/utils"]   # exact match
  "*": ["./vendor/*", "./types/*"] # fallback with multiple targets
```

### StrictLevel

```rust
enum StrictLevel {
    Off,       // no enforcement
    Warning,   // warn but continue
    Error,     // fail compilation
}
```

## CLI Overrides

```rust
struct CliOverrides {
    // All fields are Option<T> - only set values override config
    strict_null_checks: Option<bool>,
    strict_naming: Option<StrictLevel>,
    target: Option<LuaVersion>,
    out_dir: Option<String>,
    out_file: Option<String>,
    source_map: Option<bool>,
    no_emit: Option<bool>,
    module_mode: Option<ModuleMode>,
    optimization_level: Option<OptimizationLevel>,
    output_format: Option<OutputFormat>,
    // ... etc
}
```

Merge logic: `config.merge(&overrides)` — only `Some` values override. `None` values preserve file config.

## Default Module Paths

```text
./?.luax
./lua_modules/?.luax
./lua_modules/?/init.luax
```

The `?` is replaced with the module name during resolution.

## Cross-References

- [CLI](cli.md) — CLI flag definitions
- [Codegen Targets](../compiler/codegen-targets.md) — LuaTarget enum
- [Optimizer Architecture](../compiler/optimizer-architecture.md) — optimization levels
- [Module Resolution](../features/module-resolution.md) — path alias resolution
