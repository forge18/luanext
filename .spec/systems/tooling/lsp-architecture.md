# LSP Architecture

Server setup, message handler, dependency injection, and document management.

## Overview

The LuaNext LSP server provides IDE features over the Language Server Protocol. It uses `lsp-server` and `lsp-types` crates for protocol communication.

**Files**: `crates/luanext-lsp/src/`

## Server Entry Point

**File**: `main.rs`

The LSP server:

1. Opens stdio connection (`lsp-server` crate)
2. Performs capability negotiation
3. Enters the main event loop
4. Dispatches requests/notifications to the message handler

## Message Handler

**File**: `message_handler.rs`

`BasicMessageHandler` routes LSP messages to feature providers:

### Request Handlers

| LSP Request | Feature |
| ----------- | ------- |
| `textDocument/completion` | Completion |
| `textDocument/hover` | Hover |
| `textDocument/definition` | Go-to-definition |
| `textDocument/references` | Find references |
| `textDocument/rename` | Rename |
| `textDocument/codeAction` | Code actions |
| `textDocument/signatureHelp` | Signature help |
| `textDocument/formatting` | Document formatting |
| `textDocument/rangeFormatting` | Range formatting |
| `textDocument/foldingRange` | Folding ranges |
| `textDocument/selectionRange` | Selection ranges |
| `textDocument/documentSymbol` | Document symbols |
| `textDocument/semanticTokens/full` | Semantic tokens |
| `textDocument/semanticTokens/full/delta` | Incremental semantic tokens |
| `textDocument/inlayHint` | Inlay hints |

### Notification Handlers

| LSP Notification | Action |
| ---------------- | ------ |
| `textDocument/didOpen` | Parse document, update cache |
| `textDocument/didChange` | Incremental re-parse |
| `textDocument/didSave` | Full re-check |
| `textDocument/didClose` | Remove from cache |

## Dependency Injection

**File**: `di/`

The LSP uses DI for testability:

### Traits

| Trait | Purpose | File |
| ----- | ------- | ---- |
| `DiagnosticsProviderTrait` | Publish diagnostics | `traits/diagnostics.rs` |
| `TypeAnalysisTrait` | Type information queries | `traits/type_analysis.rs` |
| `ModuleResolutionTrait` | Module path resolution | `traits/module_resolution.rs` |

### Testing

**File**: `testing/`

Mock providers implement the traits for unit testing:

```rust
struct MockDiagnosticsProvider { ... }
struct MockTypeAnalysis { ... }
```

## Document Management

**File**: `core/document.rs`

```rust
struct Document {
    uri: Url,
    version: i32,
    content: String,
    parsed: Option<ParsedState>,
}
```

Documents are parsed on open/change. The parsed state includes the AST and any diagnostics.

### Document Cache

**File**: `core/cache.rs`

Manages open documents and their parsed state:

- `open(uri, content)` — parse and cache
- `update(uri, changes)` — incremental re-parse
- `close(uri)` — remove from cache
- `get(uri)` — retrieve cached document

## Arena Pool

**File**: `arena_pool.rs`

Long-lived LSP processes reuse arenas:

- Pool of `Bump` arenas to avoid repeated allocation
- Arenas are reset (not dropped) when documents are re-parsed

## Protocol Connection

**File**: `protocol/`

Wraps `lsp-server` connection handling:

- Stdio transport (stdin/stdout)
- Request/response correlation
- Notification dispatch

## Diagnostics

**File**: `core/diagnostics.rs`

Diagnostics are published after parsing and type checking:

- Syntax errors from parser
- Type errors from type checker
- Published via `textDocument/publishDiagnostics` notification

## Metrics

**File**: `core/metrics.rs`

Performance tracking for LSP operations:

- Parse time
- Type check time
- Feature response time

## Cross-References

- [LSP Features](lsp-features.md) — individual feature implementations
- [LSP Analysis](lsp-analysis.md) — symbol index and cross-file support
- [Incremental Parsing](../compiler/incremental-parsing.md) — `parse_incremental()` integration
