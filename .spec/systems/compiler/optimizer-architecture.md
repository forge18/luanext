# Optimizer Architecture

Pass management, visitor traits, composite passes, fixed-point iteration, and optimization levels.

## Overview

The optimizer transforms a `MutableProgram<'arena>` AST through a series of optimization passes organized into composite groups. Passes use a clone-and-rebuild pattern for sub-expression mutation: inner `&'arena` references are cloned to owned values, mutated, then allocated back into the arena.

**Files**: `crates/luanext-core/src/optimizer/`

**Related specs**: [optimizer-passes.md](optimizer-passes.md), [optimizer-analysis.md](optimizer-analysis.md), [optimizer-lto.md](optimizer-lto.md)

## MutableProgram

The optimizer operates on `MutableProgram<'arena>` rather than the immutable `Program<'arena>`. The key difference is a mutable `Vec<Statement<'arena>>` at the top level, enabling in-place statement insertion, removal, and reordering.

```rust
pub struct MutableProgram<'arena> {
    pub statements: Vec<Statement<'arena>>,
    // ...
}
```

Sub-expressions within statements still use immutable `&'arena` references. Passes that transform sub-expressions clone them to owned values, mutate, then allocate back into the arena via `arena.alloc()`.

## Visitor Trait Hierarchy

Three visitor traits define how passes interact with the AST, plus two traits for special pass types.

### ExprVisitor

Operates on individual expressions. Used by constant folding, algebraic simplification, peephole optimization, table preallocation, string concat optimization, and operator inlining.

```rust
pub trait ExprVisitor<'arena> {
    fn visit_expr(&mut self, expr: &mut Expression<'arena>, arena: &'arena Bump) -> bool;
    fn required_features(&self) -> AstFeatures { AstFeatures::EMPTY }
}
```

Returns `true` if the expression was modified.

### StmtVisitor

Operates on individual statements. Used by function inlining, tail call optimization, method-to-function conversion, aggressive inlining, and interface method inlining.

```rust
pub trait StmtVisitor<'arena> {
    fn visit_stmt(&mut self, stmt: &mut Statement<'arena>, arena: &'arena Bump) -> bool;
    fn required_features(&self) -> AstFeatures { AstFeatures::EMPTY }
}
```

### BlockVisitor

Operates on `Vec<Statement>` to access sibling statements. Used by dead code elimination (truncate after return/break), dead store elimination (reverse liveness), copy propagation, CSE, SCCP, and jump threading.

```rust
pub trait BlockVisitor<'arena> {
    fn visit_block_stmts(
        &mut self,
        stmts: &mut Vec<Statement<'arena>>,
        arena: &'arena Bump,
    ) -> bool;
    fn required_features(&self) -> AstFeatures { AstFeatures::EMPTY }
}
```

### PreAnalysisPass

Runs an analysis phase before transformation. Used by function inlining (to build call graph before inlining decisions).

```rust
pub trait PreAnalysisPass<'arena> {
    fn analyze(&mut self, program: &MutableProgram<'arena>);
    fn required_features(&self) -> AstFeatures { AstFeatures::EMPTY }
}
```

### WholeProgramPass

Standalone pass that operates on the entire `MutableProgram`. Used by loop optimization, rich enum optimization, devirtualization, generic specialization, loop unrolling, function cloning, interprocedural const prop, scalar replacement, and global localization.

```rust
pub trait WholeProgramPass<'arena> {
    fn name(&self) -> &'static str;
    fn min_level(&self) -> OptimizationLevel { OptimizationLevel::Minimal }
    fn required_features(&self) -> AstFeatures { AstFeatures::EMPTY }
    fn run(&mut self, program: &mut MutableProgram<'arena>, arena: &'arena Bump) -> Result<bool, String>;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
```

## Composite Passes

Composite passes merge multiple visitors into single AST traversals to reduce traversal overhead. There are four composite groups in the optimizer.

### ExpressionCompositePass

Merges multiple `ExprVisitor` implementations into a single expression-tree traversal. Visits children recursively via `visit_expr_children()`, then applies all registered visitors.

**Used for**: expression transforms (constant folding + algebraic simplification, peephole at O2+, operator inlining at O3) and data-structure transforms (table preallocation + string concat optimization).

### StatementCompositePass

Merges `StmtVisitor` and `BlockVisitor` implementations. Per-statement visitors run first, then block-level visitors operate on the full `Vec<Statement>`.

**Used for**: elimination transforms (dead code elimination + SCCP + jump threading + copy propagation + CSE + dead store elimination at O2+).

### AnalysisCompositePass

Combines `PreAnalysisPass` analyzers with `StmtVisitor` visitors. Runs all pre-analyzers first, then applies statement visitors.

**Used for**: function transforms (function inlining analysis + function inlining + tail call optimization + method-to-function conversion, aggressive inlining + interface method inlining at O3).

## Optimizer Struct

The `Optimizer<'arena>` struct orchestrates all passes.

```rust
pub struct Optimizer<'arena> {
    level: OptimizationLevel,
    interner: Arc<StringInterner>,
    expr_pass: Option<ExpressionCompositePass<'arena>>,      // expression transforms
    elim_pass: Option<StatementCompositePass<'arena>>,       // elimination transforms
    func_pass: Option<AnalysisCompositePass<'arena>>,        // function transforms
    data_pass: Option<ExpressionCompositePass<'arena>>,      // data-structure transforms
    standalone_passes: Vec<Box<dyn WholeProgramPass<'arena>>>, // whole-program passes
    whole_program_analysis: Option<WholeProgramAnalysis>,
    analysis_context: Option<analysis::AnalysisContext>,
    module_graph: Option<Arc<analysis::module_graph::ModuleGraph>>,
    current_module_path: Option<PathBuf>,
}
```

## AstFeatureDetector

Before the optimization loop begins, `AstFeatureDetector::detect()` scans the program once and returns a bitflag set of `AstFeatures`. Each pass declares its `required_features()`. A pass is skipped if its required features are not present in the program.

```rust
bitflags! {
    pub struct AstFeatures: u32 {
        const HAS_LOOPS      = 0b000000001;
        const HAS_CLASSES    = 0b000000010;
        const HAS_METHODS    = 0b000000100;
        const HAS_FUNCTIONS  = 0b000001000;
        const HAS_ARROWS     = 0b000010000;
        const HAS_INTERFACES = 0b000100000;
        const HAS_ARRAYS     = 0b001000000;
        const HAS_OBJECTS    = 0b010000000;
        const HAS_ENUMS      = 0b100000000;
        const EMPTY          = 0b000000000;
    }
}
```

Passes that return `AstFeatures::EMPTY` from `required_features()` always run regardless of detected features.

## Fixed-Point Iteration

The optimizer runs all passes in a loop until no pass reports a change, or the maximum of **10 iterations** is reached.

```lua
for iteration in 1..=10:
    changed = false
    changed |= expr_pass.run()      // O1+
    changed |= elim_pass.run()      // O1+
    changed |= func_pass.run()      // O2+
    changed |= data_pass.run()      // O2+
    for pass in standalone_passes:
        if pass.min_level() <= level:
            changed |= pass.run()
    if !changed:
        break
```

[NOTE: BEHAVIOR] At O2+, `AnalysisContext` (CFG, dominance, SSA, alias, side-effects) is computed once before the iteration loop, not recomputed per iteration. Passes that modify the AST may invalidate analysis results, but the analysis is not refreshed.

## Optimization Levels

| Level | Flag | Description |
|-------|------|-------------|
| O0 (None) | `--opt-level none` | No optimizations. Optimizer returns immediately. |
| O1 (Minimal) | `--opt-level minimal` | Safe transforms: constant folding, DCE, algebraic simplification, copy propagation, peephole |
| O2 (Moderate) | `--opt-level moderate` | All O1 + function inlining, loop optimization, string concat, dead store elimination, tail call, jump threading, CSE, SCCP, global localization, table preallocation |
| O3 (Aggressive) | `--opt-level aggressive` | All O2 + generic specialization, loop unrolling, function cloning, interprocedural const prop, scalar replacement, devirtualization, aggressive inlining, interface inlining, operator inlining |
| Auto | `--opt-level auto` | Minimal for debug builds, Moderate for release builds |

## register_passes() Function

The `register_passes()` method on `Optimizer` constructs the pass pipeline based on the optimization level:

**O1+ (expr_pass)**:
- `ConstantFoldingPass`
- `AlgebraicSimplificationPass`
- `PeepholeOptimizationPass` (added at O2+)

**O1+ (elim_pass)**:
- `DeadCodeEliminationPass` (block visitor)
- At O2+: `SccpPass`, `JumpThreadingPass`, `CopyPropagationPass`, `CommonSubexpressionEliminationPass`, `DeadStoreEliminationPass` (all block visitors)

**O2+ (data_pass)**:
- `TablePreallocationPass`
- `StringConcatOptimizationPass`

**O2+ (func_pass)**:
- `FunctionInliningPass` (both as pre-analyzer and visitor)
- `TailCallOptimizationPass`
- `MethodToFunctionConversionPass`

**O2+ (standalone)**:
- `LoopOptimizationPass`
- `RichEnumOptimizationPass`

**O3+ additions**:
- `OperatorInliningPass` (added to expr_pass)
- `AggressiveInliningPass` (added to func_pass)
- `InterfaceMethodInliningPass` (added to func_pass)
- `DevirtualizationPass` (standalone)
- `GenericSpecializationPass` (standalone)
- `LoopUnrollingPass` (standalone)
- `FunctionCloningPass` (standalone)
- `InterproceduralConstPropPass` (standalone)
- `ScalarReplacementPass` (standalone)

**All levels (standalone)**:
- `GlobalLocalizationPass` -- always registered, runs at all levels

[NOTE: BEHAVIOR] `GlobalLocalizationPass` is registered outside the optimization level guards. It is pushed to `standalone_passes` unconditionally after all level-gated passes.

[NOTE: BEHAVIOR] LTO passes (`DeadImportEliminationPass`, `DeadExportEliminationPass`, `UnusedModuleEliminationPass`, `ReExportFlatteningPass`) are NOT registered here. They are applied separately during the codegen phase. See [optimizer-lto.md](optimizer-lto.md).
