# CLI

CLI flags, argument parsing, compilation pipeline orchestration, and watch mode.

## Overview

The LuaNext CLI (`luanext`) compiles `.luax` files to Lua. It handles configuration loading, file discovery, parallel compilation, and output writing.

**File**: `crates/luanext-cli/src/main.rs` (~1800+ lines)
**Argument parsing**: `clap` crate

## Usage

```bash
luanext [OPTIONS] [FILES...]
luanext --project luanext.config.yaml
luanext src/**/*.luax --out-dir dist --target 5.4
luanext --init    # create luanext.config.yaml
luanext --watch   # watch mode
```

## CLI Flags

### Input

| Flag | Description |
| ---- | ----------- |
| `[FILES...]` | Source files (positional, glob patterns) |
| `--project <path>` | Config file path (default: `luanext.config.yaml`) |

### Output

| Flag | Description |
| ---- | ----------- |
| `--out-dir <dir>` | Output directory |
| `--out-file <file>` | Single bundled output file |
| `--format <fmt>` | Output format: readable/compact/minified |
| `--emit` | Emit to stdout instead of files |
| `--no-emit` | Type check only, no output |
| `--source-map` | Generate source maps |
| `--inline-source-map` | Embed source maps in output |

### Target

| Flag | Description |
| ---- | ----------- |
| `--target <ver>` | Lua target: 5.1, 5.2, 5.3, 5.4, 5.5, jit |

### Compiler

| Flag | Description |
| ---- | ----------- |
| `--strict-naming` | Enable naming convention enforcement |
| `--no-strict-null-checks` | Disable null safety |
| `--no-implicit-unknown` | Disallow implicit unknown |
| `--enable-decorators` | Enable decorator syntax |

### Optimization

| Flag | Description |
| ---- | ----------- |
| `--optimize <level>` | Optimization level: none/minimal/moderate/aggressive |
| `--no-optimize` | Disable all optimizations |
| `--profile-optimizer` | Print optimization timing info |
| `--no-parallel-optimization` | Disable parallel optimization |

### Modules

| Flag | Description |
| ---- | ----------- |
| `--module-mode <mode>` | require or bundle |
| `--module-paths <paths>` | Module search paths |
| `--enforce-namespace-path` | Namespace must match file path |
| `--copy-lua-to-output` | Copy plain .lua files to output |

### Incremental

| Flag | Description |
| ---- | ----------- |
| `--no-cache` | Disable compilation cache |
| `--force-full-check` | Force full recheck (ignore cache) |

### Debugging

| Flag | Description |
| ---- | ----------- |
| `--no-tree-shake` | Disable tree shaking |
| `--no-scope-hoist` | Disable scope hoisting |
| `--diagnostics` | Diagnostic verbosity level |
| `--reflection <mode>` | Reflection: selective/full/none |

### Other

| Flag | Description |
| ---- | ----------- |
| `--init` | Create default config file |
| `--watch` | Watch mode for file changes |
| `--pretty` | Pretty-print diagnostics |

## Compilation Pipeline

The `main()` function orchestrates the full pipeline:

### 1. Config Loading

```text
CLI args + luanext.config.yaml → CompilerConfig
CliOverrides merged into config
```

### 2. File Discovery

```text
Glob pattern expansion → Vec<PathBuf>
Exclude patterns filtered out
```

### 3. Parallel Parsing (rayon)

Each file is parsed in parallel:

```text
File → Lexer → Parser → ParsedModule<'arena>
```

- Separate `bumpalo::Bump` arena per file
- Shared `Arc<StringInterner>` across all files

### 4. Module Registration

```text
ImportScanner → dependency edges
Topological sort → compilation order
ModuleRegistry.register_parsed() for each module
```

### 5. Type Checking (incremental)

```text
For each module (in topo order):
  Cache check → skip if valid
  5-phase type check
  Store in cache
  ModuleRegistry.mark_checked()
```

### 6. Module Graph (O2+ only)

```text
Build ModuleGraph from type-checked modules
Track exports, imports, re-exports
```

### 7. Optimization

```text
Program → MutableProgram
Apply passes based on OptimizationLevel
Fixed-point iteration (max 10 rounds)
MutableProgram → Program
```

### 8. Code Generation (parallel)

```text
CodeGeneratorBuilder → CodeGenerator
LTO passes at O2+ (inside codegen closure)
Generate Lua code per file
```

### 9. Output

```text
Write .lua files to outDir
Or concatenate to outFile
Or emit to stdout
Generate .map files if source maps enabled
```

## Watch Mode

File system monitoring via the `notify` crate:

- Watches source files for changes
- Triggers incremental recompilation
- Uses debouncing to coalesce rapid changes

## Init Command

`luanext --init` creates a default `luanext.config.yaml` in the current directory.

`[NOTE: BEHAVIOR]` Multi-file compilation uses `Box::leak` for arenas instead of pooled arenas to avoid use-after-free. This means arena memory is not reclaimed until process exit.

## Cross-References

- [Config](config.md) — configuration format and options
- [Codegen Architecture](../compiler/codegen-architecture.md) — code generator setup
- [Incremental Cache](../compiler/incremental-cache.md) — cache integration
- [Module Resolution](../features/module-resolution.md) — module dependency ordering
