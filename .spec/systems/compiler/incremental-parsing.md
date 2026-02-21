# Incremental Parsing

`parse_incremental()`, multi-arena management, statement caching, and consolidation.

## Overview

Incremental parsing reuses previously parsed AST statements when only part of a file changes, avoiding full re-parsing. This is critical for LSP responsiveness during editing.

**Files**: `crates/luanext-parser/src/incremental/`, `crates/luanext-parser/src/parser/mod.rs`

## parse_incremental()

Three optimization paths based on state:

### Path 1: First Parse

No previous parse state exists. Falls through to full parse.

### Path 2: No Edits

The source text is unchanged since last parse. Returns the cached AST directly — zero parsing work.

### Path 3: Partial Edits

Some source text changed. For each previously cached statement:

1. Check if the statement's span overlaps with any edit range (`is_statement_clean()`)
2. Clean statements are reused from cache
3. Dirty statements are re-parsed from the new source

`[NOTE: BEHAVIOR]` No offset adjustment is performed on cached statements. The `is_statement_clean()` check uses simple overlap validation between the statement's span and edit ranges. This means cached statements retain their original byte offsets even if earlier text was inserted/deleted.

## Multi-Arena System

Incremental parsing uses multiple arenas to avoid copying AST nodes:

- **Max 3 active arenas** — each incremental parse creates a new arena for newly parsed statements
- **Old arenas kept alive** — cached statements reference their original arena
- **Consolidation** triggers when:
  - More than 3 arenas accumulated, OR
  - Every 10 parses (periodic GC)

### Arena Consolidation

`consolidate_all()` clones all statements into a single fresh arena:

1. Allocate new arena
2. For each cached statement: `clone_statement_to_arena(stmt, new_arena)`
3. Drop all old arenas
4. Replace cache with cloned statements in new arena

### Statement Cloning

`clone_statement_to_arena()` uses `Statement::clone()` + `unsafe transmute` for lifetime casting:

```rust
// Clone the statement (deep copy)
let cloned = stmt.clone();
// Transmute lifetime from old arena to new arena
unsafe { std::mem::transmute::<Statement<'old>, Statement<'new>>(cloned) }
```

`[NOTE: UNSAFE]` The `transmute` is safe because the cloned statement's data is fully owned (via `Clone`) and the new arena outlives the returned reference.

## Cache Structure

**File**: `incremental/cache.rs`

The incremental cache stores:

- Previous source text (for change detection)
- Cached statements with their arenas
- Parse count (for periodic consolidation trigger)
- Arena list

### Garbage Collection

`collect_garbage()` runs when arenas exceed limit or parse count threshold:

```lua
if arena_count > 3 || parse_count % 10 == 0 {
    consolidate_all()
}
```

## Integration

### LSP Usage

The LSP server calls `parse_incremental()` on every `textDocument/didChange` notification:

1. Receives text edits (ranges + new text)
2. Applies edits to get new source
3. Calls `parse_incremental()` with edits
4. Gets back AST with mix of cached and fresh statements

### CLI Usage

The CLI typically does full parses (no incremental). Incremental parsing is primarily an LSP optimization.

## Cross-References

- [Parser](../language/parser.md) — parser architecture
- [Incremental Cache](incremental-cache.md) — compilation-level caching
- [LSP Architecture](../tooling/lsp-architecture.md) — LSP document management
