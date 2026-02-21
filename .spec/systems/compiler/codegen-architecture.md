# Codegen Architecture

CodeGenerator, Emitter, builder pattern, and output formats.

## Overview

The code generator transforms a type-checked AST into Lua source code. It uses a strategy pattern for target-specific behavior and a builder pattern for configuration.

**Files**: `crates/luanext-core/src/codegen/`

## CodeGenerator

The central code generation struct. Created via `CodeGeneratorBuilder`.

### Configuration

| Field | Type | Default | Purpose |
| ----- | ---- | ------- | ------- |
| `interner` | `Arc<StringInterner>` | (required) | Resolve StringId to strings |
| `target` | `LuaTarget` | `Lua54` | Lua version target |
| `mode` | `CodeGenMode` | `Require` | Module output mode |
| `optimization_level` | `OptimizationLevel` | `None` | Controls codegen optimizations |
| `output_format` | `OutputFormat` | `Readable` | Whitespace/indentation style |
| `source_map` | `Option<String>` | `None` | Source file name for maps |
| `reflection_mode` | `ReflectionMode` | `None` | Type reflection generation |
| `whole_program_analysis` | `Option<WholeProgramAnalysis>` | `None` | Cross-module analysis data |
| `reachable_exports` | `Option<HashSet<String>>` | `None` | LTO export filtering |
| `alias_require_map` | `HashMap<String, String>` | `{}` | Path alias resolution |

### CodeGenMode

```rust
enum CodeGenMode {
    Require,                     // separate files with require()
    Bundle { entry: String },    // single file with __require
}
```

### LuaTarget

```rust
enum LuaTarget {
    Lua51, Lua52, Lua53, Lua54, Lua55, LuaJIT,
}
```

Default: `Lua54`

### ReflectionMode

```rust
enum ReflectionMode {
    None,        // no reflection metadata
    Selective,   // only for modules importing @std/reflection
    Full,        // all classes
}
```

## Builder Pattern

```rust
let generator = CodeGeneratorBuilder::new(interner)
    .target(LuaTarget::Lua54)
    .source_map("main.luax".to_string())
    .optimization_level(OptimizationLevel::Moderate)
    .output_format(OutputFormat::Readable)
    .build();
```

All fields except `interner` have defaults. The builder validates configuration before producing a `CodeGenerator`.

## Emitter

**File**: `emitter.rs`

The `Emitter` handles writing Lua text with indentation tracking:

| Method | Purpose |
| ------ | ------- |
| `write(s)` | Append text |
| `writeln(s)` | Append text + newline |
| `write_indent()` | Write current indentation |
| `indent()` | Increase indentation level |
| `dedent()` | Decrease indentation level |
| `newline()` | Write a newline |
| `output()` | Get the generated string |

The emitter respects `OutputFormat`:

| Format | Behavior |
| ------ | -------- |
| `Readable` | Full indentation, newlines, spacing |
| `Compact` | Minimal whitespace, single spaces |
| `Minified` | No unnecessary whitespace |

## Generation Pipeline

The `generate()` method on `CodeGenerator` takes a `Program<'arena>` and produces a Lua string:

1. **Preamble**: Emit target-specific preamble (e.g., bitwise library for Lua 5.1)
2. **Tree shaking**: Remove unreachable code (if enabled)
3. **Scope hoisting**: Hoist variable declarations (bundle mode)
4. **Statements**: Generate each statement in order
5. **Exports**: Emit module return table (Require mode)
6. **Source map**: Finalize source map if enabled

## Tree Shaking

**File**: `tree_shaking.rs`

Removes unreachable code from the output:

- Unused local variables and functions
- Dead branches (constant conditions)
- Type-only declarations (interfaces, type aliases)

`[NOTE: BEHAVIOR]` Tree shaking runs during codegen, not during optimization. It operates on the AST level, removing statements that have no runtime effect.

## Scope Hoisting

**File**: `scope_hoisting.rs`

In bundle mode, hoists `local` declarations to the module scope to avoid re-declaration:

- Functions are hoisted to the top
- Variables are hoisted to their first assignment
- Global variables (`VariableKind::Global`) are NOT hoisted

## Source Map Integration

**File**: `sourcemap.rs`

During codegen, the emitter tracks source positions and builds a `SourceMapBuilder`:

- Maps generated Lua positions back to original `.luax` positions
- Emitted as separate `.map` file or inline data URI

## Cross-References

- [Codegen Statements](codegen-statements.md) — statement-level code generation
- [Codegen Expressions](codegen-expressions.md) — expression-level code generation
- [Codegen Targets](codegen-targets.md) — target-specific strategies
- [Source Maps](sourcemap.md) — source map generation details
