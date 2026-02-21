# Module Resolution

Module registry, resolver, dependency graph, and path aliases.

## Overview

Module resolution determines how import paths map to files and how modules track their interdependencies. The system handles topological sorting, cycle detection, and incremental compilation.

**Files**: `crates/luanext-typechecker/src/module_resolver/`

## Module Registry

**File**: `registry.rs`

The `ModuleRegistry` is the central store for all module state:

```rust
struct ModuleRegistry {
    modules: FxHashMap<ModuleId, ModuleEntry>,
}
```

### ModuleId

Unique identifier for a module, typically derived from its file path.

### ModuleEntry

Tracks a single module's lifecycle:

- **Status**: `Parsed` → `Checked` → `Invalid`
- **Exports**: `ModuleExports` — map of exported symbol names to their types
- **Diagnostics**: Collected during type checking
- **Type-only flag**: Whether the module is imported as type-only

### Lifecycle

1. `register_parsed()` — called after parsing, before type checking
2. Type checking runs (all 5 phases)
3. `mark_checked()` — called after successful type checking

### Cross-Module Symbol Sharing

`[NOTE: UNSAFE]` The registry uses `unsafe transmute` to convert `Symbol<'arena>` → `Symbol<'static>` via `symbol_to_static()` in `module_phase.rs`. This is necessary because different modules are parsed in different arenas with different lifetimes, but the registry needs to hold symbols from all modules.

### Type Check Depth Tracking

Circuit-breaking for lazy resolution:

- `increment_type_check_depth()` / `decrement_type_check_depth()`
- `get_type_check_depth()` / `is_ready_for_type_checking()`
- Prevents infinite recursion when modules have circular type dependencies

## Module Resolver

**File**: `crates/luanext-typechecker/src/module_resolver/`

Resolves import paths to file paths:

### Resolution Order

1. **Path aliases** — TypeScript-style path mapping (e.g., `@app/*` → `./src/*`)
2. **Relative imports** — `./sibling`, `../parent`
3. **Module paths** — search directories (like `node_modules`)
4. **Directory imports** — try `index.luax` for directory paths

### Path Aliases

**File**: `path_aliases.rs`

TypeScript-compatible path mapping from config:

```yaml
# luanext.config.yaml
paths:
  "@app/*": ["./src/*"]
  "@lib/*": ["./lib/*"]
```

## Dependency Graph

**File**: `dependency_graph.rs`

Tracks import relationships between modules:

### Edge Kinds

```rust
enum EdgeKind {
    Value,      // runtime import (generates require())
    TypeOnly,   // type-only import (erased at codegen)
    Dynamic,    // dynamic import
}
```

### Topological Sort

Modules are compiled in dependency order via topological sort:

- **Value edges** and **TypeOnly edges** are both followed for ordering
- Only **Value cycles** are rejected (circular runtime imports are invalid)
- **TypeOnly cycles** are gracefully degraded — return `Unknown` instead of failing

`[NOTE: BEHAVIOR]` TypeOnly edges were originally excluded from topological sort ordering, causing compilation failures. The fix was to follow TypeOnly edges for ordering while only rejecting Value cycles.

### Import Scanner

**File**: `crates/luanext-typechecker/` (import scanning logic)

The `ImportScanner` discovers dependencies before type checking:

- Scans import statements for module dependencies
- Also scans `export ... from` statements to discover re-export dependencies
- `parse_export_statement()` was added to handle re-export edge discovery

## Module Errors

**File**: `error.rs`

```rust
enum ModuleError {
    TypeCheckInProgress,          // circular dependency during checking
    ExportTypeMismatch,           // export type doesn't match declaration
    RuntimeImportOfTypeOnly,      // runtime use of type-only import
    CircularReExport,             // re-export cycle detected
    ReExportChainTooDeep,         // re-export chain > 10 hops
    TypeOnlyReExportAsValue,      // type-only re-export used as value
}
```

## Lazy Type Check Callback

```rust
trait LazyTypeCheckCallback: Send + Sync {
    fn type_check_module(&self, module_id: &ModuleId) -> Result<(), ModuleError>;
}
```

Enables on-demand type checking of dependencies when resolving imports. The callback triggers type checking of a dependency that hasn't been checked yet.

## Import Type Resolution

`resolve_import_type()` resolves the type of an imported symbol:

1. Look up the source module in the registry
2. Find the exported symbol by name
3. Validate type-only constraints (`is_type_only_import` parameter)
4. Apply generic type arguments if present (`apply_type_arguments()`)
5. Return the resolved type

Returns proper errors instead of falling back to `Unknown`.

## Multi-File Compilation

Key integration details for multi-file compilation:

- **Shared StringInterner**: Single interner shared across all files for consistent `StringId` values
- **Arena per file**: Each file gets its own `bumpalo::Bump` arena
- **Box::leak for multi-file**: Multi-file mode uses `Box::leak` instead of pooled arenas to avoid use-after-free

`[NOTE: BEHAVIOR]` Using pooled arenas (`with_pooled_arena`) in multi-file mode caused use-after-free bugs because module symbols outlived the arena. The fix uses `Box::leak` for permanent allocation.

## Cross-References

- [Modules](modules.md) — import/export syntax
- [Type Checking](../language/type-checking.md) — module phase
- [Incremental Cache](../compiler/incremental-cache.md) — caching resolved modules
- [Optimizer LTO](../compiler/optimizer-lto.md) — module graph for dead code elimination
