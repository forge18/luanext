---
title: Operators
---

# Operators

LuaNext provides a comprehensive set of operators for arithmetic, comparison, logical operations, bitwise operations, and special operators like optional chaining, null coalescing, and more.

## Arithmetic Operators

### Basic Arithmetic

| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `+` | Addition | `5 + 3` | `8` |
| `-` | Subtraction | `10 - 4` | `6` |
| `*` | Multiplication | `6 * 7` | `42` |
| `/` | Division | `15 / 3` | `5.0` |
| `%` | Modulo | `17 % 5` | `2` |
| `^` | Exponentiation | `2 ^ 10` | `1024` |
| `//` | Floor division | `17 // 5` | `3` |

```lua
const a: number = 10 + 5   -- 15
const b: number = 20 - 7   -- 13
const c: number = 6 * 4    -- 24
const d: number = 15 / 3   -- 5.0
const e: number = 17 % 5   -- 2
const f: number = 2 ^ 10   -- 1024
const g: number = 17 // 5  -- 3 (floor division)
```

### Unary Arithmetic

| Operator | Description | Example | Result |
|----------|-------------|---------|--------|
| `-` | Negation | `-5` | `-5` |
| `#` | Length | `#{1,2,3}` | `3` |

```lua
const x: number = 10
const negated: number = -x  -- -10

const arr: number[] = {1, 2, 3, 4, 5}
const length: number = #arr  -- 5

const str: string = "hello"
const strLen: number = #str  -- 5
```

## Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `5 == 5` |
| `~=` | Not equal | `5 ~= 3` |
| `<` | Less than | `3 < 5` |
| `<=` | Less than or equal | `5 <= 5` |
| `>` | Greater than | `7 > 3` |
| `>=` | Greater than or equal | `5 >= 5` |

```lua
const isEqual: boolean = 5 == 5          -- true
const notEqual: boolean = 5 ~= 3         -- true
const lessThan: boolean = 3 < 5          -- true
const lessOrEqual: boolean = 5 <= 5      -- true
const greaterThan: boolean = 7 > 3       -- true
const greaterOrEqual: boolean = 5 >= 5   -- true
```

## Logical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `and` | Logical AND | `true and false` |
| `or` | Logical OR | `true or false` |
| `not` | Logical NOT | `not true` |

```lua
const result1: boolean = true and false   -- false
const result2: boolean = true or false    -- true
const result3: boolean = not true         -- false

-- Short-circuit evaluation
const value = x > 0 and x or 0  -- Returns x if x > 0, else 0
```

## Bitwise Operators

Available in Lua 5.3+:

| Operator | Description | Example |
|----------|-------------|---------|
| `&` | Bitwise AND | `12 & 10` |
| `|` | Bitwise OR | `12 | 10` |
| `~` (unary) | Bitwise NOT | `~5` |
| `~` (binary) | Bitwise XOR | `12 ~ 10` |
| `<<` | Left shift | `1 << 3` |
| `>>` | Right shift | `8 >> 2` |

```lua
const a: integer = 12       -- 1100 in binary
const b: integer = 10       -- 1010 in binary

const bitwiseAnd: integer = a & b   -- 8 (1000)
const bitwiseOr: integer = a | b    -- 14 (1110)
const bitwiseXor: integer = a ~ b   -- 6 (0110)
const bitwiseNot: integer = ~a      -- -13
const leftShift: integer = 1 << 3   -- 8 (1 * 2^3)
const rightShift: integer = 8 >> 2  -- 2 (8 / 2^2)
```

## String Concatenation

| Operator | Description | Example |
|----------|-------------|---------|
| `..` | Concatenate | `"Hello" .. " World"` |

```lua
const greeting: string = "Hello" .. " " .. "World"  -- "Hello World"
const message: string = "Count: " .. tostring(42)    -- "Count: 42"
```

## Compound Assignment

| Operator | Equivalent | Description |
|----------|------------|-------------|
| `+=` | `x = x + y` | Add and assign |
| `-=` | `x = x - y` | Subtract and assign |
| `*=` | `x = x * y` | Multiply and assign |
| `/=` | `x = x / y` | Divide and assign |
| `%=` | `x = x % y` | Modulo and assign |
| `^=` | `x = x ^ y` | Exponentiate and assign |
| `//=` | `x = x // y` | Floor divide and assign |
| `..=` | `x = x .. y` | Concatenate and assign |
| `&=` | `x = x & y` | Bitwise AND and assign |
| `|=` | `x = x | y` | Bitwise OR and assign |
| `<<=` | `x = x << y` | Left shift and assign |
| `>>=` | `x = x >> y` | Right shift and assign |

```lua
local x: number = 10
x += 5   -- x = 15
x -= 3   -- x = 12
x *= 2   -- x = 24
x /= 4   -- x = 6.0
x %= 5   -- x = 1.0
x ^= 3   -- x = 1.0

local s: string = "Hello"
s ..= " World"  -- s = "Hello World"

local flags: integer = 0b1100
flags &= 0b1010  -- flags = 0b1000
```

## Optional Chaining

Safely access nested properties that might be nil:

| Operator | Description | Example |
|----------|-------------|---------|
| `?.` | Optional member access | `obj?.property` |
| `?[]` | Optional index access | `arr?[1]` |
| `?()` | Optional call | `func?()` |
| `?:` | Optional method call | `obj?:method()` |

```lua
interface User
    name: string
    address?: {
        street: string
        city: string
    }
end

const user: User | nil = getUser()

-- Without optional chaining (verbose)
const city1 = user ~= nil and user.address ~= nil and user.address.city or nil

-- With optional chaining (concise)
const city2 = user?.address?.city  -- nil if user or address is nil

-- Optional method call
const result = obj?:method()  -- Only calls if obj is not nil

-- Optional function call
const value = func?()  -- Only calls if func is not nil

-- Optional index
const item = array?[1]  -- nil if array is nil
```

## Null Coalescing

Provide default values for nil:

| Operator | Description | Example |
|----------|-------------|---------|
| `??` | Null coalesce | `value ?? default` |

```lua
const name: string = user?.name ?? "Anonymous"
const port: number = config?.port ?? 8080
const items: string[] = data?.items ?? {}

-- Chaining
const value = a ?? b ?? c ?? defaultValue
```

Difference from `or`:

```lua
-- or returns right side for any falsy value (nil, false)
const x = false or "default"  -- "default"

-- ?? only returns right side for nil
const y = false ?? "default"  -- false
```

## Ternary Operator

Conditional expression:

| Operator | Description | Example |
|----------|-------------|---------|
| `? :` | Ternary | `condition ? trueValue : falseValue` |

```lua
const max = a > b ? a : b
const status = isActive ? "Active" : "Inactive"
const message = count > 0 ? `${count} items` : "No items"

-- Nested ternary
const category = score >= 90 ? "A" :
                 score >= 80 ? "B" :
                 score >= 70 ? "C" : "F"
```

## Pipe Operator

Chain function calls:

| Operator | Description | Example |
|----------|-------------|---------|
| `|>` | Pipe | `value |> func` |

```lua
const result = data
    |> parseJSON
    |> validateData
    |> transform
    |> stringify

-- Equivalent to:
const result = stringify(transform(validateData(parseJSON(data))))

-- With arguments
const result = data
    |> trim
    |> split(_, ",")
    |> map(_, parseInt)
    |> filter(_, isPositive)
```

## Error Chain Operator

Shorthand for try-catch:

| Operator | Description | Example |
|----------|-------------|---------|
| `!!` | Error chain | `operation() !! fallback` |

```lua
const data = fetchData() !! defaultData
const config = loadConfig() !! getDefaultConfig()

-- Equivalent to:
const data = try fetchData() catch _ => defaultData
```

## Instanceof Operator

Check instance types:

| Operator | Description | Example |
|----------|-------------|---------|
| `instanceof` | Type check | `obj instanceof ClassName` |

```lua
if user instanceof AdminUser then
    print("User is an admin")
end

function handleAnimal(animal: Animal): void
    if animal instanceof Dog then
        animal:bark()
    elseif animal instanceof Cat then
        animal:meow()
    end
end
```

## Type Assertion

Cast types:

| Operator | Description | Example |
|----------|-------------|---------|
| `as` | Type assertion | `value as Type` |

```lua
const data: unknown = getData()
const user = data as User

const num = someValue as number
const str = someValue as string

-- Use with caution—bypasses type checking
```

## Spread and Rest

### Spread Operator

Expand arrays or objects:

```lua
-- Array spread
const arr1 = {1, 2, 3}
const arr2 = {4, 5, 6}
const combined = {...arr1, ...arr2}  -- {1, 2, 3, 4, 5, 6}

-- Object spread
const obj1 = {x = 1, y = 2}
const obj2 = {y = 3, z = 4}
const merged = {...obj1, ...obj2}  -- {x = 1, y = 3, z = 4}
```

### Rest Operator

Collect remaining elements:

```lua
-- Function parameters
function sum(...numbers: number[]): number
    local total = 0
    for _, n in ipairs(numbers) do
        total += n
    end
    return total
end

print(sum(1, 2, 3, 4, 5))  -- 15

-- Array destructuring
const [first, ...rest] = {1, 2, 3, 4, 5}
-- first = 1, rest = {2, 3, 4, 5}

-- Object destructuring
const {x, ...remaining} = {x = 1, y = 2, z = 3}
-- x = 1, remaining = {y = 2, z = 3}
```

## Precedence

Operators listed from highest to lowest precedence:

1. **Member access**: `.`, `[]`, `:`
2. **Function call**: `()`
3. **Unary**: `not`, `-`, `#`, `~`
4. **Exponentiation**: `^`
5. **Multiplicative**: `*`, `/`, `%`, `//`
6. **Additive**: `+`, `-`
7. **Concatenation**: `..`
8. **Bitwise shift**: `<<`, `>>`
9. **Bitwise AND**: `&`
10. **Bitwise XOR**: `~`
11. **Bitwise OR**: `|`
12. **Comparison**: `<`, `<=`, `>`, `>=`, `==`, `~=`, `instanceof`
13. **Logical AND**: `and`
14. **Logical OR**: `or`
15. **Null coalesce**: `??`
16. **Ternary**: `? :`
17. **Pipe**: `|>`
18. **Error chain**: `!!`
19. **Assignment**: `=`, `+=`, `-=`, etc.

Use parentheses for clarity:

```lua
const result = (a + b) * c     -- Explicit grouping
const value = x > 0 and y or z  -- Can be ambiguous, use parens
const value = (x > 0) and y or z  -- Clear precedence
```

## Operator Overloading

Classes can overload operators:

```lua
class Vector(public x: number, public y: number)
    operator +(other: Vector): Vector
        return Vector.new(self.x + other.x, self.y + other.y)
    end

    operator -(other: Vector): Vector
        return Vector.new(self.x - other.x, self.y - other.y)
    end

    operator *(scalar: number): Vector
        return Vector.new(self.x * scalar, self.y * scalar)
    end

    operator ==(other: Vector): boolean
        return self.x == other.x and self.y == other.y
    end

    operator #(): number
        return math.sqrt(self.x * self.x + self.y * self.y)
    end
end

const v1 = Vector.new(1, 2)
const v2 = Vector.new(3, 4)
const v3 = v1 + v2          -- Vector(4, 6)
const v4 = v1 * 2           -- Vector(2, 4)
const equal = v1 == v2      -- false
const length = #v1          -- 2.236...
```

## See Also

- [Classes](classes.md) — Operator overloading
- [Error Handling](error-handling.md) — Error chain operator
- [Type System](type-system.md) — Type assertions and instanceof
- [Advanced Types](advanced-types.md) — Type operators
