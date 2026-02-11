---
title: Advanced Types
---

# Advanced Types

LuaNext provides advanced type system features inspired by TypeScript, including conditional types, mapped types, template literal types, type predicates, and more.

## Conditional Types

Conditional types select one of two types based on a condition.

### Syntax

```lua
T extends U ? X : Y
```

If `T` extends `U`, the type resolves to `X`, otherwise `Y`.

### Examples

#### Basic Conditional Type

```lua
type IsString<T> = T extends string ? true : false

type A = IsString<string>  -- true
type B = IsString<number>  -- false
```

#### Extracting Return Type

```lua
type ReturnType<T> = T extends (...args: any) => infer R ? R : never

type Func = (x: number) => string
type Result = ReturnType<Func>  -- string
```

#### Distributive Conditional Types

Conditional types distribute over unions:

```lua
type ToArray<T> = T extends any ? T[] : never

type Result = ToArray<string | number>
-- Expands to: (string extends any ? string[] : never) | (number extends any ? number[] : never)
-- Result: string[] | number[]
```

#### Non-Nullable Type

```lua
type NonNilable<T> = T extends nil ? never : T

type A = NonNilable<string | nil>  -- string
type B = NonNilable<number | nil>  -- number
```

## Mapped Types

Mapped types transform properties of an existing type.

### Syntax

```lua
{[K in keyof T]: NewType}
{[K in keyof T]?: NewType}      -- Optional
{readonly [K in keyof T]: NewType}  -- Readonly
```

### Examples

#### Basic Mapped Type

```lua
type Partial<T> = {[K in keyof T]?: T[K]}

interface User
    name: string
    age: number
    email: string
end

type PartialUser = Partial<User>
-- Result:
-- {
--     name?: string,
--     age?: number,
--     email?: string
-- }
```

#### Readonly Mapped Type

```lua
type Readonly<T> = {readonly [K in keyof T]: T[K]}

type ReadonlyUser = Readonly<User>
-- All properties become readonly
```

#### Required Mapped Type

```lua
type Required<T> = {[K in keyof T]-?: T[K]}

interface PartialConfig
    host?: string
    port?: number
end

type Config = Required<PartialConfig>
-- Both properties become required
```

#### Adding and Removing Modifiers

```lua
-- Add readonly
type AddReadonly<T> = {+readonly [K in keyof T]: T[K]}

-- Remove readonly
type RemoveReadonly<T> = {-readonly [K in keyof T]: T[K]}

-- Add optional
type AddOptional<T> = {[K in keyof T]+?: T[K]}

-- Remove optional (make required)
type RemoveOptional<T> = {[K in keyof T]-?: T[K]}
```

#### Record Type

```lua
type Record<K extends string | number, T> = {[P in K]: T}

type StringMap = Record<string, number>
-- {[key: string]: number}

type Status = "pending" | "active" | "completed"
type StatusRecord = Record<Status, boolean>
-- {
--     pending: boolean,
--     active: boolean,
--     completed: boolean
-- }
```

## Template Literal Types

Create types from template string patterns.

### Syntax

```lua
type Pattern = `prefix_${T}_suffix`
```

### Examples

#### Basic Template Literal Type

```lua
type EventName<T extends string> = `on${T}Changed`

type NameEvent = EventName<"name">  -- "onNameChanged"
type AgeEvent = EventName<"age">    -- "onAgeChanged"
```

#### Combining with Unions

```lua
type HTTPVerb = "GET" | "POST" | "PUT" | "DELETE"
type Endpoint = `/${HTTPVerb}`

-- Result: "/GET" | "/POST" | "/PUT" | "/DELETE"
```

#### Multiple Interpolations

```lua
type EventHandler<T extends string, U extends string> = `on${T}${U}`

type Handler = EventHandler<"User", "Created">  -- "onUserCreated"
```

## Type Queries (typeof)

Extract the type of a value.

### Syntax

```lua
typeof expression
```

### Examples

```lua
const config = {
    host = "localhost",
    port = 8080,
    ssl = true
}

type Config = typeof config
-- Result:
-- {
--     host: string,
--     port: number,
--     ssl: boolean
-- }
```

#### With Functions

```lua
function add(a: number, b: number): number
    return a + b
end

type AddType = typeof add
-- (a: number, b: number) => number
```

## KeyOf Type Operator

Get union of all property keys.

### Syntax

```lua
keyof T
```

### Examples

```lua
interface Point
    x: number
    y: number
    z: number
end

type PointKeys = keyof Point
-- "x" | "y" | "z"
```

#### With Index Signatures

```lua
interface StringMap
    [key: string]: any
end

type Keys = keyof StringMap
-- string
```

## Indexed Access Types

Access the type of a property.

### Syntax

```lua
T[K]
```

### Examples

```lua
interface User
    name: string
    age: number
    email: string
end

type NameType = User["name"]    -- string
type AgeType = User["age"]      -- number
```

#### With keyof

```lua
type PropertyType = User[keyof User]
-- string | number (union of all property types)
```

## Infer Keyword

Extract types within conditional types.

### Syntax

```lua
T extends Pattern<infer U> ? U : never
```

### Examples

#### Inferring Return Type

```lua
type ReturnType<T> = T extends (...args: any) => infer R ? R : never

type Func1 = () => string
type Func2 = () => number

type R1 = ReturnType<Func1>  -- string
type R2 = ReturnType<Func2>  -- number
```

#### Inferring Array Element Type

```lua
type ElementType<T> = T extends (infer U)[] ? U : never

type A = ElementType<string[]>  -- string
type B = ElementType<number[]>  -- number
```

#### Inferring Function Parameters

```lua
type Parameters<T> = T extends (...args: infer P) => any ? P : never

type Func = (a: string, b: number) => void
type Params = Parameters<Func>  -- [string, number]
```

## Type Predicates

Type guards with predicates.

### Syntax

```lua
parameter is Type
```

### Examples

```lua
function isString(value: unknown): value is string
    return type(value) == "string"
end

function process(value: string | number): void
    if isString(value) then
        -- value is string here
        print(value:upper())
    else
        -- value is number here
        print(value * 2)
    end
end
```

#### Custom Type Guards

```lua
interface Dog
    bark(): void
end

interface Cat
    meow(): void
end

function isDog(animal: Dog | Cat): animal is Dog
    return (animal as any).bark ~= nil
end

function handleAnimal(animal: Dog | Cat): void
    if isDog(animal) then
        animal:bark()
    else
        animal:meow()
    end
end
```

## Variadic Types

Handle variable-length return types.

### Syntax

```lua
...T[]
```

### Examples

```lua
function tuple<T...>(...args: T...): T...
    return ...
end

const [a, b, c] = tuple(1, "hello", true)
-- a: number, b: string, c: boolean
```

## Utility Types

LuaNext provides built-in utility types:

### Partial<T>

Make all properties optional:

```lua
type Partial<T> = {[K in keyof T]?: T[K]}
```

### Required<T>

Make all properties required:

```lua
type Required<T> = {[K in keyof T]-?: T[K]}
```

### Readonly<T>

Make all properties readonly:

```lua
type Readonly<T> = {readonly [K in keyof T]: T[K]}
```

### Record<K, T>

Create object type with keys K and values T:

```lua
type Record<K extends string | number, T> = {[P in K]: T}
```

### Pick<T, K>

Pick subset of properties:

```lua
type Pick<T, K extends keyof T> = {[P in K]: T[P]}

interface User
    id: string
    name: string
    email: string
    age: number
end

type UserPreview = Pick<User, "id" | "name">
-- {id: string, name: string}
```

### Omit<T, K>

Omit properties:

```lua
type Omit<T, K extends keyof T> = Pick<T, Exclude<keyof T, K>>

type UserWithoutEmail = Omit<User, "email">
-- {id: string, name: string, age: number}
```

### Exclude<T, U>

Exclude types from union:

```lua
type Exclude<T, U> = T extends U ? never : T

type A = Exclude<"a" | "b" | "c", "a">  -- "b" | "c"
```

### Extract<T, U>

Extract types from union:

```lua
type Extract<T, U> = T extends U ? T : never

type A = Extract<"a" | "b" | "c", "a" | "d">  -- "a"
```

### NonNilable<T>

Remove nil from type:

```lua
type NonNilable<T> = T extends nil ? never : T

type A = NonNilable<string | nil>  -- string
```

### Nilable<T>

Add nil to type:

```lua
type Nilable<T> = T | nil
```

### ReturnType<T>

Extract function return type:

```lua
type ReturnType<T> = T extends (...args: any) => infer R ? R : never
```

### Parameters<T>

Extract function parameter types:

```lua
type Parameters<T> = T extends (...args: infer P) => any ? P : never
```

## Complex Type Transformations

### Flatten Object Type

```lua
type Flatten<T> = T extends object ? {[K in keyof T]: T[K]} : T
```

### Deep Partial

```lua
type DeepPartial<T> = {
    [K in keyof T]?: T[K] extends object ? DeepPartial<T[K]> : T[K]
}
```

### Deep Readonly

```lua
type DeepReadonly<T> = {
    readonly [K in keyof T]: T[K] extends object ? DeepReadonly<T[K]> : T[K]
}
```

### Mutable (Remove Readonly)

```lua
type Mutable<T> = {
    -readonly [K in keyof T]: T[K]
}
```

## Type Constraints

Constrain type parameters:

```lua
type Lengthwise = {length: number}

function logLength<T extends Lengthwise>(arg: T): void
    print(arg.length)
end

logLength({length = 10})      -- ✅ OK
logLength([1, 2, 3])          -- ✅ OK (arrays have length)
-- logLength(42)              -- ❌ Error: number doesn't have length
```

## Details

### Conditional Type Inference

When using `infer`, the inferred type is captured and can be used in the true branch:

```lua
type UnpackArray<T> = T extends (infer U)[] ? U : T

type A = UnpackArray<string[]>  -- string
type B = UnpackArray<number>    -- number (fallback)
```

### Mapped Type Modifiers

- `+?` or `?` — Make optional
- `-?` — Make required
- `+readonly` or `readonly` — Make readonly
- `-readonly` — Remove readonly

### Template Literal Type Inference

Template literal types support inference:

```lua
type ExtractVersion<T> = T extends `v${infer N}` ? N : never

type A = ExtractVersion<"v1.0.0">  -- "1.0.0"
```

### Distributive Conditional Types

Conditional types distribute over unions automatically:

```lua
type ToArray<T> = T extends any ? T[] : never

type Result = ToArray<string | number>
-- Distributes to: ToArray<string> | ToArray<number>
-- Result: string[] | number[]
```

Wrap in `[]` to prevent distribution:

```lua
type ToArray<T> = [T] extends [any] ? T[] : never

type Result = ToArray<string | number>
-- Result: (string | number)[]
```

## See Also

- [Type System](type-system.md) — Basic type system features
- [Interfaces](interfaces.md) — Object type definitions
- [Generics](type-system.md#generics) — Generic types
- [Utility Types](../reference/utility-types.md) — Complete utility type reference
