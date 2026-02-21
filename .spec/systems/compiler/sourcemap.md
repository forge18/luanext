# Source Maps

Source map V3 generation, VLQ encoding, builder, and position translation.

## Overview

Source maps allow debugging tools to map positions in generated Lua code back to original LuaNext source positions. Implementation follows the Source Map V3 specification.

**Files**: `crates/luanext-sourcemap/src/`

## SourceMapBuilder

Incrementally builds a source map during code generation:

| Method | Purpose |
| ------ | ------- |
| `add_mapping(gen_line, gen_col, src_line, src_col)` | Record a position mapping |
| `add_source(filename)` | Register a source file |
| `add_name(name)` | Register an identifier name |
| `build()` | Produce the final SourceMap |

The builder accumulates mappings as codegen emits Lua code. Each significant AST node records a mapping from its generated position to its original `Span`.

## SourceMap

The final source map data structure:

```rust
struct SourceMap {
    version: u32,          // always 3
    file: Option<String>,  // generated file name
    sources: Vec<String>,  // source file names
    names: Vec<String>,    // identifier names
    mappings: String,      // VLQ-encoded mapping data
}
```

### Serialization

- **JSON**: Standard source map JSON format
- **Data URI**: `data:application/json;base64,...` for inline embedding
- **Inline comment**: `--# sourceMappingURL=data:...` appended to Lua output

## VLQ Encoding

Variable-Length Quantity encoding for compact position representation:

Each mapping segment encodes (all as VLQ-encoded deltas from previous values):

1. Generated column
2. Source file index
3. Original line
4. Original column
5. Name index (optional)

VLQ uses base64 characters with a continuation bit for multi-byte values. Segments are separated by commas, lines by semicolons.

## Position Translation

**File**: `translator.rs`

`PositionTranslator` / `SourceMapLoader` for consuming source maps:

| Method | Purpose |
| ------ | ------- |
| `translate(gen_line, gen_col)` | Map generated position → original position |
| `load(json)` | Parse a source map from JSON |

Used by debugging tools and error reporters to show original source locations.

## Multi-Source Support

Source maps can reference multiple source files (common in bundle mode):

- Each source gets an index in the `sources` array
- Mappings reference source files by index
- Enables debugging bundled output with multiple original files

## Integration

### CLI Flags

- `--source-map` — Generate `.map` file alongside `.lua` output
- `--inline-source-map` — Embed source map as data URI comment in `.lua` output

### Codegen Integration

The codegen `Emitter` optionally tracks positions and feeds them to `SourceMapBuilder`. Source map generation adds minimal overhead to codegen.

## Cross-References

- [Codegen Architecture](codegen-architecture.md) — emitter integration
- [CLI](../tooling/cli.md) — source map CLI flags
