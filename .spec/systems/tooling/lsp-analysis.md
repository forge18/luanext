# LSP Analysis

Symbol index, cross-file support, semantic tokens, and inlay hints.

## Overview

Analysis infrastructure that powers LSP features. Includes cross-file symbol tracking, semantic highlighting, and type-based hints.

**Files**: `crates/luanext-lsp/src/core/analysis/`, `crates/luanext-lsp/src/features/semantic/`, `crates/luanext-lsp/src/features/hints/`

## Symbol Index

**File**: `core/analysis/symbol_index.rs`

Maintains a workspace-wide index of symbols for cross-file features:

### ExportInfo

```rust
struct ExportInfo {
    name: String,
    kind: SymbolKind,
    span: Span,
    is_type_only: bool,
    // ... type information
}
```

### ImportInfo

```rust
struct ImportInfo {
    local_name: String,
    imported_name: String,
    source: String,
    is_type_only: bool,
    span: Span,
}
```

### Indexing

The symbol index is built/updated when documents are opened or changed:

1. Parse the document
2. Extract all exports (declarations, named exports, re-exports)
3. Extract all imports (named, namespace, type-only)
4. Store in the index keyed by file URI

### Type-Only Tracking

Both `ExportInfo` and `ImportInfo` have `is_type_only: bool`:

- Set for `import type { ... }` clauses
- Set for `export type { ... }` declarations
- `is_declaration_type_only()` helper marks exports of interfaces and type aliases

### Import Clause Indexing

Full `ImportClause::TypeOnly` indexing in `symbol_index.rs`:

- All specifiers in type-only import clauses are marked `is_type_only: true`
- Mixed imports: only the type-only subset is marked

## Cross-File Support

Cross-file operations use the symbol index to resolve symbols across module boundaries:

| Feature | Cross-File Status |
| ------- | ----------------- |
| Go-to-definition | Implemented — follows imports to source |
| Rename | Implemented — renames across import/export |
| Hover | Partial — may not resolve all imported types |
| References | Partial — may miss re-export chain references |
| Completion | Partial — may not show all exported symbols |

`[NOTE: PARTIAL]` Cross-file support is tested via dedicated test files in `crates/luanext-lsp/tests/` (definition, hover, references, completion, rename — 41 cross-file tests total). These tests document both implemented features and known gaps.

### Test Infrastructure

Tests use `LspTestWorkspace` from `test_utils.rs` with `ModuleId` from the type checker to simulate multi-file workspaces.

## Semantic Tokens

**File**: `features/semantic/semantic_tokens.rs`

Provides rich syntax highlighting data to the editor:

### Token Types

- Keywords, operators, strings, numbers
- Functions, variables, parameters
- Types, interfaces, classes, enums
- Decorators
- Comments

### Incremental Updates

**File**: `features/semantic/incremental.rs`

Supports delta updates (`textDocument/semanticTokens/full/delta`):

- Tracks previous token set
- Computes delta (insertions, deletions, modifications)
- Sends only changed tokens for efficiency

## Inlay Hints

**File**: `features/hints/inlay_hints.rs`

Shows inline type and parameter information:

### Type Hints

Show inferred types for variables without explicit annotations:

```lua
const x = 42        -- shows ": number" after x
const s = "hello"   -- shows ": string" after s
```

### Parameter Hints

Show parameter names at call sites:

```lua
greet("Alice", 30)  -- shows "name:" before "Alice", "age:" before 30
```

## Heuristics

**File**: `core/heuristics.rs`

Type inference heuristics for LSP-specific scenarios:

- Incomplete expressions (while user is typing)
- Partial type information
- Best-effort resolution

## Cross-References

- [LSP Architecture](lsp-architecture.md) — server and document management
- [LSP Features](lsp-features.md) — feature implementations that consume analysis
- [Type Checking](../language/type-checking.md) — type information source
- [Module Resolution](../features/module-resolution.md) — module registry for cross-file
