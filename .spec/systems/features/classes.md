# Classes

Class declarations, inheritance, access control, and primary constructors.

## Overview

LuaNext classes compile to Lua tables with metatables for inheritance and method dispatch. The class system spans the parser (AST), type checker (access control, validation), and codegen (runtime table construction).

**Parser**: `crates/luanext-parser/src/ast/statement.rs` (`ClassDeclaration`)
**Codegen**: `crates/luanext-core/src/codegen/classes.rs`
**Access Control**: `crates/luanext-typechecker/src/visitors/access_control.rs`

## Class Declaration

```rust
struct ClassDeclaration<'arena> {
    decorators: &'arena [Decorator<'arena>],
    is_abstract: bool,
    is_final: bool,
    name: Ident,
    type_parameters: Option<&'arena [TypeParameter<'arena>]>,
    primary_constructor: Option<&'arena [ConstructorParameter<'arena>]>,
    extends: Option<Type<'arena>>,
    parent_constructor_args: Option<&'arena [Expression<'arena>]>,
    implements: &'arena [Type<'arena>],
    members: &'arena [ClassMember<'arena>],
    is_forward_declaration: bool,
    span: Span,
}
```

### Syntax Examples

```lua
-- Basic class
class Animal {
    public name: string
    constructor(name: string) {
        self.name = name
    }
    greet(): string {
        return "I am " .. self.name
    }
}

-- Inheritance
class Dog extends Animal {
    private breed: string
    constructor(name: string, breed: string) {
        super(name)
        self.breed = breed
    }
}

-- Interfaces
class Printable implements Serializable, Displayable { ... }
```

## Class Members

```rust
enum ClassMember<'arena> {
    Property(PropertyDeclaration),
    Constructor(ConstructorDeclaration),
    Method(MethodDeclaration),
    Getter(GetterDeclaration),
    Setter(SetterDeclaration),
    Operator(OperatorDeclaration),
}
```

### Properties

```rust
struct PropertyDeclaration<'arena> {
    decorators: &'arena [Decorator<'arena>],
    access: Option<AccessModifier>,   // Public / Private / Protected
    is_static: bool,
    is_readonly: bool,
    name: Ident,
    type_annotation: Type<'arena>,
    initializer: Option<Expression<'arena>>,
    span: Span,
}
```

### Methods

```rust
struct MethodDeclaration<'arena> {
    decorators: &'arena [Decorator<'arena>],
    access: Option<AccessModifier>,
    is_static: bool,
    is_abstract: bool,
    is_final: bool,
    is_override: bool,
    name: Ident,
    type_parameters: Option<&'arena [TypeParameter<'arena>]>,
    parameters: &'arena [Parameter<'arena>],
    return_type: Option<Type<'arena>>,
    body: Option<Block<'arena>>,   // None for abstract methods
    span: Span,
}
```

### Constructors

```rust
struct ConstructorDeclaration<'arena> {
    decorators: &'arena [Decorator<'arena>],
    parameters: &'arena [Parameter<'arena>],
    body: Block<'arena>,
    span: Span,
}
```

## Primary Constructors

Compact syntax that declares constructor parameters and class properties in one place:

```lua
class Point(public x: number, private readonly y: number)
```

This is equivalent to:

```lua
class Point {
    public x: number
    private readonly y: number
    constructor(x: number, y: number) {
        self.x = x
        self.y = y
    }
}
```

```rust
struct ConstructorParameter<'arena> {
    decorators: &'arena [Decorator<'arena>],
    access: Option<AccessModifier>,
    is_readonly: bool,
    name: Ident,
    type_annotation: Type<'arena>,
    default: Option<Expression<'arena>>,
    span: Span,
}
```

## Access Modifiers

```rust
enum AccessModifier {
    Public,      // accessible everywhere
    Private,     // only within declaring class
    Protected,   // within declaring class + subclasses
}
```

Default access is `public` when no modifier is specified.

`[NOTE: BEHAVIOR]` Access control is enforced at the type checker level, not at runtime. The generated Lua code has no access restriction mechanisms.

## Inheritance

### Extends

Single inheritance via `extends`:

```lua
class Dog extends Animal { ... }
```

- Subclass inherits all members from parent
- Constructor must call `super(args)` if parent has a constructor
- `parent_constructor_args` passes arguments to parent constructor in primary constructor syntax

### Implements

Multiple interface conformance via `implements`:

```lua
class MyClass implements Serializable, Comparable<MyClass> { ... }
```

The type checker validates that all interface members are satisfied.

## Forward Declarations

```rust
is_forward_declaration: bool
```

Enables mutual references between classes:

```lua
class A   -- forward declaration
class B {
    other: A
}
class A {
    other: B
}
```

`[NOTE: BEHAVIOR]` Forward-declared classes have no members in their first appearance. The full declaration must follow.

## Codegen: Lua Table Construction

Classes compile to Lua using metatables:

```lua
-- class Animal
local Animal = {}
Animal.__index = Animal
Animal.__metatable = "Animal"

function Animal.new(name)
    local self = setmetatable({}, Animal)
    self.name = name
    return self
end

function Animal:greet()
    return "I am " .. self.name
end
```

### Inheritance Codegen

```lua
-- class Dog extends Animal
local Dog = setmetatable({}, { __index = Animal })
Dog.__index = Dog
Dog.__metatable = "Dog"

function Dog.new(name, breed)
    local self = Animal.new(name)  -- parent constructor
    setmetatable(self, Dog)
    self.breed = breed
    return self
end
```

### Static Members

Static members are stored directly on the class table:

```lua
Animal.count = 0
function Animal.getCount()
    return Animal.count
end
```

## Generics

Classes support type parameters:

```lua
class Container<T> {
    private items: T[] = []
    add(item: T): void { ... }
    get(index: number): T { ... }
}
```

Type parameters are erased at codegen (no runtime representation).

## Cross-References

- [Class Advanced](class-advanced.md) — operator overloading, decorators, getters/setters, abstract/final
- [AST](../language/ast.md) — ClassDeclaration, ClassMember nodes
- [Type Checking](../language/type-checking.md) — class type validation
- [Runtime Library](../compiler/runtime-library.md) — class runtime support code
