# Type Compatibility

Assignability rules, structural typing, and cycle detection for the LuaNext type system.

## Overview

Type compatibility determines whether a value of type `Source` can be used where type `Target` is expected. LuaNext uses **structural typing** — types are compatible based on their shape, not their declared name.

**File**: `crates/luanext-typechecker/src/core/type_compat.rs`

## Entry Points

```rust
struct TypeCompatibility;

impl TypeCompatibility {
    // Basic assignability check
    fn is_assignable(source: &Type, target: &Type) -> bool;

    // With cache for repeated checks
    fn is_assignable_with_cache(source: &Type, target: &Type, cache: &mut TypeRelationCache) -> bool;

    // With type environment for alias resolution
    fn is_assignable_with_env(source: &Type, target: &Type, env: &TypeEnvironment, interner: &StringInterner) -> bool;
}
```

The `is_assignable_with_env` variant is the most complete — it resolves type aliases through the `TypeEnvironment` and uses the `StringInterner` for name comparison.

## Cycle Detection

Recursive assignability checks use a `HashSet<(usize, usize)>` of pointer pairs to detect cycles. When a `(source_ptr, target_ptr)` pair is already in the visited set, the check returns `true` (assumes compatible to break the cycle).

`[NOTE: BEHAVIOR]` The cycle-detection-returns-true assumption is correct for recursive types (e.g., linked lists) but was the source of a bug where `is_assignable_with_env_recursive()` inserted pairs before checking literal/primitive compatibility. Literal types (e.g., `Literal(Number(123))`) were incorrectly considered assignable to incompatible primitives (e.g., `Primitive(String)`) because the delegation to `is_assignable_recursive()` found the pair already visited. This was fixed by duplicating literal/primitive checks before delegation.

## Top and Bottom Types

| Rule | Behavior |
| ---- | -------- |
| `Unknown` is assignable to/from anything | Top type — universal compatibility |
| `Never` is assignable to anything | Bottom type — subtype of all types |
| Nothing is assignable to `Never` | No value inhabits `Never` |

## Primitive Assignability

Checked via `is_primitive_assignable(source, target)`:

| Source | Target | Result |
| ------ | ------ | ------ |
| Same primitive | Same primitive | `true` |
| `Integer` | `Number` | `true` |
| `Number` | `Integer` | `false` |
| `Void` | `Nil` | `true` |
| Other mismatches | — | `false` |

`[NOTE: BEHAVIOR]` `Integer` widens to `Number` but not the reverse. This matches Lua 5.3+ semantics where integers are a subtype of numbers.

## Literal Assignability

Checked via `is_literal_assignable_to_primitive(literal, primitive)`:

| Literal | Primitive | Result |
| ------- | --------- | ------ |
| `Literal::Number(_)` | `Number` | `true` |
| `Literal::Integer(_)` | `Number` | `true` |
| `Literal::Integer(_)` | `Integer` | `true` |
| `Literal::String(_)` | `String` | `true` |
| `Literal::Boolean(_)` | `Boolean` | `true` |
| `Literal::Nil` | `Nil` | `true` |

Literal-to-literal: only if values are equal (`s_lit == t_lit`).

## Type Reference Resolution

When comparing type references:

1. **Same name**: Check type arguments are pairwise compatible
2. **Different names**: Resolve both through `TypeEnvironment.lookup_type_alias()`
3. **One reference, one concrete**: Resolve the reference, compare structurally
4. **Neither resolvable**: Incompatible

## Union Assignability

- **Value → Union**: Compatible if assignable to **any** union member
- **Union → Value**: Compatible if **all** union members are assignable to the target
- **Union → Union**: Each source member must be assignable to some target member

## Intersection Assignability

- A value is assignable to an intersection if it's assignable to **all** constituent types
- An intersection is assignable to a target if **any** constituent type is assignable

## Nullable Assignability

`Nullable(T)` is treated as `Union([T, Nil])`:

- `string` is assignable to `string?` (widening)
- `string?` is NOT assignable to `string` (narrowing requires type guard)
- `nil` is assignable to `T?` for any `T`

## Function Assignability

Functions are checked with **contravariant parameters** and **covariant return types**:

1. Target must have at most as many required parameters as source
2. Each target parameter type must be assignable to the corresponding source parameter type (contravariant)
3. Source return type must be assignable to target return type (covariant)

## Object Assignability

Structural compatibility — the source must have all required properties of the target:

1. For each non-optional property in target, source must have a compatible property
2. Source property type must be assignable to target property type
3. Extra properties in source are allowed (open/extensible objects)

## Array Assignability

`Array(S)` is assignable to `Array(T)` if `S` is assignable to `T`.

`[NOTE: BEHAVIOR]` Array types are covariant (not invariant). This matches TypeScript behavior but can theoretically lead to unsound mutations.

## Tuple Assignability

`Tuple(S1, S2, ...)` is assignable to `Tuple(T1, T2, ...)` if:

1. Same length
2. Each `Si` is assignable to `Ti`

## Caching

`TypeRelationCache` caches assignability results for repeated checks:

```rust
struct TypeRelationCache {
    // Maps (source_ptr, target_ptr) -> is_assignable result
}
```

Used during type checking to avoid re-computing expensive structural comparisons.

## Cross-References

- [Type Primitives](type-primitives.md) — primitive, literal, and composite type definitions
- [Type Advanced](type-advanced.md) — generic and conditional type compatibility
- [Type Checking](type-checking.md) — how compatibility checks are invoked during type checking
