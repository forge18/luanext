---
title: Control Flow
---

# Control Flow

LuaNext supports all Lua control flow constructs plus additional features like `continue`.

## If Statements

### Basic If

```lua
const age: number = 25

if age >= 18 then
    print("Adult")
end
```

### If-Else

```lua
const score: number = 85

if score >= 90 then
    print("A")
else
    print("B or lower")
end
```

### If-Elseif-Else

```lua
const score: number = 85

if score >= 90 then
    print("A")
elseif score >= 80 then
    print("B")
elseif score >= 70 then
    print("C")
else
    print("F")
end
```

### Type Narrowing

LuaNext narrows types within conditional branches:

```lua
const value: string | number = getValue()

if type(value) == "string" then
    -- value is string here
    print(value:upper())
elseif type(value) == "number" then
    -- value is number here
    print(value * 2)
end
```

## While Loops

Execute block while condition is true:

```lua
local count: number = 0

while count < 5 do
    print(count)
    count = count + 1
end
```

## Repeat-Until Loops

Execute block at least once, until condition becomes true:

```lua
local count: number = 0

repeat
    print(count)
    count = count + 1
until count >= 5
```

## For Loops

### Numeric For

```lua
-- for var = start, end, step do
for i = 1, 10, 1 do
    print(i)
end

-- step defaults to 1
for i = 1, 10 do
    print(i)
end

-- Counting down
for i = 10, 1, -1 do
    print(i)
end
```

Type annotation on loop variable:

```lua
for i: integer = 1, 100 do
    print(i)
end
```

### Generic For

Iterate over tables, arrays, iterators:

```lua
-- Arrays
const numbers: number[] = {10, 20, 30, 40}

for i, value in ipairs(numbers) do
    print(i, value)
end

-- Tables
const user = {name = "Alice", age = 30}

for key, value in pairs(user) do
    print(key, value)
end

-- Custom iterators
for line in io.lines("file.txt") do
    print(line)
end
```

Type annotations:

```lua
for i: integer, value: number in ipairs(numbers) do
    print(i, value)
end
```

## Break

Exit loop early:

```lua
for i = 1, 100 do
    if i == 50 then
        break
    end
    print(i)
end
```

## Continue

Skip to next iteration (LuaNext extension):

```lua
for i = 1, 10 do
    if i % 2 == 0 then
        continue  -- Skip even numbers
    end
    print(i)  -- Only prints odd numbers
end
```

Compiles to a `goto` statement:

```lua
for i = 1, 10 do
    if i % 2 == 0 then
        goto continue_1
    end
    print(i)
    ::continue_1::
end
```

## Labels and Goto

Jump to labeled positions:

```lua
local i: number = 0

::start::
print(i)
i = i + 1

if i < 5 then
    goto start
end

print("Done")
```

**Restrictions:**

- Cannot jump into a block from outside
- Cannot jump out of a function
- Labels must be defined in the same scope

## Do Blocks

Create scoped blocks:

```lua
do
    const x: number = 10
    print(x)
end

-- x is not in scope here
```

Useful for limiting variable scope:

```lua
local result: number = 0

do
    const temp: number = calculate()
    const adjusted: number = temp * 2
    result = adjusted
end

-- temp and adjusted are not in scope
print(result)
```

## Ternary Operator

Concise conditional expressions:

```lua
const age: number = 20
const status: string = age >= 18 ? "adult" : "minor"
```

Compiles to:

```lua
local age = 20
local status = (age >= 18) and "adult" or "minor"
```

**Note:** Beware of falsy values. Use full `if` statement when middle value could be `false` or `nil`.

## Short-Circuit Evaluation

### And (`and`)

Returns first falsy value or last value:

```lua
const a: boolean = true
const b: boolean = false

const result = a and b  -- false
```

### Or (`or`)

Returns first truthy value or last value:

```lua
const name: string | nil = getName()
const displayName: string = name or "Guest"
```

## Null Coalescing Operator

Returns right-hand side if left is `nil` (unlike `or`, which checks for all falsy values):

```lua
const value: number | nil = 0
const result: number = value ?? 10  -- 0 (not 10, because 0 is not nil)

const none: number | nil = nil
const fallback: number = none ?? 10  -- 10
```

Compiles to:

```lua
local value = 0
local result = (value ~= nil) and value or 10

local none = nil
local fallback = (none ~= nil) and none or 10
```

## Optional Chaining

Safely access nested properties that might be nil:

```lua
interface User {
    profile?: {
        address?: {
            city?: string
        }
    }
}

const user: User = getUser()

-- Safe access with optional chaining
const city: string | nil = user.profile?.address?.city

-- Without optional chaining (verbose)
local city: string | nil = nil
if user.profile ~= nil and user.profile.address ~= nil then
    city = user.profile.address.city
end
```

Method calls:

```lua
const result: any = obj?.method?.(arg1, arg2)
```

## Pattern Matching

Match expressions for complex conditionals (see [Pattern Matching](pattern-matching.md) for details):

```lua
type Result = {ok: true, value: number} | {ok: false, error: string}

const result: Result = processData()

match result {
    {ok: true, value} => print("Success:", value),
    {ok: false, error} => print("Error:", error)
}
```

## Type Guards

Functions that narrow types:

```lua
function isString(value: unknown): value is string
    return type(value) == "string"
end

const data: unknown = getData()

if isString(data) then
    -- data is string here
    print(data:upper())
end
```

## Exhaustiveness Checking

Ensure all cases are handled:

```lua
type Status = "active" | "inactive" | "pending"

function handleStatus(status: Status): void
    if status == "active" then
        print("Active")
    elseif status == "inactive" then
        print("Inactive")
    elseif status == "pending" then
        print("Pending")
    else
        -- Exhaustive check: status is never here
        const _exhaustive: never = status
    end
end
```

If you add a new status value and forget to handle it, the compiler will catch the error.

## Next Steps

- [Functions](functions.md) — Function declarations and types
- [Pattern Matching](pattern-matching.md) — Advanced pattern matching
- [Type System](type-system.md) — Type narrowing and guards

## See Also

- [Operators](operators.md) — Complete operator reference
- [Basics](basics.md) — Variable declarations and types
