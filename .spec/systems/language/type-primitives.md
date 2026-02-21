# Type Primitives

Primitive types, literal types, composite types, and basic type constructors in the LuaNext type system.

## Overview

All types are represented as `Type<'arena>` nodes with a `TypeKind<'arena>` discriminant, arena-allocated via `bumpalo::Bump`. The type system is structural (not nominal), meaning types are compatible based on shape rather than name.

**File**: `crates/luanext-parser/src/ast/types.rs`

```rust
struct Type<'arena> {
    kind: TypeKind<'arena>,
    span: Span,
}
```

## Primitive Types

```rust
enum PrimitiveType {
    Nil,        // absence of value
    Boolean,    // true / false
    Number,     // floating-point number (f64)
    Integer,    // integer (i64)
    String,     // string value
    Unknown,    // top type — any value, requires narrowing to use
    Never,      // bottom type — no value satisfies this
    Void,       // function returns nothing
    Table,      // untyped Lua table
    Coroutine,  // Lua coroutine
    Thread,     // Lua thread
}
```

`[NOTE: BEHAVIOR]` `Unknown` is the top type (like TypeScript's `unknown`). Values typed as `Unknown` cannot be used without type narrowing. This differs from `any` in TypeScript — LuaNext does not have a true `any` type.

`[NOTE: BEHAVIOR]` `Integer` is distinct from `Number`. Integer literals (e.g., `42`) are inferred as `Integer`, float literals (e.g., `3.14`) as `Number`. `Integer` is assignable to `Number` but not vice versa.

`[NOTE: BEHAVIOR]` `Table` is the untyped Lua table. It accepts any key-value pairs but provides no type safety. Prefer `Object`, `Array`, or named types.

`[NOTE: BEHAVIOR]` `Coroutine` and `Thread` map to Lua's coroutine and thread types. They have limited type-checker support — primarily used in stdlib declarations.

## Literal Types

```rust
TypeKind::Literal(Literal)
```

Literal types narrow a primitive to a specific value:

| Literal Kind | Example | Narrows |
| ------------ | ------- | ------- |
| `Literal::Number(f64)` | `type X = 42` | `Number` |
| `Literal::Integer(i64)` | `type X = 42` | `Integer` |
| `Literal::String(String)` | `type X = "hello"` | `String` |
| `Literal::Boolean(bool)` | `type X = true` | `Boolean` |

Literal types are assignable to their base primitive but not the reverse.

## Union Types

```rust
TypeKind::Union(&'arena [Type<'arena>])
```

A value that can be one of several types: `string | number | nil`

- Assignability: A value is assignable to a union if it's assignable to any member.
- A union is assignable to a target if all members are assignable to the target.

## Intersection Types

```rust
TypeKind::Intersection(&'arena [Type<'arena>])
```

A value that satisfies all types simultaneously: `Serializable & Printable`

- Primarily used for combining interfaces and object types.
- For object types, intersection merges members from all constituent types.

## Nullable Types

```rust
TypeKind::Nullable(&'arena Type<'arena>)
```

Shorthand for `T | nil`. Written as `T?` in source code.

- `string?` is equivalent to `string | nil`
- Used extensively in optional parameters and return types

## Array Types

```rust
TypeKind::Array(&'arena Type<'arena>)
```

Typed array: `number[]`, `string[]`

- Compiles to Lua tables with sequential integer keys
- Element type is the inner type

## Tuple Types

```rust
TypeKind::Tuple(&'arena [Type<'arena>])
```

Fixed-length array with per-position types: `[string, number, boolean]`

- Each position has its own type
- Length is part of the type (a 3-tuple is not assignable to a 2-tuple)

## Object Types

```rust
TypeKind::Object(ObjectType<'arena>)

struct ObjectType<'arena> {
    members: &'arena [ObjectTypeMember<'arena>],
    span: Span,
}

enum ObjectTypeMember<'arena> {
    Property(PropertySignature),   // name: Type
    Method(MethodSignature),       // name(params): ReturnType
    Index(IndexSignature),         // [key: string]: Type
}
```

Object types describe the shape of a table with named properties:

```lua
{ name: string, age: number, greet(): void }
```

### Property Signatures

```rust
struct PropertySignature<'arena> {
    is_readonly: bool,
    name: Ident,
    is_optional: bool,
    type_annotation: Type<'arena>,
    span: Span,
}
```

### Method Signatures

```rust
struct MethodSignature<'arena> {
    name: Ident,
    type_parameters: Option<&'arena [TypeParameter<'arena>]>,
    parameters: &'arena [Parameter<'arena>],
    return_type: Type<'arena>,
    body: Option<Block<'arena>>,   // for interface default methods
    span: Span,
}
```

`[NOTE: BEHAVIOR]` Interface methods can have default implementations via the `body` field.

### Index Signatures

```rust
struct IndexSignature<'arena> {
    key_name: Ident,
    key_type: IndexKeyType,    // String or Number
    value_type: Type<'arena>,
    span: Span,
}
```

Allows arbitrary key access: `[key: string]: number`

## Function Types

```rust
TypeKind::Function(FunctionType<'arena>)

struct FunctionType<'arena> {
    type_parameters: Option<&'arena [TypeParameter<'arena>]>,
    parameters: &'arena [Parameter<'arena>],
    return_type: &'arena Type<'arena>,
    throws: Option<&'arena [Type<'arena>]>,
    span: Span,
}
```

Describes a callable: `(x: number, y: number) -> number`

- `throws` clause lists exception types the function may throw

## Type References

```rust
TypeKind::Reference(TypeReference<'arena>)

struct TypeReference<'arena> {
    name: Ident,
    type_arguments: Option<&'arena [Type<'arena>]>,
    span: Span,
}
```

Named reference to a type alias, class, interface, or enum: `MyClass<T>`

- Resolved during type checking via the symbol table
- `type_arguments` carries generic instantiation: `Map<string, number>`

## Parenthesized Types

```rust
TypeKind::Parenthesized(&'arena Type<'arena>)
```

Preserves grouping for display: `(string | number)[]`

## Type Query

```rust
TypeKind::TypeQuery(&'arena Expression<'arena>)
```

`typeof expr` — extracts the type of an expression at compile time.

## Namespace Types

```rust
TypeKind::Namespace(Vec<String>)
```

File-based namespace type for `namespace` declarations.

## Cross-References

- [Type Advanced](type-advanced.md) — generics, conditional, mapped, keyof, template literal, infer
- [Type Compatibility](type-compatibility.md) — assignability rules between these types
- [Type Checking](type-checking.md) — how these types are inferred and validated
- [AST](ast.md) — Type node in the AST hierarchy
