# Class Advanced

Operator overloading, decorators, getters/setters, and abstract/final modifiers.

## Overview

Advanced class features that build on the base class system. These span parser, type checker, and codegen.

## Abstract Classes and Methods

### Abstract Classes

```lua
abstract class Shape {
    abstract area(): number
    describe(): string {
        return "Shape with area " .. tostring(self.area())
    }
}
```

- Cannot be instantiated directly — runtime check in constructor throws error
- Can contain both abstract and concrete methods
- Subclasses must implement all abstract methods

`[NOTE: BEHAVIOR]` Abstract class instantiation is rejected at type-checker time. Codegen also emits a runtime guard for defense-in-depth.

### Abstract Methods

```rust
// MethodDeclaration with is_abstract: true, body: None
```

Abstract methods have no body. The type checker enforces implementation in concrete subclasses.

## Final Classes and Methods

### Final Classes

```lua
final class Singleton { ... }
```

- Cannot be extended — type checker rejects `class Sub extends Singleton`

### Final Methods

```lua
class Base {
    final compute(): number { return 42 }
}
```

- Cannot be overridden in subclasses

`[NOTE: BEHAVIOR]` Final validation happens at type-checker time, not runtime. No runtime enforcement is generated.

## Override

```lua
class Child extends Parent {
    override greet(): string { return "hello" }
}
```

`is_override: bool` on `MethodDeclaration`. The type checker validates that:

1. A matching method exists in the parent class
2. The signature is compatible

## Getters and Setters

```lua
class Temperature {
    private _celsius: number = 0

    get celsius(): number {
        return self._celsius
    }

    set celsius(value: number) {
        self._celsius = value
    }
}
```

```rust
struct GetterDeclaration<'arena> {
    decorators, access, is_static,
    name: Ident,
    return_type: Type<'arena>,
    body: Block<'arena>,
}

struct SetterDeclaration<'arena> {
    decorators, access, is_static,
    name: Ident,
    parameter: Parameter<'arena>,
    body: Block<'arena>,
}
```

### Codegen Naming

`[NOTE: BEHAVIOR]` Getters compile to `get_X()` methods, setters to `set_X()` methods in Lua. Access control registers them under the original name `X`. The type checker has a prefix-stripping fallback: when looking up method `get_X`, it also checks for member `X` and vice versa.

```lua
-- get celsius() compiles to:
function Temperature:get_celsius()
    return self._celsius
end

-- set celsius(v) compiles to:
function Temperature:set_celsius(value)
    self._celsius = value
end
```

## Operator Overloading

Classes can define custom behavior for Lua operators via metamethods:

```lua
class Vector {
    public x: number
    public y: number

    operator +(other: Vector): Vector {
        return new Vector(self.x + other.x, self.y + other.y)
    }

    operator ==(other: Vector): boolean {
        return self.x == other.x and self.y == other.y
    }

    operator #(): number {
        return math.sqrt(self.x * self.x + self.y * self.y)
    }
}
```

### Supported Operators (24 total)

```rust
enum OperatorKind {
    // Arithmetic
    Add, Subtract, Multiply, Divide, Modulo, Power,
    FloorDivide, Concatenate,
    // Comparison
    Equal, NotEqual, LessThan, LessThanOrEqual,
    GreaterThan, GreaterThanOrEqual,
    // Bitwise
    BitwiseAnd, BitwiseOr, BitwiseXor, ShiftLeft, ShiftRight,
    // Collection
    Index, NewIndex, Call,
    // Unary
    UnaryMinus, Length,
}
```

### Codegen

Operators compile to Lua metamethods on the class metatable:

| OperatorKind | Lua Metamethod |
| ------------ | -------------- |
| `Add` | `__add` |
| `Subtract` | `__sub` |
| `Multiply` | `__mul` |
| `Divide` | `__div` |
| `Modulo` | `__mod` |
| `Power` | `__pow` |
| `FloorDivide` | `__idiv` |
| `Concatenate` | `__concat` |
| `Equal` | `__eq` |
| `LessThan` | `__lt` |
| `LessThanOrEqual` | `__le` |
| `BitwiseAnd` | `__band` |
| `BitwiseOr` | `__bor` |
| `BitwiseXor` | `__bxor` |
| `ShiftLeft` | `__shl` |
| `ShiftRight` | `__shr` |
| `Index` | `__index` (custom) |
| `NewIndex` | `__newindex` |
| `Call` | `__call` |
| `UnaryMinus` | `__unm` |
| `Length` | `__len` |

`[NOTE: BEHAVIOR]` Operator overloading uses `self`, matching Lua metamethod conventions. This differs from constructor bodies which also use `self`.

`[NOTE: BEHAVIOR]` `NotEqual`, `GreaterThan`, `GreaterThanOrEqual` are defined in the AST but Lua derives them from their counterparts (`__eq`, `__lt`, `__le`).

## Decorators

**File**: `crates/luanext-core/src/codegen/decorators.rs`

```lua
@readonly
@deprecated("Use newMethod instead")
@log.method()
class MyClass { ... }
```

### Decorator Expression Forms

```rust
enum DecoratorExpression<'arena> {
    Identifier(Ident),                    // @name
    Call { callee, arguments, span },     // @name(args)
    Member { object, property, span },    // @obj.prop
}
```

Decorators can be applied to: classes, properties, methods, getters, setters, constructor parameters.

### Built-in Decorators

| Name | Target | Effect |
| ---- | ------ | ------ |
| `@readonly` | Class, Property | Marks as immutable |
| `@sealed` | Class | Prevents subclassing |
| `@deprecated` | Any | Marks as deprecated with optional message |

`[NOTE: BEHAVIOR]` Avoid naming user decorators `sealed`, `deprecated`, etc. — the stdlib exports these names globally, causing naming conflicts.

`[NOTE: BEHAVIOR]` Decorator functions must be table literals with function values, not `function NS.method(target)` syntax — that syntax is unsupported in LuaNext.

### Codegen

Decorators compile to function calls that wrap/modify the decorated target:

```lua
-- @deprecated("use other")
-- class Foo { ... }
local Foo = deprecated("use other")(Foo)
```

## Cross-References

- [Classes](classes.md) — base class system
- [AST](../language/ast.md) — OperatorKind, DecoratorExpression
- [Runtime Library](../compiler/runtime-library.md) — decorator runtime support
