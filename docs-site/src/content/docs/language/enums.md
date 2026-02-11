---
title: Enums
---

# Enums

LuaNext provides rich enums that go beyond simple value sets. Enums can have string or number values, fields, constructors, methods, and can implement interfaces.

## Syntax

```lua
enum EnumName [implements Interface1, Interface2]
    -- Simple members
    Member1,
    Member2 = value,

    -- Fields (shared across all members)
    fieldName: Type,

    -- Constructor
    constructor(parameters)
        -- initialization logic for fields
    end

    -- Methods
    function methodName(parameters): ReturnType
        -- body
    end
end
```

## Examples

### Basic Enum

Simple enums without explicit values:

```lua
enum Status
    Pending,
    Active,
    Inactive
end

const currentStatus: Status = Status.Active

if currentStatus == Status.Active then
    print("System is active")
end
```

Compiles to:

```lua
local Status = {
    Pending = "Pending",
    Active = "Active",
    Inactive = "Inactive"
}

local currentStatus = Status.Active

if currentStatus == Status.Active then
    print("System is active")
end
```

### String Enums

Enums with explicit string values:

```lua
enum Color
    Red = "red",
    Green = "green",
    Blue = "blue"
end

const primary: Color = Color.Red
print(primary)  -- "red"
```

###Number Enums

Enums with explicit number values:

```lua
enum HttpStatus
    OK = 200,
    NotFound = 404,
    InternalError = 500
end

function handleResponse(status: HttpStatus): string
    if status == HttpStatus.OK then
        return "Success"
    elseif status == HttpStatus.NotFound then
        return "Not found"
    else
        return "Error"
    end
end
```

Auto-incrementing number enums:

```lua
enum Priority
    Low = 1,
    Medium,   -- 2
    High,     -- 3
    Critical  -- 4
end
```

### Rich Enums with Fields

Enums can have fields shared across all members:

```lua
enum Direction
    North,
    East,
    South,
    West,

    -- Fields
    angle: number,

    -- Constructor initializes fields based on enum value
    constructor()
        if self == Direction.North then
            self.angle = 0
        elseif self == Direction.East then
            self.angle = 90
        elseif self == Direction.South then
            self.angle = 180
        elseif self == Direction.West then
            self.angle = 270
        end
    end
end

const dir = Direction.East
print(dir.angle)  -- 90
```

### Enums with Methods

Enums can have methods:

```lua
enum Status
    Pending,
    Active,
    Completed,

    function isActive(): boolean
        return self == Status.Active
    end

    function canTransitionTo(other: Status): boolean
        if self == Status.Pending then
            return other == Status.Active
        elseif self == Status.Active then
            return other == Status.Completed
        else
            return false
        end
    end
end

const status = Status.Pending
print(status:isActive())  -- false
print(status:canTransitionTo(Status.Active))  -- true
print(status:canTransitionTo(Status.Completed))  -- false
```

### Rich Enums with Fields and Methods

Combining fields, constructors, and methods:

```lua
enum Planet
    Mercury,
    Venus,
    Earth,
    Mars,

    -- Fields
    mass: number,           -- in kg
    radius: number,         -- in meters

    -- Constructor
    constructor()
        if self == Planet.Mercury then
            self.mass = 3.303e23
            self.radius = 2.4397e6
        elseif self == Planet.Venus then
            self.mass = 4.869e24
            self.radius = 6.0518e6
        elseif self == Planet.Earth then
            self.mass = 5.976e24
            self.radius = 6.37814e6
        elseif self == Planet.Mars then
            self.mass = 6.421e23
            self.radius = 3.3972e6
        end
    end

    -- Methods
    function surfaceGravity(): number
        const G = 6.67430e-11  -- gravitational constant
        return G * self.mass / (self.radius * self.radius)
    end

    function surfaceWeight(mass: number): number
        return mass * self:surfaceGravity()
    end
end

const earth = Planet.Earth
const earthMass = 70  -- kg

print(`Mass on Earth: ${earthMass} kg`)
print(`Weight on Earth: ${earth:surfaceWeight(earthMass)} N`)

const mars = Planet.Mars
print(`Weight on Mars: ${mars:surfaceWeight(earthMass)} N`)
```

### Enums with Constructor Arguments

Rich enums can accept constructor arguments per member:

```lua
enum LogLevel
    Debug(color = "gray"),
    Info(color = "blue"),
    Warn(color = "yellow"),
    Error(color = "red"),

    -- Fields
    color: string,
    severity: number,

    -- Constructor accepts arguments
    constructor(color: string)
        self.color = color

        -- Set severity based on enum value
        if self == LogLevel.Debug then
            self.severity = 0
        elseif self == LogLevel.Info then
            self.severity = 1
        elseif self == LogLevel.Warn then
            self.severity = 2
        elseif self == LogLevel.Error then
            self.severity = 3
        end
    end

    function format(message: string): string
        return `[${self.color}] ${message}`
    end
end

const level = LogLevel.Warn
print(level:format("Low memory"))  -- [yellow] Low memory
print(level.severity)               -- 2
```

### Implementing Interfaces

Enums can implement interfaces:

```lua
interface Describable
    function describe(): string
end

enum Animal implements Describable
    Dog,
    Cat,
    Bird,

    -- Fields
    sound: string,

    constructor()
        if self == Animal.Dog then
            self.sound = "bark"
        elseif self == Animal.Cat then
            self.sound = "meow"
        elseif self == Animal.Bird then
            self.sound = "chirp"
        end
    end

    -- Implement interface method
    function describe(): string
        return `A ${tostring(self)} that ${self.sound}s`
    end
end

const dog: Describable = Animal.Dog
print(dog:describe())  -- A Dog that barks
```

### Pattern Matching with Enums

Enums work well with pattern matching:

```lua
enum Result<T, E>
    Ok(value: T),
    Err(error: E),

    value: T | E,

    constructor(val: T | E)
        self.value = val
    end
end

function divide(a: number, b: number): Result<number, string>
    if b == 0 then
        return Result.Err("Division by zero")
    else
        return Result.Ok(a / b)
    end
end

const result = divide(10, 2)

match result
    | Result.Ok -> print(`Success: ${result.value}`)
    | Result.Err -> print(`Error: ${result.value}`)
end
```

### Mixed Enum Values

Enums can mix string and number values:

```lua
enum MixedEnum
    First = "first",
    Second = 2,
    Third = "third",
    Fourth = 4
end
```

However, this is generally discouraged—prefer consistent value types.

## Details

### Enum Value Types

Enums can have three types of values:

1. **Auto string** — Default, member name as string
2. **Explicit string** — Custom string values
3. **Explicit number** — Numeric values (can auto-increment)

### Enum Member Access

Enum members are accessed via the enum name:

```lua
const status = Status.Active
const color = Color.Red
```

### Enum Comparison

Enums can be compared with `==` and `~=`:

```lua
if currentStatus == Status.Active then
    print("Active")
end
```

### Type Safety

Enums provide compile-time type safety:

```lua
enum Status
    Active,
    Inactive
end

const status: Status = Status.Active  -- ✅ OK

-- Error: Type mismatch
-- const wrong: Status = "Active"  -- ❌ Type error
-- const wrong2: Status = Status.Unknown  -- ❌ Type error
```

### Enum Methods and `self`

In enum methods, `self` refers to the enum value:

```lua
enum Status
    Active,
    Inactive,

    function isActive(): boolean
        return self == Status.Active  -- self is the enum value
    end
end
```

### Fields vs Methods

- **Fields** — Data associated with each enum value (initialized in constructor)
- **Methods** — Behavior shared across all enum values (can access fields via `self`)

### Constructor Behavior

The constructor is called once per enum value when the enum is defined:

```lua
enum Counter
    A,
    B,
    C,

    count: number,

    constructor()
        print(`Creating ${tostring(self)}`)
        self.count = 1
    end
end

-- Prints:
-- Creating A
-- Creating B
-- Creating C
```

### Enum Reflection

Get enum member name as string:

```lua
const status = Status.Active
const name = tostring(status)  -- "Active"
```

Get all enum values:

```lua
enum Color
    Red,
    Green,
    Blue
end

-- Iterate over enum values (implementation-specific)
for name, value in pairs(Color) do
    print(name, value)
end
```

### Exhaustiveness Checking

Use pattern matching or if-else chains with `never` for exhaustiveness:

```lua
enum Status
    Pending,
    Active,
    Completed
end

function handleStatus(status: Status): void
    if status == Status.Pending then
        print("Pending")
    elseif status == Status.Active then
        print("Active")
    elseif status == Status.Completed then
        print("Completed")
    else
        -- This branch is unreachable if all cases are handled
        const _exhaustive: never = status
    end
end
```

## See Also

- [Classes](classes.md) — Object-oriented programming
- [Interfaces](interfaces.md) — Implementing interfaces with enums
- [Pattern Matching](pattern-matching.md) — Matching enum values
- [Type System](type-system.md) — Union types and type narrowing
