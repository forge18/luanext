# Type System

LuaNext provides a rich type system inspired by TypeScript, with unions, intersections, generics, and advanced type features.

## Type Aliases

Create named types with `type`:

```lua
type UserId = number
type Status = "active" | "inactive" | "pending"
type Point = {x: number, y: number}  -- Error: use interface for table shapes

const id: UserId = 123
const status: Status = "active"
```

**Important:** Type aliases cannot define table shapes. Use `interface` instead.

## Union Types

A value can be one of several types:

```lua
type Result = number | string | nil

const value: Result = 42        -- OK
const text: Result = "hello"    -- OK
const none: Result = nil        -- OK
```

### Type Narrowing

Use conditionals to narrow union types:

```lua
function process(value: string | number): void
    if type(value) == "string" then
        -- value is string here
        print(value:upper())
    else
        -- value is number here
        print(value * 2)
    end
end
```

Common type guards:

- `type(x) == "string"` — Narrows to string
- `type(x) == "number"` — Narrows to number
- `type(x) == "table"` — Narrows to table
- `x ~= nil` — Removes nil from union

### Discriminated Unions

Use a common field to discriminate:

```lua
type Shape =
    | {kind: "circle", radius: number}
    | {kind: "rectangle", width: number, height: number}

function area(shape: Shape): number
    if shape.kind == "circle" then
        return math.pi * shape.radius * shape.radius
    else
        return shape.width * shape.height
    end
end
```

## Intersection Types

Combine multiple types:

```lua
interface Named {
    name: string
}

interface Aged {
    age: number
}

type Person = Named & Aged

const person: Person = {
    name = "Alice",
    age = 30
}
```

**Note:** Intersections work for combining interfaces. Type aliases cannot use `&` directly.

## Literal Types

Values as types:

```lua
type Direction = "north" | "south" | "east" | "west"
type DiceRoll = 1 | 2 | 3 | 4 | 5 | 6
type Enabled = true

const dir: Direction = "north"  -- OK
const roll: DiceRoll = 3        -- OK
const flag: Enabled = true      -- OK
```

## Nullable Types

Explicitly allow nil:

```lua
type MaybeString = string | nil

const value: MaybeString = "hello"
const none: MaybeString = nil
```

Shorthand (if available):

```lua
type MaybeString = string?
```

## Generics

Type-safe parameterized types:

```lua
type Box<T> = {
    value: T,
    get: () -> T,
    set: (value: T) -> void
}

const numberBox: Box<number> = {
    value = 42,
    get = function() return 42 end,
    set = function(v) end
}
```

### Generic Constraints

Restrict type parameters:

```lua
interface HasLength {
    length: number
}

type LengthGetter<T extends HasLength> = (value: T) -> number

const getLength: LengthGetter<string> = (s) => s.length
```

### Multiple Type Parameters

```lua
type Pair<K, V> = {
    key: K,
    value: V
}

const entry: Pair<string, number> = {
    key = "count",
    value = 10
}
```

### Generic Defaults

```lua
type Result<T = string, E = Error> =
    | {ok: true, value: T}
    | {ok: false, error: E}

const success: Result = {ok = true, value = "done"}  -- Uses defaults
const failure: Result<number> = {ok = false, error = {message = "failed"}}
```

## Type Inference

LuaNext infers types in many contexts:

```lua
-- Variable initialization
const x = 42               -- number
const name = "Alice"       -- string
const items = {1, 2, 3}    -- number[]

-- Function returns
function double(x: number)
    return x * 2  -- Return type inferred as number
end

-- Generic instantiation
function identity<T>(x: T): T
    return x
end

const num = identity(42)  -- T inferred as number
```

## Type Assertions

Override type inference with `as`:

```lua
const data: unknown = getData()
const user = data as {name: string, age: number}

print(user.name)  -- OK (assumes data is the right shape)
```

**Warning:** Type assertions bypass type checking. Use with caution.

## Type Queries

Get the type of a value with `typeof`:

```lua
const config = {
    host = "localhost",
    port = 8080
}

type Config = typeof config  -- {host: string, port: number}
```

## Index Access Types

Extract property types:

```lua
interface User {
    id: number,
    name: string,
    email: string
}

type UserId = User["id"]        -- number
type UserField = User["name" | "email"]  -- string
```

## Keyof Types

Get all keys as a union:

```lua
interface User {
    id: number,
    name: string,
    email: string
}

type UserKey = keyof User  -- "id" | "name" | "email"
```

## Conditional Types

Types that depend on conditions:

```lua
type IsString<T> = T extends string ? true : false

type A = IsString<string>  -- true
type B = IsString<number>  -- false
```

More practical example:

```lua
type Awaited<T> = T extends Promise<infer U> ? U : T

type Value = Awaited<Promise<number>>  -- number
```

## Mapped Types

Transform object types:

```lua
type Readonly<T> = {
    readonly [K in keyof T]: T[K]
}

interface User {
    name: string,
    age: number
}

type ReadonlyUser = Readonly<User>
-- {readonly name: string, readonly age: number}
```

Add or remove modifiers:

```lua
type Mutable<T> = {
    -readonly [K in keyof T]: T[K]
}

type Optional<T> = {
    [K in keyof T]?: T[K]
}

type Required<T> = {
    [K in keyof T]-?: T[K]
}
```

## Template Literal Types

String types with interpolation:

```lua
type EventName = "click" | "focus" | "blur"
type Handler = `on${Capitalize<EventName>}`  -- "onClick" | "onFocus" | "onBlur"
```

## Type Predicates

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

## Variadic Types

Rest elements in tuples:

```lua
type Nums = [number, ...number[]]  -- At least one number

const valid: Nums = {1, 2, 3, 4}   -- OK
const invalid: Nums = {}           -- Error: needs at least one
```

## Infer Keyword

Extract types in conditional types:

```lua
type ReturnType<T> = T extends (...args: any[]) -> infer R ? R : never

function getUser(): {name: string, age: number}
    return {name = "Alice", age = 30}
end

type User = ReturnType<typeof getUser>  -- {name: string, age: number}
```

## Never Type

Type for impossible values:

```lua
type Result = "success" | "error"

function handle(result: Result): void
    if result == "success" then
        print("OK")
    elseif result == "error" then
        print("Failed")
    else
        -- result is never here
        const _exhaustive: never = result
    end
end
```

## Unknown Type

Type-safe alternative to `any`:

```lua
const data: unknown = getExternalData()

-- Error: Cannot use unknown directly
print(data.name)  -- ❌

-- OK: Narrow the type first
if type(data) == "table" then
    print(data.name)  -- ✅
end
```

## Type Compatibility

LuaNext uses structural typing:

```lua
interface Point {
    x: number,
    y: number
}

interface Vector {
    x: number,
    y: number
}

const p: Point = {x = 1, y = 2}
const v: Vector = p  -- OK: shapes match
```

Excess properties are allowed in assignments:

```lua
interface User {
    name: string
}

const user: User = {
    name = "Alice",
    age = 30  -- OK: excess property ignored
}
```

## Type Narrowing Patterns

### Truthiness

```lua
function process(value: string | nil): void
    if value then
        -- value is string here
        print(value:upper())
    end
end
```

### Equality

```lua
function compare(x: string | number, y: string | number): void
    if x == y then
        -- Both have the same type here (string | number, narrowed to intersection)
    end
end
```

### `instanceof`

```lua
class Animal {}
class Dog extends Animal {}

function handle(animal: Animal): void
    if animal instanceof Dog then
        -- animal is Dog here
    end
end
```

## Next Steps

- [Interfaces](interfaces.md) — Define table shapes
- [Advanced Types](advanced-types.md) — Conditional, mapped, template literal types
- [Utility Types](../reference/utility-types.md) — Built-in type helpers

## See Also

- [Basics](basics.md) — Primitive types
- [Functions](functions.md) — Generic functions
- [Classes](classes.md) — Generic classes
