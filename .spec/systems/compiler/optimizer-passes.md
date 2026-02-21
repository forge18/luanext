# Optimizer Passes

Complete catalog of all optimization passes, organized by optimization level.

## Overview

The LuaNext optimizer includes 26+ optimization passes registered via `register_passes()`. Passes are grouped into composite traversals (expression, elimination, function, data-structure) and standalone whole-program passes. Additionally, 4 LTO passes run during codegen rather than during the optimizer phase.

**Files**: `crates/luanext-core/src/optimizer/passes/`

**Related specs**: [optimizer-architecture.md](optimizer-architecture.md), [optimizer-analysis.md](optimizer-analysis.md), [optimizer-lto.md](optimizer-lto.md)

---

## O1 Passes (Minimal)

### constant_folding

**File**: `passes/constant_folding.rs` | **Visitor**: `ExprVisitor` | **Composite**: `expr_pass`

Evaluates compile-time constant expressions and replaces them with their results.

- Arithmetic: `1 + 2` becomes `3`, `10 / 2` becomes `5`
- String concatenation: `"a" .. "b"` becomes `"ab"`
- Comparison: `1 < 2` becomes `true`
- Unary: `-(-5)` becomes `5`, `not true` becomes `false`
- Boolean logic: `true and false` becomes `false`

### dead_code_elimination

**File**: `passes/dead_code_elimination.rs` | **Visitor**: `BlockVisitor` | **Composite**: `elim_pass`

Removes unreachable code and unused declarations.

- Truncates statements after `return`, `break`, or `continue`
- Removes `if false then ... end` blocks
- Simplifies `if true then ... end` to just the body
- Removes unused local variable declarations (when the initializer has no side effects)

### algebraic_simplification

**File**: `passes/algebraic_simplification.rs` | **Visitor**: `ExprVisitor` | **Composite**: `expr_pass`

Applies algebraic identities to simplify expressions.

- Additive identity: `x + 0` becomes `x`, `x - 0` becomes `x`
- Multiplicative identity: `x * 1` becomes `x`, `x / 1` becomes `x`
- Multiplicative zero: `x * 0` becomes `0`
- Double negation: `not not x` becomes `x`, `-(-x)` becomes `x`
- Self-operations: `x - x` becomes `0`, `x / x` becomes `1`

### copy_propagation

**File**: `passes/copy_propagation.rs` | **Visitor**: `BlockVisitor` | **Composite**: `elim_pass` (at O2+)

Replaces uses of a copy variable with the original. When `local a = b` and `b` is not reassigned between the copy and its uses, replaces all uses of `a` with `b`.

[NOTE: REGISTRATION] Although this is conceptually an O1 pass, it is only registered at O2+ as part of the elimination composite.

### peephole_optimization

**File**: `passes/peephole_optimization.rs` | **Visitor**: `ExprVisitor` | **Composite**: `expr_pass` (at O2+)

Local pattern-matching optimizations on small AST fragments.

- Removes redundant parenthesization
- Simplifies `x == true` to `x` and `x == false` to `not x`
- Optimizes double type assertions
- Simplifies redundant nil checks

[NOTE: REGISTRATION] Added to `expr_pass` at O2+, not O1.

---

## O2 Passes (Moderate)

### function_inlining

**File**: `passes/function_inlining.rs` | **Visitor**: `PreAnalysisPass` + `StmtVisitor` | **Composite**: `func_pass`

Inlines small function bodies at call sites. Uses conservative heuristics:

- Function body must be small (limited statement count)
- Function must not be recursive
- Pre-analysis phase builds a call graph to determine inlining candidates
- Parameter substitution replaces formal params with actual arguments

### loop_optimization

**File**: `passes/loop_optimization.rs` | **Visitor**: `WholeProgramPass` (standalone)

Loop invariant code motion and strength reduction.

- **LICM**: Moves loop-invariant computations outside the loop body
- **Strength reduction**: Replaces expensive operations with cheaper equivalents (e.g., `i * 4` in a loop increment becomes `i + 4` accumulation)

Requires `AstFeatures::HAS_LOOPS`.

### string_concat_optimization

**File**: `passes/string_concat_optimization.rs` | **Visitor**: `ExprVisitor` | **Composite**: `data_pass`

Optimizes chained string concatenation (`..`) operations.

- Folds adjacent literal string operands: `"a" .. "b" .. "c"` becomes `"abc"`
- Flattens nested concatenation trees for better runtime performance
- Uses `table.concat` for large concatenation chains

### dead_store_elimination

**File**: `passes/dead_store_elimination.rs` | **Visitor**: `BlockVisitor` | **Composite**: `elim_pass`

Removes assignments to variables that are never subsequently read. Uses reverse liveness analysis across statements in a block to determine which stores are dead.

### tail_call_optimization

**File**: `passes/tail_call_optimization.rs` | **Visitor**: `StmtVisitor` | **Composite**: `func_pass`

Converts tail-position function calls to proper tail calls. Ensures `return f(args)` is emitted without intermediate local variables, enabling Lua's native tail call elimination.

### jump_threading

**File**: `passes/jump_threading.rs` | **Visitor**: `BlockVisitor` | **Composite**: `elim_pass`

Simplifies control flow by eliminating redundant branches.

- `if true then A else B end` becomes `A`
- `if false then A else B end` becomes `B`
- Chains of trivial conditional branches are collapsed

### common_subexpression_elimination

**File**: `passes/common_subexpression_elimination.rs` | **Visitor**: `BlockVisitor` | **Composite**: `elim_pass`

Identifies identical expressions computed multiple times and replaces subsequent computations with the previously computed result.

- Uses value numbering within basic blocks
- Only eliminates expressions that are provably side-effect-free

### sccp (Sparse Conditional Constant Propagation)

**File**: `passes/sccp.rs` | **Visitor**: `BlockVisitor` | **Composite**: `elim_pass`

Combined constant propagation and reachability analysis using a lattice-based approach.

- Propagates constant values through assignments and control flow
- Determines unreachable branches based on constant conditions
- More powerful than simple constant folding because it considers control flow

### global_localization

**File**: `passes/global_localization.rs` | **Visitor**: `WholeProgramPass` (standalone)

Caches frequently-used Lua globals in local variables for faster access.

```lua
-- Before:
print("a"); print("b"); print("c")

-- After:
local __print = print
__print("a"); __print("b"); __print("c")
```

Analyzes global usage frequency and only localizes globals used more than a threshold number of times.

[NOTE: BEHAVIOR] Registered unconditionally but has `min_level = Minimal`, so it runs at O1 and above. It is the last pass registered in `register_passes()`.

### table_preallocation

**File**: `passes/table_preallocation.rs` | **Visitor**: `ExprVisitor` | **Composite**: `data_pass`

Adds size hints to table constructors so the Lua runtime can pre-allocate the correct hash and array portions.

- Counts array and hash elements in table/object/array literals
- Emits constructor with pre-allocation hints for the target Lua version

---

## O3 Passes (Aggressive)

### generic_specialization

**File**: `passes/generic_specialization.rs` | **Visitor**: `WholeProgramPass` (standalone)

Creates specialized versions of generic functions for known concrete type arguments. When a generic function is called with specific types, a type-specialized copy is generated, eliminating runtime type dispatch.

- Uses type argument hashing to cache and deduplicate specializations
- Generates unique names for specialized variants (e.g., `foo__spec_1`)
- Uses `build_substitutions()` and `instantiate_function_declaration()` for type substitution

### loop_unrolling

**File**: `passes/loop_unrolling.rs` | **Visitor**: `WholeProgramPass` (standalone)

Unrolls small numeric for-loops with constant bounds.

- Only numeric for-loops (not generic for-in loops)
- Bounds (start, end, step) must be compile-time constants
- Maximum trip count: 4 iterations (to avoid code bloat)
- No break/continue/return allowed in the loop body
- Substitutes loop variable with the concrete iteration value in each unrolled copy

Requires `AstFeatures::HAS_LOOPS`.

### function_cloning

**File**: `passes/function_cloning.rs` | **Visitor**: `WholeProgramPass` (standalone)

Clones small functions at call sites when called with constant arguments, enabling further optimization of the cloned copy.

- Functions must have 8 or fewer statements
- Maximum 4 clones per original function
- Deduplicates identical specializations (same constant arguments produce one clone)
- Cloned functions are renamed with a `__clone_N` suffix

### interprocedural_const_prop

**File**: `passes/interprocedural_const_prop.rs` | **Visitor**: `WholeProgramPass` (standalone)

When ALL callers of a function pass the same constant value for a parameter, substitutes that constant directly into the function body and removes the parameter from the signature.

- Fixed-point iteration with maximum 3 rounds
- Requires all call sites to agree on the constant value
- Modifies both the function declaration (removes param) and all call sites (removes argument)

### scalar_replacement

**File**: `passes/scalar_replacement.rs` | **Visitor**: `WholeProgramPass` (standalone)

Replaces local table/object variables with individual scalar local variables when the table does not escape its scope.

```lua
-- Before:
local point = { x = 1, y = 2 }
local dx = point.x + 10

-- After:
local point__x = 1
local point__y = 2
local dx = point__x + 10
```

Safety constraints:
- Only `const`/`let` variables with object literal initializers
- All accesses must be static member reads/writes (`.field`)
- Table must not escape (not passed to functions, not returned, not assigned elsewhere)
- Maximum 8 fields per table

Requires `AstFeatures::HAS_OBJECTS`.

---

## O2+ Whole-Program Passes (standalone)

### rich_enum_optimization

**File**: `rich_enum_optimization.rs` | **Visitor**: `WholeProgramPass` (standalone)

Optimizes rich enum (enum with methods/fields) implementations by analyzing enum structure and inlining simple enum method calls.

- Collects enum fields and simple methods during analysis
- Inlines trivial enum methods at call sites

Requires `AstFeatures::HAS_ENUMS`.

---

## O3 Whole-Program Passes (standalone)

### devirtualization

**File**: `devirtualization.rs` | **Visitor**: `WholeProgramPass` (standalone)

Replaces virtual method calls with direct calls when the concrete class type can be determined. Uses `ClassHierarchy` from `WholeProgramAnalysis`.

- Final classes: all method calls can be devirtualized
- Final methods: calls to specific methods can be devirtualized
- Rapid Type Analysis (RTA): tracks instantiated subclasses to narrow candidates
- Single-target optimization: if only one subclass is instantiated, devirtualize

Requires `AstFeatures::HAS_CLASSES`.

### aggressive_inlining

**File**: `aggressive_inlining.rs` | **Visitor**: `StmtVisitor` | **Composite**: `func_pass` (at O3)

More aggressive function inlining using call graph information from whole-program analysis. Relaxes the conservative heuristics of the standard `function_inlining` pass.

### interface_inlining

**File**: `interface_inlining.rs` | **Visitor**: `StmtVisitor` | **Composite**: `func_pass` (at O3)

Inlines interface method calls when the concrete implementation can be determined. Works with `ClassHierarchy` to resolve interface method dispatch.

Requires `AstFeatures::HAS_INTERFACES`.

### operator_inlining

**File**: `operator_inlining.rs` | **Visitor**: `ExprVisitor` | **Composite**: `expr_pass` (at O3)

Inlines operator overload implementations. When a class defines `__add`, `__sub`, etc., and the operand types are known, replaces the operator expression with the inlined method body.

### method_to_function_conversion

**File**: `method_to_function_conversion.rs` | **Visitor**: `StmtVisitor` | **Composite**: `func_pass` (at O2+)

Converts method calls to plain function calls when possible. If a method does not reference `self`, it can be called as a regular function, avoiding the overhead of method dispatch and the implicit `self` parameter.

---

## LTO Passes

These passes are NOT registered in the optimizer's `register_passes()`. They run during the codegen phase.

### dead_import_elimination

**File**: `passes/dead_import_elimination.rs`

Removes import statements for symbols that are never referenced in the module's code. Uses `ModuleGraph.ImportInfo.is_referenced` to determine usage.

### dead_export_elimination

**File**: `passes/dead_export_elimination.rs`

Removes export statements for symbols not imported by any other module. Conservative: only removes the export wrapper, not the underlying declaration (which may be used locally).

### unused_module_elimination

**File**: `passes/unused_module_elimination.rs`

Filters out entire modules that are not reachable from any entry point. Not a traditional AST pass -- it provides a `should_compile()` predicate used at the CLI level to skip compilation.

### reexport_flattening

**File**: `passes/reexport_flattening.rs`

Flattens re-export chains to eliminate indirection. When A re-exports from B and B re-exports from C, rewrites A to require directly from C.

[NOTE: BEHAVIOR] Only enabled at O3 because it can change module evaluation order, which matters if intermediate modules have side effects.

See [optimizer-lto.md](optimizer-lto.md) for full LTO documentation.
