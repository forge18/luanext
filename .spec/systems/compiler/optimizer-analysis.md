# Optimizer Analysis Infrastructure

CFG, dominance trees, SSA form, alias analysis, side-effect analysis, and module graph.

## Overview

The analysis infrastructure provides program analysis data structures consumed by optimization passes. Analyses are computed once before the optimizer's fixed-point loop and do not modify the AST. All data structures use `usize` statement indices and `StringId` variable names rather than `&'arena` AST references, keeping them decoupled from arena lifetimes.

**Files**: `crates/luanext-core/src/optimizer/analysis/`

**Related specs**: [optimizer-architecture.md](optimizer-architecture.md), [optimizer-lto.md](optimizer-lto.md)

## Analysis Dependency Chain

```text
CFG (needs only AST)
 +-> Dominance (needs CFG)
      +-> SSA (needs CFG + Dominance)

Alias Analysis (needs only AST, independent)
Side-Effect Analysis (needs only AST, independent)
Module Graph (needs AST + ModuleRegistry, independent)
```

## AnalysisContext

Program-wide aggregation of all per-function analyses plus program-wide side-effect information. Computed at O2+ before the optimizer's iteration loop.

```rust
pub struct AnalysisContext {
    function_analyses: FxHashMap<StringId, FunctionAnalysis>,
    side_effects: Option<SideEffectInfo>,
    top_level_key: Option<StringId>,
}
```

### FunctionAnalysis

Per-function analysis bundle containing CFG, dominance tree, SSA form, and alias info.

```rust
pub struct FunctionAnalysis {
    pub cfg: ControlFlowGraph,
    pub dominators: DominatorTree,
    pub ssa: SsaForm,
    pub alias_info: AliasInfo,
}
```

### Computation

`AnalysisContext::compute()` iterates through all statements:

1. Analyzes the top-level scope (keyed by sentinel `"<top-level>"` in the interner)
2. Analyzes each `Statement::Function` body individually
3. Runs program-wide side-effect analysis

For each function/scope: builds CFG, computes dominators, constructs SSA form, runs alias analysis.

---

## Control Flow Graph (CFG)

**File**: `analysis/cfg.rs`

Builds basic blocks and edges from a function body or top-level statement list.

### BasicBlock

```rust
pub struct BasicBlock {
    pub id: BlockId,
    pub statement_indices: Vec<usize>,   // indices into source statement list
    pub span: Span,
    pub terminator: Terminator,
}
```

### BlockId

Thin wrapper over `u32`. Two sentinel values:
- `BlockId::ENTRY` (0) -- control flow begins here
- `BlockId::EXIT` (1) -- control flow ends here

### Terminator

How control leaves a basic block:

| Variant | Description |
|---------|-------------|
| `Goto(BlockId)` | Unconditional jump |
| `Branch { condition_stmt_index, true_target, false_target }` | Conditional branch |
| `Return` | Function return |
| `Unreachable` | Dead code after return/break/continue |
| `LoopBack(BlockId)` | Loop back-edge |
| `FallThrough` | Implicit exit (no explicit return) |
| `TryCatch { normal, catch_targets }` | Exception handling |

### CfgBuilder

`CfgBuilder::build(statements)` constructs the CFG by walking statements and creating basic blocks at branch points (if/while/for/try-catch). Returns a `ControlFlowGraph` with blocks, predecessor/successor maps, and reverse postorder traversal.

---

## Dominance Tree

**File**: `analysis/dominance.rs`

Computes the dominator tree from a CFG using the Cooper-Harvey-Kennedy iterative algorithm.

### DominatorTree

```rust
pub struct DominatorTree {
    pub idom: FxHashMap<BlockId, BlockId>,         // immediate dominator
    pub children: FxHashMap<BlockId, Vec<BlockId>>, // dominator tree children
    pub frontiers: FxHashMap<BlockId, Vec<BlockId>>, // dominance frontiers
}
```

**Key concepts**:
- Block A **dominates** block B if every path from entry to B passes through A
- **Immediate dominator** (idom): closest strict dominator
- **Dominance frontier** of A: blocks where A dominates a predecessor but not the block itself -- used for SSA phi-function placement

---

## SSA Form

**File**: `analysis/ssa.rs`

Static Single Assignment form using the Cytron et al. algorithm:
1. Collect variable definitions per block
2. Place phi-functions using dominance frontiers (iterative worklist)
3. Rename variables via dominator tree preorder walk

### SsaVar

```rust
pub struct SsaVar {
    pub name: StringId,   // original variable name
    pub version: u32,     // SSA version (0 = undefined, 1+ = definitions)
}
```

### PhiFunction

```rust
pub struct PhiFunction {
    pub target: SsaVar,                       // variable this phi defines
    pub operands: Vec<(BlockId, SsaVar)>,     // (predecessor_block, incoming_version)
}
```

### SsaForm

```rust
pub struct SsaForm {
    pub phi_functions: FxHashMap<BlockId, Vec<PhiFunction>>,
    pub definitions: FxHashMap<usize, Vec<SsaVar>>,   // stmt_index -> defined vars
    pub uses: FxHashMap<usize, Vec<SsaVar>>,           // stmt_index -> used vars
}
```

[NOTE: DESIGN] SSA form is a parallel data structure -- it does NOT modify the AST. Optimization passes query this mapping to understand variable versioning and data flow. This avoids the complexity of maintaining SSA invariants during transformations.

---

## Alias Analysis

**File**: `analysis/alias.rs`

Flow-insensitive, intraprocedural alias analysis using union-find for alias class tracking.

### MemoryLocation

```rust
pub enum MemoryLocation {
    Local(StringId),                    // local variable
    Global(StringId),                   // global variable
    TableField(StringId, StringId),     // table.field
    TableDynamic(StringId),             // table[expr] (dynamic index)
    Upvalue(StringId),                  // captured by closure
}
```

### AliasResult

```rust
pub enum AliasResult {
    NoAlias,    // definitely distinct memory locations
    MayAlias,   // conservative: might be the same
    MustAlias,  // definitely the same memory location
}
```

### AliasInfo

```rust
pub struct AliasInfo {
    pub alias_classes: Vec<FxHashSet<MemoryLocation>>,           // alias class sets
    pub location_to_class: FxHashMap<MemoryLocation, usize>,     // location -> class index
    pub escaped: FxHashSet<StringId>,                             // escaped variables
}
```

**Lua-specific simplifications**:
- Primitive types (number, string, boolean, nil) are value types and never alias
- Only tables and closures can alias (no pointer arithmetic)
- Global variables conservatively may alias anything
- Variables that escape scope (passed to unknown functions, stored in tables, returned) are tracked

---

## Side-Effect Analysis

**File**: `analysis/side_effect.rs`

Tracks which functions have observable side effects, enabling dead call elimination, function cloning, and interprocedural optimizations.

### SideEffects

```rust
pub struct SideEffects {
    pub global_reads: FxHashSet<StringId>,
    pub global_writes: FxHashSet<StringId>,
    pub table_mutations: FxHashMap<StringId, Option<FxHashSet<StringId>>>,
    pub has_io: bool,
    pub calls_unknown: bool,
    pub may_throw: bool,
    pub accesses_environment: bool,
}
```

Key predicates:
- `is_pure()` -- no observable side effects at all
- `is_read_only()` -- reads globals but never writes/mutates/does I/O

### SideEffectInfo

Program-wide side-effect analysis results. Maps function names to their `SideEffects`. Built by `SideEffectAnalyzer::analyze()` which walks all statements and expressions looking for:
- Global variable reads and writes
- Table mutations
- I/O operations (`print`, `io.*`, file operations)
- Calls to unknown/unanalyzable functions
- Exception throwing (`error()`, `throw`)
- Environment access (`_ENV`, `getfenv`, `setfenv`)

---

## Module Graph

**File**: `analysis/module_graph.rs`

Whole-program module dependency graph with export/import tracking and reachability analysis. Used by LTO passes.

See [optimizer-lto.md](optimizer-lto.md) for full module graph documentation.

---

## WholeProgramAnalysis

**File**: `whole_program_analysis.rs`

Thread-safe cross-module analysis results, built once sequentially after type checking and shared (read-only) across parallel optimization passes via `Arc`.

```rust
pub struct WholeProgramAnalysis {
    pub class_hierarchy: Arc<ClassHierarchy>,
    pub side_effects: Option<Arc<SideEffectInfo>>,
}
```

### ClassHierarchy

**File**: `devirtualization.rs`

Class hierarchy information for devirtualization safety analysis:

```rust
pub struct ClassHierarchy {
    parent_of: FxHashMap<StringId, Option<StringId>>,           // class -> parent
    children_of: FxHashMap<StringId, Vec<StringId>>,             // parent -> children
    is_final: FxHashMap<StringId, bool>,                         // final classes
    final_methods: FxHashMap<(StringId, StringId), bool>,        // final methods
    declares_method: FxHashMap<(StringId, StringId), bool>,      // method declared here
    known_classes: FxHashMap<StringId, bool>,                     // class vs interface
    instantiated_subclasses: FxHashMap<StringId, FxHashSet<StringId>>,  // RTA
    single_instantiated_subclass: FxHashMap<StringId, StringId>,        // RTA single target
    instantiation_counts: FxHashMap<StringId, usize>,                   // RTA counts
    classes_with_instantiations: FxHashSet<StringId>,                   // RTA instantiated set
}
```

Built via `ClassHierarchy::build_multi_module()` which scans all programs for class declarations. Only built when O3+ optimization is enabled.

[NOTE: BEHAVIOR] `WholeProgramAnalysis` is built sequentially before parallel codegen. It is cloned cheaply (via `Arc`) for each parallel worker. The `class_hierarchy` is pushed into the `DevirtualizationPass` via `Optimizer::set_whole_program_analysis()` using `downcast_mut`.
