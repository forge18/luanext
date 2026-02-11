# Pattern Matching

Pattern matching provides a powerful way to destructure and test values. LuaNext supports match expressions with identifier, literal, array, object, wildcard, and or patterns, along with guards for conditional matching.

## Syntax

```lua
match expression
    | pattern [if guard] -> expression
    | pattern [if guard] -> expression
    | pattern [if guard] ->
        -- block
    end
end
```

## Examples

### Basic Pattern Matching

Match against literal values:

```lua
function getStatusMessage(code: number): string
    return match code
        | 200 -> "OK"
        | 404 -> "Not Found"
        | 500 -> "Internal Server Error"
        | _ -> "Unknown Status"
    end
end

print(getStatusMessage(200))  -- OK
print(getStatusMessage(999))  -- Unknown Status
```

Compiles to:

```lua
local function getStatusMessage(code)
    local _match = code
    if _match == 200 then
        return "OK"
    elseif _match == 404 then
        return "Not Found"
    elseif _match == 500 then
        return "Internal Server Error"
    else
        return "Unknown Status"
    end
end

print(getStatusMessage(200))
print(getStatusMessage(999))
```

### Identifier Pattern

Bind values to identifiers:

```lua
const value = 42

const result = match value
    | x -> x * 2
end

print(result)  -- 84
```

### Wildcard Pattern

The wildcard pattern `_` matches any value without binding:

```lua
function isPositive(x: number): boolean
    return match x
        | 0 -> false
        | _ if x > 0 -> true
        | _ -> false
    end
end
```

### Array Patterns

Destructure arrays with pattern matching:

```lua
function describeArray(arr: number[]): string
    return match arr
        | [] -> "Empty array"
        | [x] -> `Single element: ${x}`
        | [x, y] -> `Two elements: ${x}, ${y}`
        | [x, y, z] -> `Three elements: ${x}, ${y}, ${z}`
        | _ -> "More than three elements"
    end
end

print(describeArray({}))         -- Empty array
print(describeArray({1}))        -- Single element: 1
print(describeArray({1, 2}))     -- Two elements: 1, 2
print(describeArray({1, 2, 3}))  -- Three elements: 1, 2, 3
print(describeArray({1, 2, 3, 4}))  -- More than three elements
```

### Rest Pattern

Capture remaining array elements:

```lua
function getFirstAndRest(arr: number[]): {first: number, rest: number[]}
    return match arr
        | [] -> {first = 0, rest = {}}
        | [first, ...rest] -> {first = first, rest = rest}
    end
end

const result = getFirstAndRest({1, 2, 3, 4})
print(result.first)        -- 1
print(#result.rest)        -- 3
print(result.rest[1])      -- 2
```

### Object Patterns

Destructure objects/tables:

```lua
interface Point
    x: number
    y: number
end

function describePoint(p: Point): string
    return match p
        | {x = 0, y = 0} -> "Origin"
        | {x = 0, y} -> `On Y axis at ${y}`
        | {x, y = 0} -> `On X axis at ${x}`
        | {x, y} -> `Point at (${x}, ${y})`
    end
end

print(describePoint({x = 0, y = 0}))   -- Origin
print(describePoint({x = 0, y = 5}))   -- On Y axis at 5
print(describePoint({x = 3, y = 0}))   -- On X axis at 3
print(describePoint({x = 3, y = 4}))   -- Point at (3, 4)
```

### Guards

Add conditional checks to patterns:

```lua
function classifyNumber(x: number): string
    return match x
        | 0 -> "Zero"
        | n if n > 0 and n < 10 -> "Small positive"
        | n if n >= 10 and n < 100 -> "Medium positive"
        | n if n >= 100 -> "Large positive"
        | n if n < 0 -> "Negative"
        | _ -> "Unknown"
    end
end

print(classifyNumber(0))    -- Zero
print(classifyNumber(5))    -- Small positive
print(classifyNumber(50))   -- Medium positive
print(classifyNumber(150))  -- Large positive
print(classifyNumber(-5))   -- Negative
```

### Or Patterns

Match multiple patterns in one arm:

```lua
function isWeekend(day: string): boolean
    return match day
        | "Saturday" | "Sunday" -> true
        | _ -> false
    end
end

print(isWeekend("Saturday"))  -- true
print(isWeekend("Monday"))    -- false
```

### Nested Patterns

Patterns can be nested:

```lua
interface User
    name: string
    address: {city: string, zip: string}
end

function getCityFromUser(user: User): string
    return match user
        | {address = {city}} -> city
        | _ -> "Unknown city"
    end
end

const user: User = {
    name = "Alice",
    address = {city = "New York", zip = "10001"}
}

print(getCityFromUser(user))  -- New York
```

### Matching Enums

Pattern matching works excellently with enums:

```lua
enum Status
    Pending,
    Active,
    Completed,
    Failed
end

function getColor(status: Status): string
    return match status
        | Status.Pending -> "yellow"
        | Status.Active -> "green"
        | Status.Completed -> "blue"
        | Status.Failed -> "red"
    end
end

const status = Status.Active
print(getColor(status))  -- green
```

### Block Bodies

Match arms can have block bodies:

```lua
function processValue(x: number): number
    return match x
        | 0 ->
            print("Zero detected")
            return 1
        end
        | n if n < 0 ->
            print("Negative number")
            return -n
        end
        | n ->
            print("Positive number")
            return n * 2
        end
    end
end
```

### Exhaustiveness Checking

The compiler warns if not all cases are handled:

```lua
enum Color
    Red,
    Green,
    Blue
end

function getName(color: Color): string
    return match color
        | Color.Red -> "red"
        | Color.Green -> "green"
        -- Missing Color.Blue!
        -- Warning: Non-exhaustive match
    end
end

-- Use wildcard to ensure exhaustiveness:
function getName2(color: Color): string
    return match color
        | Color.Red -> "red"
        | Color.Green -> "green"
        | _ -> "other"
    end
end
```

### Matching Complex Types

Match against complex union types:

```lua
type Result<T, E> = {success: true, value: T} | {success: false, error: E}

function handleResult<T>(result: Result<T, string>): T | nil
    return match result
        | {success = true, value} -> value
        | {success = false, error} ->
            print(`Error: ${error}`)
            return nil
        end
    end
end

const result1: Result<number, string> = {success = true, value = 42}
const result2: Result<number, string> = {success = false, error = "Failed"}

print(handleResult(result1))  -- 42
print(handleResult(result2))  -- nil (prints "Error: Failed")
```

### Matching Optional Values

Handle nullable values:

```lua
function getLength(str: string | nil): number
    return match str
        | nil -> 0
        | s -> #s
    end
end

print(getLength(nil))      -- 0
print(getLength("hello"))  -- 5
```

### Pattern Matching in Variable Declarations

Destructure in variable declarations:

```lua
const [x, y, z] = {1, 2, 3}
print(x, y, z)  -- 1 2 3

const {name, age} = {name = "Alice", age = 30}
print(name, age)  -- Alice 30

-- With rest pattern
const [first, ...rest] = {1, 2, 3, 4, 5}
print(first)     -- 1
print(#rest)     -- 4
```

### Pattern Matching in Function Parameters

Destructure function parameters:

```lua
function distance(p1: {x: number, y: number}, p2: {x: number, y: number}): number
    return math.sqrt((p2.x - p1.x) ^ 2 + (p2.y - p1.y) ^ 2)
end

-- With destructuring:
function distance2({x = x1, y = y1}: {x: number, y: number}, {x = x2, y = y2}: {x: number, y: number}): number
    return math.sqrt((x2 - x1) ^ 2 + (y2 - y1) ^ 2)
end
```

### Matching Multiple Values

Match against tuples:

```lua
function classifyPoint(x: number, y: number): string
    return match {x, y}
        | {0, 0} -> "Origin"
        | {0, _} -> "Y axis"
        | {_, 0} -> "X axis"
        | {x, y} if x == y -> "Diagonal"
        | _ -> "Other"
    end
end

print(classifyPoint(0, 0))  -- Origin
print(classifyPoint(0, 5))  -- Y axis
print(classifyPoint(3, 0))  -- X axis
print(classifyPoint(5, 5))  -- Diagonal
print(classifyPoint(3, 4))  -- Other
```

## Details

### Pattern Types

LuaNext supports six pattern types:

1. **Identifier** — `x`, `name`, `value` (binds to variable)
2. **Literal** — `42`, `"hello"`, `true`, `nil` (exact match)
3. **Array** — `[x, y, z]`, `[first, ...rest]` (array destructuring)
4. **Object** — `{x, y}`, `{name = n, age}` (object destructuring)
5. **Wildcard** — `_` (matches anything, no binding)
6. **Or** — `Red | Green | Blue` (match any alternative)

### Guard Evaluation

Guards are evaluated left to right:

```lua
match x
    | n if n > 10 -> "large"   -- Checked first
    | n if n > 5 -> "medium"   -- Checked if first guard fails
    | _ -> "small"              -- Fallback
end
```

### Pattern Precedence

Patterns are matched in order:

```lua
match x
    | _ -> "first"   -- Always matches, later patterns unreachable
    | 42 -> "second" -- Warning: Unreachable pattern
end
```

Put more specific patterns before general ones:

```lua
match x
    | 42 -> "forty-two"
    | _ -> "other"
end
```

### Exhaustiveness

Match expressions must handle all cases:

- Use wildcard `_` as catch-all
- Or handle all enum variants explicitly
- Compiler warns on non-exhaustive matches

### Array Pattern Constraints

- Rest pattern `...rest` must be last
- Can have at most one rest pattern
- Holes `_` skip elements: `[x, _, z]`

### Object Pattern Constraints

- Properties can be bound or matched
- `{x}` is shorthand for `{x = x}`
- `{x = value}` binds `x` to `value`

### Match Expression Return Types

All arms must return the same type:

```lua
const result: string = match x
    | 0 -> "zero"
    | 1 -> "one"
    | _ -> "many"
end
```

Different return types cause type error:

```lua
-- Error: Type mismatch
const result = match x
    | 0 -> "zero"   -- string
    | 1 -> 1        -- number (error!)
end
```

### Performance

Pattern matching compiles to efficient if-else chains:

- Literal comparisons use `==`
- Array/object patterns check structure
- Guards evaluated only when pattern matches
- No runtime overhead for exhaustiveness checking

## See Also

- [Enums](enums.md) — Pattern matching with enums
- [Type System](type-system.md) — Union types and narrowing
- [Functions](functions.md) — Destructuring parameters
- [Basics](basics.md) — Destructuring variable declarations
