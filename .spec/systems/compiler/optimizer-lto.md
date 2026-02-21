# Link-Time Optimization (LTO)

Module graph construction, reachability analysis, and cross-module elimination passes.

## Overview

LTO performs whole-program optimizations across module boundaries. It operates in two phases: module graph construction (after type checking) and per-module LTO passes (during codegen). The module graph tracks exports, imports, re-exports, and reachability to enable dead code elimination at the module level.

**Files**: `crates/luanext-core/src/optimizer/analysis/module_graph.rs`, `crates/luanext-core/src/optimizer/passes/dead_import_elimination.rs`, `crates/luanext-core/src/optimizer/passes/dead_export_elimination.rs`, `crates/luanext-core/src/optimizer/passes/unused_module_elimination.rs`, `crates/luanext-core/src/optimizer/passes/reexport_flattening.rs`

**Integration**: `crates/luanext-cli/src/main.rs` lines ~1710-1920

**Related specs**: [optimizer-architecture.md](optimizer-architecture.md), [optimizer-passes.md](optimizer-passes.md)

[NOTE: BEHAVIOR] LTO runs during the codegen phase inside the parallel codegen closure, NOT during the optimizer phase. This is because LTO passes need `Vec<Statement<'arena>>` access via `MutableProgram`, and the optimizer runs earlier in the pipeline. The module graph is constructed at O2+ after type checking, then shared (read-only via `Arc`) across parallel codegen workers.

## Module Graph

### ModuleGraph

The top-level structure holding the entire program's dependency information.

```rust
pub struct ModuleGraph {
    pub modules: FxHashMap<PathBuf, ModuleNode>,
    pub entry_points: FxHashSet<PathBuf>,
}
```

### ModuleNode

Per-module metadata: exports, imports, re-exports, and reachability status.

```rust
pub struct ModuleNode {
    pub path: PathBuf,
    pub exports: FxHashMap<String, ExportInfo>,
    pub imports: FxHashMap<String, ImportInfo>,
    pub re_exports: Vec<ReExportInfo>,
    pub is_reachable: bool,
}
```

### ExportInfo

```rust
pub struct ExportInfo {
    pub name: String,
    pub is_type_only: bool,
    pub is_default: bool,
    pub is_used: bool,       // true if imported by any other module
}
```

### ImportInfo

```rust
pub struct ImportInfo {
    pub name: String,
    pub source_module: PathBuf,
    pub source_symbol: String,
    pub is_type_only: bool,
    pub is_referenced: bool, // true if symbol is actually used in code
}
```

### ReExportInfo

```rust
pub struct ReExportInfo {
    pub source_module: PathBuf,
    pub specifiers: ReExportKind,
}

pub enum ReExportKind {
    All,                            // export * from './foo'
    Named(Vec<(String, String)>),   // export { x as y } from './foo'
}
```

## Module Graph Construction

`ModuleGraph::build()` takes all modules (path + statements), a `StringInterner`, a `ModuleRegistry`, and entry points. Construction proceeds in 5 phases:

### Phase 1: Extract Export/Import Info

Scans each module's AST for `Statement::Export` and `Statement::Import`, building `ExportInfo`, `ImportInfo`, and `ReExportInfo` records.

**Export extraction handles**:
- `ExportKind::Declaration` -- function, variable, or class declarations
- `ExportKind::Named` -- with or without source (re-export vs local)
- `ExportKind::Default` -- default export (keyed as `"default"`)
- `ExportKind::All` -- star re-export

**Import extraction handles**:
- `ImportClause::Default` -- `import X from './foo'`
- `ImportClause::Named` -- `import { X, Y as Z } from './foo'`
- `ImportClause::Namespace` -- `import * as Mod from './foo'`
- `ImportClause::Mixed` -- `import Default, { Named } from './foo'`
- `ImportClause::TypeOnly` -- skipped (type-only imports don't create runtime dependencies)

### Phase 2: Mark Import References

Traverses each module's statements and expressions to find which imported symbols are actually referenced in code. Sets `ImportInfo.is_referenced = true` for used imports.

### Phase 3: Resolve Source Paths

Resolves relative import paths (e.g., `./b`, `../utils`) to canonical module paths. Tries the path with common extensions (`.luax`, `.d.luax`, `.lua`) and the directory index pattern (`index.luax`).

### Phase 4: Compute Reachability

DFS from entry points marks all reachable modules via `is_reachable`. Follows both import edges and re-export edges. Type-only imports are NOT followed (they create no runtime dependency).

### Phase 5: Mark Usage

Builds a reverse map of which exports are imported by other modules, setting `ExportInfo.is_used = true` for exports that have at least one importer.

## Re-Export Chain Resolution

`ModuleGraph::resolve_re_export_chain()` flattens re-export chains to find the original export source. Handles both `All` and `Named` re-export kinds with cycle detection (visited set) and depth limiting (max 10 levels).

```lua
// A re-exports from B, B re-exports from C:
// resolve_re_export_chain(A, "foo") -> Some((C, "foo"))
```

## LTO Passes

### Pass Execution Order

LTO passes execute in the parallel codegen closure in `main.rs`, per-module:

1. **UnusedModuleEliminationPass** (Phase 1.7, before codegen) -- filters modules at O2+
2. **ReExportFlatteningPass** (O3 only) -- flattens re-export chains
3. **DeadImportEliminationPass** (O2+) -- removes unused import bindings
4. **DeadExportEliminationPass** (O2+) -- removes unused export wrappers

### UnusedModuleEliminationPass

**File**: `passes/unused_module_elimination.rs` | **Level**: O2+

Filters entire modules not reachable from any entry point. This is not an AST transformation -- it provides a `should_compile()` predicate used at the CLI level to skip compilation entirely.

```rust
impl UnusedModuleEliminationPass {
    pub fn should_compile(&self, module_path: &Path) -> bool;
    pub fn get_modules_to_compile(&self) -> Vec<PathBuf>;
    pub fn get_stats(&self) -> UnusedModuleStats;
}
```

Safety: entry points are always compiled. Unknown modules (not in graph) default to compiling.

[NOTE: BEHAVIOR] This pass runs BEFORE the parallel codegen closure. It filters `checked_modules_filtered` using `retain()` based on the compile set from `get_modules_to_compile()`. Eliminated modules are never passed to codegen at all.

### ReExportFlatteningPass

**File**: `passes/reexport_flattening.rs` | **Level**: O3 only

Flattens re-export chains to eliminate indirection. When module A has `export { foo } from './b'` and B has `export { foo } from './c'`, rewrites A's statement to require directly from C.

Uses `ModuleGraph::resolve_re_export_chain()` to find the original source, then `compute_relative_require_path()` to generate the correct relative path for the rewritten import.

[NOTE: BEHAVIOR] Only enabled at O3 because it can change module evaluation order. If intermediate module B has side effects in its body, they may execute in a different order after flattening.

### DeadImportEliminationPass

**File**: `passes/dead_import_elimination.rs` | **Level**: O2+

Removes import statements for symbols that are never referenced in the module's code. Uses `ImportInfo.is_referenced` from the module graph. More aggressive than tree-shaking: removes the import binding entirely.

```lua
// Before (if unusedFunc is never referenced):
import { usedFunc, unusedFunc } from './utils';

// After:
import { usedFunc } from './utils';
```

### DeadExportEliminationPass

**File**: `passes/dead_export_elimination.rs` | **Level**: O2+

Removes export statements for symbols not imported by any other module. Conservative: only removes the export wrapper, not the underlying declaration (which may still be used locally).

```lua
// Before (if unusedFunc is never imported):
export function unusedFunc() { return 42; }

// After:
function unusedFunc() { return 42; }  // export removed, definition kept
```

## Type-Only Handling

Type-only exports and imports are tracked separately throughout the LTO system:

- `ExportInfo.is_type_only` -- marks exports that only provide type information
- `ImportInfo.is_type_only` -- marks imports that only consume type information
- `ImportClause::TypeOnly` -- skipped entirely during import extraction (no runtime dependency)
- Reachability analysis does NOT follow type-only import edges
- Type-only exports/imports are erased at codegen regardless of LTO

## Integration Point

The module graph is constructed in `main.rs` after type checking (Phase 1.5) at O2+:

```rust
let graph = ModuleGraph::build(&module_data, interner, &registry, &entry_points);
```

The resulting graph is wrapped in `Arc<ModuleGraph>` and shared across:
1. **Phase 1.7**: `UnusedModuleEliminationPass` filters unreachable modules
2. **Phase 2**: Parallel codegen closure applies `ReExportFlatteningPass` (O3), `DeadImportEliminationPass` (O2+), and `DeadExportEliminationPass` (O2+) per-module

Each LTO pass receives a clone of the `Arc<ModuleGraph>` and the module's `Arc<StringInterner>`, then calls `set_current_module()` with the module's file path before applying.

## Helper Functions

The module graph module provides utility functions for path resolution:

- `resolve_relative_source(from_dir, source, known_modules)` -- resolves relative import strings to canonical module paths
- `compute_relative_require_path(from, to)` -- computes a relative require path between two module files
- `normalize_path(path)` -- resolves `.` and `..` components without filesystem access
- `strip_module_extension(path)` -- removes `.luax`, `.d.luax`, `.lua` extensions
