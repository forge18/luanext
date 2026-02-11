---
title: Utility Types Reference
---

# Utility Types Reference

Complete reference for LuaNext's built-in utility types. These types transform and manipulate other types.

## Overview

LuaNext provides 12 built-in utility types inspired by TypeScript:

| Utility Type | Purpose |
|--------------|---------|
| `Partial<T>` | Make all properties optional |
| `Required<T>` | Make all properties required |
| `Readonly<T>` | Make all properties readonly |
| `Record<K, T>` | Create object type with keys K and values T |
| `Pick<T, K>` | Select subset of properties |
| `Omit<T, K>` | Remove properties |
| `Exclude<T, U>` | Exclude types from union |
| `Extract<T, U>` | Extract types from union |
| `NonNilable<T>` | Remove nil from type |
| `Nilable<T>` | Add nil to type |
| `ReturnType<T>` | Extract function return type |
| `Parameters<T>` | Extract function parameter types |

## Partial<T>

Makes all properties of `T` optional.

### Definition

```lua
type Partial<T> = {[K in keyof T]?: T[K]}
```

### Usage

```lua
interface User
    id: string
    name: string
    email: string
    age: number
end

type PartialUser = Partial<User>
-- Result:
-- {
--     id?: string,
--     name?: string,
--     email?: string,
--     age?: number
-- }

function updateUser(id: string, updates: PartialUser): void
    -- All fields are optional
    if updates.name ~= nil then
        setUserName(id, updates.name)
    end
    if updates.email ~= nil then
        setUserEmail(id, updates.email)
    end
end

updateUser("123", {name = "Alice"})  -- Only update name
updateUser("456", {email = "bob@example.com", age = 30})  -- Update multiple
```

### Common Patterns

```lua
-- Partial update functions
function patch<T>(original: T, updates: Partial<T>): T
    return {...original, ...updates}
end

-- Optional configuration
interface Config
    host: string
    port: number
    ssl: boolean
end

const defaultConfig: Config = {host = "localhost", port = 8080, ssl = false}

function createServer(options?: Partial<Config>): Server
    const config = {...defaultConfig, ...options}
    return Server.new(config)
end
```

## Required<T>

Makes all properties of `T` required (removes optional modifiers).

### Definition

```lua
type Required<T> = {[K in keyof T]-?: T[K]}
```

### Usage

```lua
interface PartialConfig
    host?: string
    port?: number
    ssl?: boolean
end

type Config = Required<PartialConfig>
-- Result:
-- {
--     host: string,
--     port: number,
--     ssl: boolean
-- }

function startServer(config: Config): void
    -- All fields guaranteed to exist
    print(`Server on ${config.host}:${config.port}`)
end

const partialConfig: PartialConfig = {host = "localhost"}
-- startServer(partialConfig)  -- ❌ Error: port and ssl missing

const fullConfig: Config = {host = "localhost", port = 8080, ssl = true}
startServer(fullConfig)  -- ✅ OK
```

## Readonly<T>

Makes all properties of `T` readonly.

### Definition

```lua
type Readonly<T> = {readonly [K in keyof T]: T[K]}
```

### Usage

```lua
interface Point
    x: number
    y: number
end

type ReadonlyPoint = Readonly<Point>
-- Result:
-- {
--     readonly x: number,
--     readonly y: number
-- }

const p: ReadonlyPoint = {x = 10, y = 20}
-- p.x = 15  -- ❌ Error: Cannot assign to readonly property

function movePoint(p: Point, dx: number, dy: number): ReadonlyPoint
    return {x = p.x + dx, y = p.y + dy}
end
```

## Record<K, T>

Creates an object type with keys `K` and values `T`.

### Definition

```lua
type Record<K extends string | number, T> = {[P in K]: T}
```

### Usage

```lua
-- String keys
type StringMap = Record<string, number>
-- {[key: string]: number}

const counts: StringMap = {
    apples = 5,
    oranges = 3,
    bananas = 7
}

-- Literal union keys
type Status = "pending" | "active" | "completed"
type StatusRecord = Record<Status, boolean>
-- {
--     pending: boolean,
--     active: boolean,
--     completed: boolean
-- }

const statusFlags: StatusRecord = {
    pending = true,
    active = false,
    completed = false
}

-- Number keys
type Cache = Record<number, string>
const cache: Cache = {
    [1] = "one",
    [2] = "two",
    [3] = "three"
}
```

## Pick<T, K>

Selects a subset of properties from `T`.

### Definition

```lua
type Pick<T, K extends keyof T> = {[P in K]: T[P]}
```

### Usage

```lua
interface User
    id: string
    name: string
    email: string
    age: number
    password: string
end

-- Pick only public fields
type UserPreview = Pick<User, "id" | "name" | "email">
-- Result:
-- {
--     id: string,
--     name: string,
--     email: string
-- }

function getPublicProfile(user: User): UserPreview
    return {
        id = user.id,
        name = user.name,
        email = user.email
    }
end

-- Pick single field
type UserId = Pick<User, "id">  -- {id: string}
```

## Omit<T, K>

Removes properties from `T`.

### Definition

```lua
type Omit<T, K extends keyof T> = Pick<T, Exclude<keyof T, K>>
```

### Usage

```lua
interface User
    id: string
    name: string
    email: string
    password: string
end

-- Omit sensitive fields
type PublicUser = Omit<User, "password">
-- Result:
-- {
--     id: string,
--     name: string,
--     email: string
-- }

function sanitizeUser(user: User): PublicUser
    return {
        id = user.id,
        name = user.name,
        email = user.email
    }
end

-- Omit multiple fields
type UserWithoutMeta = Omit<User, "id" | "password">
-- {name: string, email: string}
```

## Exclude<T, U>

Excludes types from a union.

### Definition

```lua
type Exclude<T, U> = T extends U ? never : T
```

### Usage

```lua
type All = "a" | "b" | "c" | "d"
type ExcludeAB = Exclude<All, "a" | "b">
-- Result: "c" | "d"

type Primitive = string | number | boolean | nil
type NonNilPrimitive = Exclude<Primitive, nil>
-- Result: string | number | boolean

type Status = "pending" | "active" | "inactive" | "deleted"
type ActiveStatuses = Exclude<Status, "deleted">
-- Result: "pending" | "active" | "inactive"
```

## Extract<T, U>

Extracts types from a union that are assignable to `U`.

### Definition

```lua
type Extract<T, U> = T extends U ? T : never
```

### Usage

```lua
type All = "a" | "b" | "c" | "d"
type ExtractAB = Extract<All, "a" | "b" | "e">
-- Result: "a" | "b" (only types present in both)

type Mixed = string | number | boolean
type OnlyStrings = Extract<Mixed, string>
-- Result: string

type Status = "pending" | "active" | "completed" | "failed"
type CompletedStatuses = Extract<Status, "completed" | "failed">
-- Result: "completed" | "failed"
```

## NonNilable<T>

Removes `nil` from a type.

### Definition

```lua
type NonNilable<T> = T extends nil ? never : T
```

### Usage

```lua
type MaybeString = string | nil
type DefinitelyString = NonNilable<MaybeString>
-- Result: string

type Optional = number | string | nil
type Required = NonNilable<Optional>
-- Result: number | string

function requireValue<T>(value: T | nil): NonNilable<T>
    if value == nil then
        error("Value cannot be nil")
    end
    return value
end
```

## Nilable<T>

Adds `nil` to a type.

### Definition

```lua
type Nilable<T> = T | nil
```

### Usage

```lua
type DefinitelyString = string
type MaybeString = Nilable<DefinitelyString>
-- Result: string | nil

function findUser(id: string): Nilable<User>
    return users[id]  -- May return nil
end

const user: Nilable<User> = findUser("123")
if user ~= nil then
    print(user.name)
end
```

## ReturnType<T>

Extracts the return type of a function.

### Definition

```lua
type ReturnType<T> = T extends (...args: any) => infer R ? R : never
```

### Usage

```lua
function getUser(): {name: string, age: number}
    return {name = "Alice", age = 30}
end

type User = ReturnType<typeof getUser>
-- Result: {name: string, age: number}

function calculate(a: number, b: number): number
    return a + b
end

type CalculateResult = ReturnType<typeof calculate>
-- Result: number

-- With type alias
type FetchData = () => {id: string, value: number}
type Data = ReturnType<FetchData>
-- Result: {id: string, value: number}
```

## Parameters<T>

Extracts the parameter types of a function as a tuple.

### Definition

```lua
type Parameters<T> = T extends (...args: infer P) => any ? P : never
```

### Usage

```lua
function add(a: number, b: number): number
    return a + b
end

type AddParams = Parameters<typeof add>
-- Result: [number, number]

function greet(name: string, greeting?: string): void
    print((greeting or "Hello") .. ", " .. name)
end

type GreetParams = Parameters<typeof greet>
-- Result: [string, string | nil]

-- Extract individual parameters
type FirstParam = AddParams[1]  -- number
type SecondParam = AddParams[2]  -- number
```

## Advanced Combinations

### Deep Partial

Make all nested properties optional:

```lua
type DeepPartial<T> = {
    [K in keyof T]?: T[K] extends object ? DeepPartial<T[K]> : T[K]
}

interface Config
    database: {
        host: string
        port: number
    }
    cache: {
        enabled: boolean
        ttl: number
    }
end

type PartialConfig = DeepPartial<Config>
-- All nested properties are optional
const config: PartialConfig = {
    database = {host = "localhost"}  -- port optional
}
```

### Deep Readonly

Make all nested properties readonly:

```lua
type DeepReadonly<T> = {
    readonly [K in keyof T]: T[K] extends object ? DeepReadonly<T[K]> : T[K]
}
```

### Mutable (Remove Readonly)

Remove readonly modifiers:

```lua
type Mutable<T> = {
    -readonly [K in keyof T]: T[K]
}

interface ReadonlyUser
    readonly id: string
    readonly name: string
end

type MutableUser = Mutable<ReadonlyUser>
-- {id: string, name: string}
```

### Nullable (Add nil to all properties)

```lua
type Nullable<T> = {
    [K in keyof T]: T[K] | nil
}

interface User
    name: string
    age: number
end

type NullableUser = Nullable<User>
-- {name: string | nil, age: number | nil}
```

## Practical Examples

### API Response Types

```lua
interface ApiResponse<T>
    data: T
    status: number
    error?: string
end

type UserResponse = ApiResponse<User>
type UsersResponse = ApiResponse<User[]>
type PartialUserResponse = ApiResponse<Partial<User>>
```

### Form State

```lua
interface FormState<T>
    values: T
    errors: Partial<Record<keyof T, string>>
    touched: Partial<Record<keyof T, boolean>>
end

interface LoginForm
    email: string
    password: string
end

type LoginFormState = FormState<LoginForm>
```

### Configuration Builder

```lua
interface FullConfig
    host: string
    port: number
    ssl: boolean
    timeout: number
end

const defaults: FullConfig = {
    host = "localhost",
    port = 8080,
    ssl = false,
    timeout = 5000
}

function buildConfig(options: Partial<FullConfig>): Required<FullConfig>
    return {...defaults, ...options}
end
```

## See Also

- [Advanced Types](../language/advanced-types.md) — Detailed explanations and examples
- [Type System](../language/type-system.md) — Core type system features
- [Generics](../language/type-system.md#generics) — Generic type parameters
