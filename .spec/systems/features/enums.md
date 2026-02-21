# Enums

Simple enums with literal values and rich enums with fields, methods, and interfaces.

## Overview

LuaNext supports two enum styles: simple enums (mapping names to values) and rich enums (with fields, constructors, methods, and interface conformance).

**Parser**: `crates/luanext-parser/src/ast/statement.rs` (`EnumDeclaration`)
**Codegen**: `crates/luanext-core/src/codegen/enums.rs`

## Enum Declaration

```rust
struct EnumDeclaration<'arena> {
    name: Ident,
    members: &'arena [EnumMember<'arena>],
    fields: &'arena [EnumField<'arena>],
    constructor: Option<EnumConstructor<'arena>>,
    methods: &'arena [EnumMethod<'arena>],
    implements: &'arena [Type<'arena>],
    span: Span,
}
```

An enum is **simple** if it has no fields, constructor, or methods. Otherwise it is **rich**.

## Simple Enums

```lua
enum Color {
    Red = 1,
    Green = 2,
    Blue = 3,
}

enum Direction {
    Up = "up",
    Down = "down",
    Left = "left",
    Right = "right",
}
```

### Member Values

```rust
struct EnumMember<'arena> {
    name: Ident,
    arguments: &'arena [Expression<'arena>],
    value: Option<EnumValue>,
    span: Span,
}

enum EnumValue {
    Number(f64),
    String(String),
}
```

If no value is specified, members auto-increment from the previous numeric value (starting at 0).

### Codegen

Simple enums compile to a plain Lua table:

```lua
local Color = {
    Red = 1,
    Green = 2,
    Blue = 3,
}
```

## Rich Enums

Rich enums have fields, constructors, and methods — they behave like sealed algebraic data types:

```lua
enum Planet {
    Mercury(3.303e23, 2.4397e6),
    Venus(4.869e24, 6.0518e6),
    Earth(5.976e24, 6.37814e6),

    mass: number
    radius: number

    constructor(mass: number, radius: number) {
        self.mass = mass
        self.radius = radius
    }

    surfaceGravity(): number {
        return 6.67300E-11 * self.mass / (self.radius * self.radius)
    }
}
```

### Fields

```rust
struct EnumField<'arena> {
    name: Ident,
    type_annotation: Type<'arena>,
    span: Span,
}
```

### Constructor

```rust
struct EnumConstructor<'arena> {
    parameters: &'arena [Parameter<'arena>],
    body: Block<'arena>,
    span: Span,
}
```

### Methods

```rust
struct EnumMethod<'arena> {
    name: Ident,
    parameters: &'arena [Parameter<'arena>],
    return_type: Option<Type<'arena>>,
    body: Block<'arena>,
    span: Span,
}
```

### Interface Conformance

Rich enums can implement interfaces:

```lua
enum Status implements Displayable {
    Active, Inactive,
    display(): string { return self:name() }
}
```

### Codegen

Rich enums compile to class-like structures:

```lua
local Planet = {}
Planet.__index = Planet

function Planet.__new(mass, radius)
    local self = setmetatable({}, Planet)
    self.mass = mass
    self.radius = radius
    return self
end

-- Members are constructed via __new
Planet.Mercury = Planet.__new(3.303e23, 2.4397e6)
Planet.Venus = Planet.__new(4.869e24, 6.0518e6)
Planet.Earth = Planet.__new(5.976e24, 6.37814e6)

-- Built-in methods
Planet.__values = { Planet.Mercury, Planet.Venus, Planet.Earth }
Planet.__byName = { Mercury = Planet.Mercury, Venus = Planet.Venus, Earth = Planet.Earth }

function Planet:name() ... end
function Planet:ordinal() ... end
function Planet:equals(other) ... end

-- User methods
function Planet:surfaceGravity()
    return 6.67300E-11 * self.mass / (self.radius * self.radius)
end
```

Built-in methods on rich enums:

| Method | Returns |
| ------ | ------- |
| `name()` | String name of the member |
| `ordinal()` | Integer index of the member |
| `equals(other)` | Boolean equality comparison |

`[NOTE: BEHAVIOR]` Rich enum codegen had several bugs that were fixed: missing comma in member list, incomplete `__new` function, and duplicate `setmetatable` call.

## Cross-References

- [Pattern Matching](pattern-matching.md) — destructuring enum values in match expressions
- [AST](../language/ast.md) — EnumDeclaration, EnumMember, EnumField
- [Runtime Library](../compiler/runtime-library.md) — enum runtime support
