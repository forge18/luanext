# Functions

Functions in LuaNext support type annotations, generics, default parameters, rest parameters, and more.

## Function Declarations

### Basic Function

```lua
function greet(name: string): string
    return "Hello, " .. name
end

const message: string = greet("Alice")
```

Compiles to:

```lua
local function greet(name)
    return "Hello, " .. name
end

local message = greet("Alice")
```

### Without Return Type

Return type can be inferred:

```lua
function add(a: number, b: number)
    return a + b  -- Return type inferred as number
end
```

### Void Functions

Functions that don't return a value:

```lua
function log(message: string): void
    print(message)
end
```

## Arrow Functions

Concise function syntax:

```lua
const add = (a: number, b: number): number => a + b

const greet = (name: string): string => {
    return "Hello, " .. name
}
```

Single-expression arrows don't need braces or `return`:

```lua
const double = (x: number): number => x * 2
```

Compiles to:

```lua
local add = function(a, b)
    return a + b
end

local double = function(x)
    return x * 2
end
```

## Function Parameters

### Required Parameters

```lua
function greet(first: string, last: string): string
    return "Hello, " .. first .. " " .. last
end
```

### Optional Parameters

Use `?` for optional parameters:

```lua
function greet(name: string, title?: string): string
    if title then
        return title .. " " .. name
    else
        return name
    end
end

greet("Alice")              -- OK
greet("Alice", "Dr.")       -- OK
```

Optional parameters must come after required parameters.

### Default Parameters

Provide default values:

```lua
function greet(name: string, greeting: string = "Hello"): string
    return greeting .. ", " .. name
end

greet("Alice")              -- "Hello, Alice"
greet("Bob", "Hi")          -- "Hi, Bob"
```

Compiles to:

```lua
local function greet(name, greeting)
    if greeting == nil then
        greeting = "Hello"
    end
    return greeting .. ", " .. name
end
```

### Rest Parameters

Collect remaining arguments into an array:

```lua
function sum(...numbers: number[]): number
    local total: number = 0
    for i, n in ipairs(numbers) do
        total = total + n
    end
    return total
end

sum(1, 2, 3, 4, 5)  -- 15
```

Compiles to:

```lua
local function sum(...)
    local numbers = {...}
    local total = 0
    for i, n in ipairs(numbers) do
        total = total + n
    end
    return total
end
```

## Multiple Return Values

Functions can return multiple values:

```lua
function divmod(a: number, b: number): [number, number]
    return math.floor(a / b), a % b
end

const quotient, remainder = divmod(17, 5)  -- 3, 2
```

Named return values (documentation only):

```lua
function process(data: string): [result: boolean, error: string | nil]
    if data == "" then
        return false, "Empty data"
    end
    return true, nil
end
```

## Generics

Type-safe generic functions:

```lua
function identity<T>(value: T): T
    return value
end

const num: number = identity(42)
const str: string = identity("hello")
```

Multiple type parameters:

```lua
function pair<T, U>(first: T, second: U): [T, U]
    return first, second
end

const result = pair("Alice", 30)  -- [string, number]
```

### Generic Constraints

Restrict type parameters:

```lua
interface HasLength {
    length: number
}

function getLength<T extends HasLength>(value: T): number
    return value.length
end

getLength("hello")      -- OK (string has length)
getLength({1, 2, 3})    -- OK (array has length)
getLength(42)           -- Error: number doesn't have length
```

### Generic Defaults

Provide default types:

```lua
function createArray<T = string>(size: number): T[]
    const arr: T[] = {}
    return arr
end

const strings = createArray(10)        -- string[] (default)
const numbers = createArray<number>(5) -- number[]
```

## Function Types

Define function signatures as types:

```lua
type BinaryOp = (a: number, b: number) -> number

const add: BinaryOp = (a, b) => a + b
const multiply: BinaryOp = (a, b) => a * b
```

With generics:

```lua
type Mapper<T, U> = (value: T) -> U

const toString: Mapper<number, string> = (n) => tostring(n)
```

## Higher-Order Functions

Functions that take or return functions:

```lua
function map<T, U>(array: T[], fn: (value: T) -> U): U[]
    const result: U[] = {}
    for i, value in ipairs(array) do
        result[i] = fn(value)
    end
    return result
end

const numbers: number[] = {1, 2, 3, 4}
const doubled: number[] = map(numbers, (x) => x * 2)  -- {2, 4, 6, 8}
```

## Method Syntax

Call methods with `:` (same as Lua):

```lua
interface Counter {
    count: number,
    increment: (self: Counter) -> void
}

const counter: Counter = {
    count = 0,
    increment = function(self)
        self.count = self.count + 1
    end
}

-- Method call syntax
counter:increment()

-- Equivalent to
counter.increment(counter)
```

## Closures

Functions can capture variables from outer scope:

```lua
function makeCounter(): () -> number
    local count: number = 0

    return function(): number
        count = count + 1
        return count
    end
end

const counter = makeCounter()
print(counter())  -- 1
print(counter())  -- 2
print(counter())  -- 3
```

## Recursive Functions

Functions can call themselves:

```lua
function factorial(n: number): number
    if n <= 1 then
        return 1
    end
    return n * factorial(n - 1)
end
```

Mutual recursion:

```lua
function isEven(n: number): boolean
    if n == 0 then
        return true
    end
    return isOdd(n - 1)
end

function isOdd(n: number): boolean
    if n == 0 then
        return false
    end
    return isEven(n - 1)
end
```

## Tail Call Optimization

Lua optimizes tail calls. LuaNext preserves this:

```lua
function factorial(n: number, acc: number = 1): number
    if n <= 1 then
        return acc
    end
    return factorial(n - 1, n * acc)  -- Tail call
end
```

## Function Overloading (Type-Only)

Multiple signatures for documentation:

```lua
-- Type declarations
declare function process(value: string): string
declare function process(value: number): number

-- Implementation
function process(value: string | number): string | number
    if type(value) == "string" then
        return value:upper()
    else
        return value * 2
    end
end
```

## Throws Clause

Document error types (informational):

```lua
function readFile(path: string): string throws Error
    const file = io.open(path, "r")
    if not file then
        throw {message = "File not found"}
    end
    const content: string = file:read("*a")
    file:close()
    return content
end
```

The `throws` clause documents potential errors but doesn't enforce them at compile time.

## Variadic Return Types

Functions with variable return values:

```lua
function find<T>(array: T[], predicate: (value: T) -> boolean): ...T | nil
    for i, value in ipairs(array) do
        if predicate(value) then
            return value, i  -- Returns T, number
        end
    end
    return nil
end
```

## Anonymous Functions

Functions without names:

```lua
const numbers: number[] = {1, 2, 3, 4, 5}

const doubled = map(numbers, function(x: number): number
    return x * 2
end)
```

## Immediately Invoked Function Expressions (IIFE)

```lua
const result: number = (function(): number
    const temp: number = calculate()
    return temp * 2
end)()
```

## Next Steps

- [Type System](type-system.md) — Generics, unions, advanced types
- [Classes](classes.md) — Methods and constructors
- [Error Handling](error-handling.md) — Try-catch and error types

## See Also

- [Basics](basics.md) — Variable declarations
- [Advanced Types](advanced-types.md) — Function type utilities
- [Operators](operators.md) — Function call operators
