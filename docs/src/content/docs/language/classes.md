# Classes

LuaNext provides object-oriented programming features through classes with full type safety. Classes support inheritance, interfaces, access modifiers, operator overloading, and more.

## Syntax

Classes can use either braces `{}` or `end` to delimit the body:

**Braces syntax (preferred):**
```lua
[abstract | final] class ClassName[<TypeParams>][(ConstructorParams)] [extends ParentClass] [implements Interface1, Interface2] {
    -- Properties
    [access] [static] [readonly] propertyName: Type [= initializer]

    -- Constructor
    constructor(parameters) {
        -- initialization
    }

    -- Methods
    [access] [static] [abstract | final] [override] function methodName(parameters): ReturnType {
        -- body
    }
}
```

**End syntax:**
```lua
[abstract | final] class ClassName[<TypeParams>][(ConstructorParams)] [extends ParentClass] [implements Interface1, Interface2]
    -- Properties and methods
end
```

## Examples

### Basic Class

```lua
class Person {
    name: string
    age: number

    constructor(name: string, age: number) {
        self.name = name
        self.age = age
    }

    function greet(): string {
        return `Hello, I'm ${self.name} and I'm ${self.age} years old.`
    }
}

const alice = Person.new("Alice", 30)
print(alice:greet())  -- Hello, I'm Alice and I'm 30 years old.
```

Compiles to:

```lua
local Person = {}
Person.__index = Person

function Person.new(name, age)
    local self = setmetatable({}, Person)
    self.name = name
    self.age = age
    return self
end

function Person:greet()
    return "Hello, I'm " .. tostring(self.name) .. " and I'm " .. tostring(self.age) .. " years old."
end

local alice = Person.new("Alice", 30)
print(alice:greet())
```

### Primary Constructor

Primary constructors provide a compact syntax for declaring properties:

```lua
class Point(public x: number, public y: number) {
    function distance(): number {
        return math.sqrt(self.x * self.x + self.y * self.y)
    }
}

const p = Point.new(3, 4)
print(p:distance())  -- 5.0
```

Compiles to:

```lua
local Point = {}
Point.__index = Point

function Point.new(x, y)
    local self = setmetatable({}, Point)
    self.x = x
    self.y = y
    return self
end

function Point:distance()
    return math.sqrt(self.x * self.x + self.y * self.y)
end

local p = Point.new(3, 4)
print(p:distance())
```

### Access Modifiers

LuaNext supports three access modifiers:

- `public` — Accessible everywhere (default)
- `private` — Accessible only within the class
- `protected` — Accessible within the class and subclasses

```lua
class BankAccount(private balance: number) {
    public function deposit(amount: number): void {
        self.balance = self.balance + amount
    }

    public function getBalance(): number {
        return self.balance
    }

    private function validateAmount(amount: number): boolean {
        return amount > 0
    }
}

const account = BankAccount.new(100)
account:deposit(50)
print(account:getBalance())  -- 150

-- Error: Cannot access private member
-- print(account.balance)  -- ❌ Type error
```

### Static Members

Static members belong to the class itself, not instances:

```lua
class MathUtils {
    static PI: number = 3.14159

    static function square(x: number): number {
        return x * x
    }

    static function circleArea(radius: number): number {
        return MathUtils.PI * radius * radius
    }
}

print(MathUtils.square(5))          -- 25
print(MathUtils.circleArea(10))     -- 314.159
```

### Readonly Properties

Readonly properties can only be assigned in the constructor:

```lua
class User(public readonly id: string, public name: string) {
    constructor(id: string, name: string) {
        self.id = id
        self.name = name
    }

    function rename(newName: string): void {
        self.name = newName  -- ✅ OK

        -- Error: Cannot assign to readonly property
        -- self.id = "new-id"  -- ❌ Type error
    }
}
```

### Getters and Setters

Computed properties with custom logic:

```lua
class Temperature {
    private celsius: number

    constructor(celsius: number) {
        self.celsius = celsius
    }

    get fahrenheit(): number {
        return self.celsius * 9 / 5 + 32
    }

    set fahrenheit(value: number) {
        self.celsius = (value - 32) * 5 / 9
    }
}

const temp = Temperature.new(0)
print(temp.fahrenheit)  -- 32
temp.fahrenheit = 100
print(temp.celsius)     -- 37.777...
```

### Inheritance

Classes can extend other classes:

```lua
class Animal(public name: string) {
    function speak(): string {
        return `${self.name} makes a sound`
    }
}

class Dog extends Animal(name) {
    private breed: string

    constructor(name: string, breed: string) {
        super(name)  -- Call parent constructor
        self.breed = breed
    }

    override function speak(): string {
        return `${self.name} barks`
    }

    function getBreed(): string {
        return self.breed
    }
}

const dog = Dog.new("Rex", "Labrador")
print(dog:speak())      -- Rex barks
print(dog:getBreed())   -- Labrador
```

### Abstract Classes

Abstract classes cannot be instantiated directly:

```lua
abstract class Shape {
    abstract function area(): number
    abstract function perimeter(): number

    function describe(): string {
        return `Area: ${self:area()}, Perimeter: ${self:perimeter()}`
    }
}

class Circle extends Shape(private radius: number) {
    override function area(): number {
        return 3.14159 * self.radius * self.radius
    }

    override function perimeter(): number {
        return 2 * 3.14159 * self.radius
    }
}

-- Error: Cannot instantiate abstract class
-- const shape = Shape.new()  -- ❌ Type error

const circle = Circle.new(5)
print(circle:describe())  -- Area: 78.53975, Perimeter: 31.4159
```

### Final Classes and Methods

Final classes cannot be extended, final methods cannot be overridden:

```lua
final class String {
    -- Cannot be extended
}

class Base {
    final function important(): void {
        print("Important logic")
    }
}

class Derived extends Base {
    -- Error: Cannot override final method
    -- override function important(): void  -- ❌ Type error
    --     print("Modified")
    -- }
}
```

### Implementing Interfaces

Classes can implement one or more interfaces:

```lua
interface Serializable {
    function serialize(): string
}

interface Comparable<T> {
    function compareTo(other: T): number
}

class User implements Serializable, Comparable<User> {
    name: string
    age: number

    constructor(name: string, age: number) {
        self.name = name
        self.age = age
    }

    function serialize(): string {
        return `{"name":"${self.name}","age":${self.age}}`
    }

    function compareTo(other: User): number {
        return self.age - other.age
    }
}
```

### Operator Overloading

LuaNext supports overloading 24 operators:

**Arithmetic:** `+`, `-`, `*`, `/`, `%`, `^`, `//` (floor divide)

**Comparison:** `==`, `~=`, `<`, `<=`, `>`, `>=`

**Bitwise:** `&`, `|`, `~`, `<<`, `>>`

**Special:** `..` (concat), `#` (length), `[]` (index), `[]=` (newindex), `()` (call)

**Unary:** `-` (negate)

```lua
class Vector(public x: number, public y: number) {
    operator +(other: Vector): Vector {
        return Vector.new(self.x + other.x, self.y + other.y)
    }

    operator *(scalar: number): Vector {
        return Vector.new(self.x * scalar, self.y * scalar)
    }

    operator ==(other: Vector): boolean {
        return self.x == other.x and self.y == other.y
    }

    operator #(): number {
        return math.sqrt(self.x * self.x + self.y * self.y)
    }

    operator ...(other: Vector): string {
        return `(${self.x}, ${self.y}) + (${other.x}, ${other.y})`
    }
}

const v1 = Vector.new(1, 2)
const v2 = Vector.new(3, 4)
const v3 = v1 + v2              -- Vector(4, 6)
const v4 = v1 * 2               -- Vector(2, 4)
const equal = v1 == v2          -- false
const length = #v1              -- 2.236...
const str = v1 .. v2            -- "(1, 2) + (3, 4)"
```

### Generics

Classes can be generic over type parameters:

```lua
class Box<T> {
    private value: T

    constructor(value: T) {
        self.value = value
    }

    function get(): T {
        return self.value
    }

    function set(value: T): void {
        self.value = value
    }
}

const numberBox = Box<number>.new(42)
const stringBox = Box<string>.new("hello")

print(numberBox:get())  -- 42
print(stringBox:get())  -- hello
```

## Details

### Constructor Behavior

- Classes have an implicit `new` static method that creates instances
- The `constructor` method is called after the instance is created
- Primary constructor parameters automatically become properties
- Parent constructors must be called with `super()` in derived classes

### Access Modifier Enforcement

Access modifiers are enforced at compile time:

- `private` members are not accessible outside the class definition
- `protected` members are accessible in derived classes
- `public` members are accessible everywhere

At runtime, all members are public (Lua limitation), but the type checker prevents incorrect access.

### Method Resolution

- Methods are stored in the class's metatable (`__index`)
- Static methods are stored directly on the class table
- Method calls use `:` syntax (`obj:method()`) for automatic `self` passing

### Operator Overloading Mapping

LuaNext operators map to Lua metamethods:

| LuaNext | Lua Metamethod |
|---------|----------------|
| `+` | `__add` |
| `-` | `__sub` |
| `*` | `__mul` |
| `/` | `__div` |
| `%` | `__mod` |
| `^` | `__pow` |
| `//` | `__idiv` |
| `==` | `__eq` |
| `<` | `__lt` |
| `<=` | `__le` |
| `&` | `__band` |
| `|` | `__bor` |
| `~` | `__bxor` |
| `<<` | `__shl` |
| `>>` | `__shr` |
| `..` | `__concat` |
| `#` | `__len` |
| `[]` | `__index` |
| `[]=` | `__newindex` |
| `()` | `__call` |
| `-` (unary) | `__unm` |

### Abstract Class Checks

Abstract classes and methods are checked at compile time:

- Cannot instantiate abstract classes
- Must implement all abstract methods in concrete subclasses
- Abstract methods cannot have bodies

### Multiple Inheritance

LuaNext does not support multiple inheritance (extending multiple classes), but supports implementing multiple interfaces.

## See Also

- [Interfaces](interfaces.md) — Defining contracts for classes
- [Enums](enums.md) — Rich enums with fields and methods
- [Type System](type-system.md) — Generics and constraints
- [Decorators](decorators.md) — Annotating classes and members
