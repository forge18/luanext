# Interfaces

Interfaces define the shape of objects without providing implementation. They enable structural typing, contracts for classes, and reusable type definitions.

## Syntax

```lua
interface InterfaceName[<TypeParams>] [extends Interface1, Interface2] {
    -- Property signatures
    [readonly] propertyName[?]: Type

    -- Method signatures
    methodName[<TypeParams>](parameters): ReturnType

    -- Index signatures
    [key: string | number]: ValueType
}
```

Note: Interfaces can use either braces `{}` or `end`.

## Examples

### Basic Interface

```lua
interface Point {
    x: number
    y: number
}

function distance(p1: Point, p2: Point): number
    const dx = p2.x - p1.x
    const dy = p2.y - p1.y
    return math.sqrt(dx * dx + dy * dy)
}

const p1: Point = {x = 0, y = 0}
const p2: Point = {x = 3, y = 4}
print(distance(p1, p2))  -- 5.0
```

Compiles to:

```lua
local function distance(p1, p2)
    local dx = p2.x - p1.x
    local dy = p2.y - p1.y
    return math.sqrt(dx * dx + dy * dy)
}

local p1 = {x = 0, y = 0}
local p2 = {x = 3, y = 4}
print(distance(p1, p2))
```

### Method Signatures

Interfaces can declare method signatures:

```lua
interface Logger
    function log(message: string): void
    function warn(message: string): void
    function error(message: string): void
}

function setupLogging(logger: Logger): void
    logger:log("System started")
    logger:warn("Low memory")
    logger:error("Fatal error")
}

-- Implementing with a table
const consoleLogger: Logger = {
    log = function(self, message)
        print("[LOG] " .. message)
    end,
    warn = function(self, message)
        print("[WARN] " .. message)
    end,
    error = function(self, message)
        print("[ERROR] " .. message)
    end
}

setupLogging(consoleLogger)
```

### Optional Properties

Properties marked with `?` are optional:

```lua
interface Config
    host: string
    port: number
    ssl?: boolean
    timeout?: number
}

const config1: Config = {
    host = "localhost",
    port = 8080
}

const config2: Config = {
    host = "example.com",
    port = 443,
    ssl = true,
    timeout = 5000
}
```

### Readonly Properties

Readonly properties cannot be reassigned:

```lua
interface User
    readonly id: string
    name: string
    email: string
}

const user: User = {
    id = "user-123",
    name = "Alice",
    email = "alice@example.com"
}

user.name = "Bob"  -- ✅ OK

-- Error: Cannot assign to readonly property
-- user.id = "user-456"  -- ❌ Type error
```

### Index Signatures

Index signatures allow dynamic property access:

```lua
interface Dictionary
    [key: string]: number
}

const scores: Dictionary = {
    alice = 95,
    bob = 87,
    charlie = 92
}

print(scores.alice)       -- 95
print(scores["bob"])      -- 87
scores.diana = 89         -- ✅ OK
```

Array-like interfaces use number index:

```lua
interface NumberArray
    [index: number]: number
    length: number
}

const numbers: NumberArray = {1, 2, 3, 4, 5, length = 5}
print(numbers[1])  -- 1
print(numbers.length)  -- 5
```

### Extending Interfaces

Interfaces can extend other interfaces:

```lua
interface Named
    name: string
}

interface Aged
    age: number
}

interface Person extends Named, Aged
    email: string
}

const person: Person = {
    name = "Alice",
    age = 30,
    email = "alice@example.com"
}
```

### Generic Interfaces

Interfaces can be parameterized with type variables:

```lua
interface Box<T>
    value: T
    function get(): T
    function set(value: T): void
}

const numberBox: Box<number> = {
    value = 42,
    get = function(self)
        return self.value
    end,
    set = function(self, value)
        self.value = value
    end
}

const stringBox: Box<string> = {
    value = "hello",
    get = function(self)
        return self.value
    end,
    set = function(self, value)
        self.value = value
    end
}
```

### Implementing Interfaces in Classes

Classes can implement interfaces:

```lua
interface Drawable
    function draw(): void
}

interface Resizable
    function resize(width: number, height: number): void
}

class Rectangle implements Drawable, Resizable {
    private width: number
    private height: number

    constructor(width: number, height: number) {
        self.width = width
        self.height = height
    }

    function draw(): void {
        print(`Drawing rectangle: ${self.width}x${self.height}`)
    }

    function resize(width: number, height: number): void {
        self.width = width
        self.height = height
    }
}

const rect = Rectangle.new(100, 50)
rect:draw()           -- Drawing rectangle: 100x50
rect:resize(200, 100)
rect:draw()           -- Drawing rectangle: 200x100
```

### Function Types in Interfaces

Interfaces can define function types:

```lua
interface Comparator<T>
    compare: (a: T, b: T) => number
}

const numberComparator: Comparator<number> = {
    compare = function(a, b)
        return a - b
    end
}

function sort<T>(array: T[], comparator: Comparator<T>): T[]
    -- Sort implementation
    return array
}

const numbers: number[] = {5, 2, 8, 1, 9}
sort(numbers, numberComparator)
```

### Default Implementations

Interfaces can provide default method implementations:

```lua
interface Logger
    function log(message: string): void
        print("[LOG] " .. message)
    end

    function warn(message: string): void
        print("[WARN] " .. message)
    end

    function error(message: string): void
        self:log("ERROR: " .. message)
    end
}

-- Use default implementations
const logger: Logger = {}
logger:log("Hello")        -- [LOG] Hello
logger:warn("Warning")     -- [WARN] Warning
logger:error("Failed")     -- [LOG] ERROR: Failed

-- Override specific methods
const customLogger: Logger = {
    log = function(self, message)
        print("[CUSTOM] " .. message)
    end
}
customLogger:log("Test")    -- [CUSTOM] Test
customLogger:error("Fail")  -- [CUSTOM] ERROR: Fail
```

### Hybrid Types

Interfaces can be both callable and have properties:

```lua
interface Counter
    count: number
    function increment(): void
    () => number  -- Call signature
}

const counter: Counter = setmetatable({
    count = 0,
    increment = function(self)
        self.count = self.count + 1
    end
}, {
    __call = function(self)
        return self.count
    end
})

counter:increment()
counter:increment()
print(counter())  -- 2 (callable)
print(counter.count)  -- 2 (property access)
```

### Structural Typing

LuaNext uses structural typing—any object matching the interface shape is compatible:

```lua
interface Point {
    x: number
    y: number
}

function printPoint(p: Point): void
    print(`(${p.x}, ${p.y})`)
}

-- All of these work (structural compatibility)
printPoint({x = 1, y = 2})
printPoint({x = 3, y = 4, z = 5})  -- Extra property OK
```

### Intersection Types

Combine multiple interfaces using `&`:

```lua
interface Named
    name: string
}

interface Aged
    age: number
}

type Person = Named & Aged

const person: Person = {
    name = "Alice",
    age = 30
}
```

### Excess Property Checking

Direct object literals are strictly checked:

```lua
interface Point {
    x: number
    y: number
}

-- Error: Object literal may only specify known properties
-- const p: Point = {x = 1, y = 2, z = 3}  -- ❌ Type error

-- But this works (assigned to variable first)
const obj = {x = 1, y = 2, z = 3}
const p: Point = obj  -- ✅ OK (structural typing)
```

## Details

### Interface Merging

Multiple interface declarations with the same name merge:

```lua
interface User
    name: string
}

interface User
    email: string
}

-- Merged: User has both name and email
const user: User = {
    name = "Alice",
    email = "alice@example.com"
}
```

### Method vs Function Property

Two syntaxes for methods:

```lua
interface Example
    -- Method signature (preferred for methods)
    function greet(name: string): string

    -- Function property signature (preferred for callbacks)
    onComplete: (result: string) => void
}
```

Both are functionally equivalent, but the convention is:
- Use `function` for object methods called with `:`
- Use `=>` for function properties/callbacks

### Index Signature Restrictions

- Only `string` and `number` are valid index key types
- Cannot have both string and number index signatures with different value types
- Named properties must be compatible with index signature:

```lua
interface StringMap
    [key: string]: string
    count: number  -- ❌ Error: 'number' not assignable to 'string'
}
```

### Generic Constraints

Generic interfaces can have type constraints:

```lua
interface Comparable<T extends {id: number}>
    function compareTo(other: T): number
}

interface User
    id: number
    name: string
}

const userComparator: Comparable<User> = {
    compareTo = function(self, other)
        return self.id - other.id
    end
}
```

## See Also

- [Classes](classes.md) — Implementing interfaces with classes
- [Type System](type-system.md) — Advanced type features
- [Advanced Types](advanced-types.md) — Conditional and mapped types
