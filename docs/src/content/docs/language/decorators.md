---
title: Decorators
---

# Decorators

Decorators provide a way to annotate and modify classes and their members. They enable meta-programming patterns like logging, validation, dependency injection, and more.

## Syntax

```lua
@decoratorName
@decoratorWithArgs(arg1, arg2)

-- On classes
@decorator
class ClassName
    -- ...
end

-- On class members
class ClassName
    @decorator
    property: Type

    @decorator
    function method(): void
        -- ...
    end

    @decorator
    get property(): Type
        -- ...
    end

    @decorator
    set property(value: Type)
        -- ...
    end

    @decorator
    operator +(other: Type): Type
        -- ...
    end
end
```

## Examples

### Basic Decorator

Simple decorator without arguments:

```lua
function readonly(target: any, key: string): void
    -- Mark property as readonly
    print(`Marking ${key} as readonly on ${target}`)
end

class User
    @readonly
    id: string

    name: string

    constructor(id: string, name: string)
        self.id = id
        self.name = name
    end
end

const user = User.new("user-123", "Alice")
user.name = "Bob"  -- ✅ OK
-- user.id = "new-id"  -- ❌ Error: Cannot assign to readonly property
```

### Decorator with Arguments

Decorators can accept arguments:

```lua
function deprecated(message: string): (target: any, key: string) => void
    return function(target, key)
        print(`Warning: ${key} is deprecated: ${message}`)
    end
end

class OldAPI
    @deprecated("Use newMethod() instead")
    function oldMethod(): void
        print("Old method")
    end

    function newMethod(): void
        print("New method")
    end
end

const api = OldAPI.new()
api:oldMethod()  -- Warning: oldMethod is deprecated: Use newMethod() instead
```

### Class Decorator

Decorate entire classes:

```lua
function sealed(constructor: any): void
    -- Prevent class extension
    constructor.__sealed = true
end

@sealed
class FinalClass
    value: number

    constructor(value: number)
        self.value = value
    end
end

-- Error: Cannot extend sealed class
-- class Extended extends FinalClass
-- end
```

### Method Decorator

Add behavior to methods:

```lua
function log(target: any, key: string, descriptor: any): void
    const originalMethod = descriptor.value

    descriptor.value = function(...)
        print(`Calling ${key} with arguments:`, ...)
        const result = originalMethod(...)
        print(`${key} returned:`, result)
        return result
    end
end

class Calculator
    @log
    function add(a: number, b: number): number
        return a + b
    end

    @log
    function multiply(a: number, b: number): number
        return a * b
    end
end

const calc = Calculator.new()
calc:add(5, 3)       -- Logs: Calling add with arguments: 5, 3
                      --       add returned: 8
calc:multiply(4, 2)  -- Logs: Calling multiply with arguments: 4, 2
                      --       multiply returned: 8
```

### Property Decorator

Modify property behavior:

```lua
function validate(validator: (value: any) => boolean): (target: any, key: string) => void
    return function(target, key)
        const privateKey = "_" .. key

        -- Create getter
        target[`get_${key}`] = function(self)
            return self[privateKey]
        end

        -- Create setter with validation
        target[`set_${key}`] = function(self, value)
            if not validator(value) then
                error(`Invalid value for ${key}: ${value}`)
            end
            self[privateKey] = value
        end
    end
end

class User
    @validate(function(value) return #value >= 3 end)
    name: string

    @validate(function(value) return value >= 18 end)
    age: number

    constructor(name: string, age: number)
        self.name = name
        self.age = age
    end
end

const user = User.new("Alice", 30)
-- user.name = "AB"  -- ❌ Error: Invalid value for name: AB
-- user.age = 10     -- ❌ Error: Invalid value for age: 10
```

### Multiple Decorators

Apply multiple decorators (executed bottom-to-top):

```lua
@decorator1
@decorator2
@decorator3
class MyClass
    -- Execution order: decorator3, decorator2, decorator1
end

class Example
    @log
    @validate(someValidator)
    @readonly
    property: string
    -- Applied in order: readonly, validate, log
end
```

### Getter/Setter Decorators

Decorate getters and setters:

```lua
function memoize(target: any, key: string, descriptor: any): void
    const cache = {}

    const originalGetter = descriptor.get

    descriptor.get = function(self)
        if cache[self] == nil then
            cache[self] = originalGetter(self)
        end
        return cache[self]
    end
end

class ExpensiveCalculator
    private data: number[]

    constructor(data: number[])
        self.data = data
    end

    @memoize
    get sum(): number
        print("Computing sum...")
        local total = 0
        for _, value in ipairs(self.data) do
            total = total + value
        end
        return total
    end
end

const calc = ExpensiveCalculator.new({1, 2, 3, 4, 5})
print(calc.sum)  -- Computing sum... 15
print(calc.sum)  -- 15 (cached, no recomputation)
```

### Operator Decorator

Decorate operator overloads:

```lua
function commutative(target: any, key: string, descriptor: any): void
    const originalOp = descriptor.value

    descriptor.value = function(self, other)
        -- Try both orders
        const result = originalOp(self, other) or originalOp(other, self)
        return result
    end
end

class Value
    private value: number

    constructor(value: number)
        self.value = value
    end

    @commutative
    operator +(other: Value): Value
        return Value.new(self.value + other.value)
    end
end
```

### Built-in Decorators

LuaNext provides several built-in decorators:

#### @readonly

Mark properties as read-only:

```lua
class Config
    @readonly
    version: string

    constructor(version: string)
        self.version = version
    end
end

const config = Config.new("1.0.0")
-- config.version = "2.0.0"  -- ❌ Error
```

#### @sealed

Prevent class extension:

```lua
@sealed
class FinalClass
    -- Cannot be extended
end
```

#### @deprecated

Mark members as deprecated:

```lua
class API
    @deprecated
    function oldMethod(): void
        print("Old method")
    end

    @deprecated("Use newMethod2() instead")
    function newMethod(): void
        print("New method")
    end
end
```

### Decorator Factory Pattern

Create configurable decorators:

```lua
function retry(maxAttempts: number, delay: number): (target: any, key: string, descriptor: any) => void
    return function(target, key, descriptor)
        const originalMethod = descriptor.value

        descriptor.value = function(self, ...)
            local attempts = 0

            while attempts < maxAttempts do
                attempts = attempts + 1

                const success, result = pcall(function()
                    return originalMethod(self, ...)
                end)

                if success then
                    return result
                end

                if attempts < maxAttempts then
                    print(`Retry ${attempts}/${maxAttempts} after ${delay}ms`)
                    sleep(delay)
                end
            end

            error(`Failed after ${maxAttempts} attempts`)
        end
    end
end

class NetworkClient
    @retry(3, 1000)
    function fetchData(url: string): string
        -- May fail, will retry up to 3 times
        return httpGet(url)
    end
end
```

### Type-Safe Decorators

Decorators with proper typing:

```lua
type PropertyDecorator = (target: any, propertyKey: string) => void
type MethodDecorator = (target: any, propertyKey: string, descriptor: PropertyDescriptor) => void
type ClassDecorator = (constructor: any) => void

function enumerable(value: boolean): PropertyDecorator
    return function(target, propertyKey)
        -- Set enumerable property
        target.__enumerable = target.__enumerable or {}
        target.__enumerable[propertyKey] = value
    end
end

class Example
    @enumerable(false)
    private secret: string

    @enumerable(true)
    public data: string
end
```

### Decorator Composition

Combine decorators for complex behavior:

```lua
function compose(...decorators: ((target: any, key: string, descriptor: any) => void)[]): (target: any, key: string, descriptor: any) => void
    return function(target, key, descriptor)
        for i = #decorators, 1, -1 do
            decorators[i](target, key, descriptor)
        end
    end
end

const logAndValidate = compose(log, validate(someValidator))

class Service
    @logAndValidate
    function process(data: string): void
        -- Both logging and validation applied
    end
end
```

## Details

### Decorator Targets

Decorators can be applied to:

1. **Classes** — Modify class constructors
2. **Properties** — Modify property descriptors
3. **Methods** — Modify method descriptors
4. **Getters** — Modify getter descriptors
5. **Setters** — Modify setter descriptors
6. **Operators** — Modify operator overload descriptors

### Decorator Signatures

Different decorator types have different signatures:

```lua
-- Class decorator
type ClassDecorator = (constructor: any) => void

-- Property decorator
type PropertyDecorator = (target: any, propertyKey: string) => void

-- Method decorator
type MethodDecorator = (target: any, propertyKey: string, descriptor: PropertyDescriptor) => void
```

### Execution Order

When multiple decorators are applied:

1. **Decorators are evaluated** top-to-bottom
2. **Decorators are executed** bottom-to-top

```lua
@first   -- Evaluated 1st, executed 3rd
@second  -- Evaluated 2nd, executed 2nd
@third   -- Evaluated 3rd, executed 1st
class MyClass
end
```

### PropertyDescriptor

Method decorators receive a PropertyDescriptor:

```lua
interface PropertyDescriptor
    value: any          -- The method function
    writable: boolean   -- Can be reassigned
    enumerable: boolean -- Appears in iteration
    configurable: boolean -- Can be deleted or reconfigured
end
```

### Decorator Metadata

Store metadata on decorated members:

```lua
function metadata(key: string, value: any): (target: any, propertyKey: string) => void
    return function(target, propertyKey)
        target.__metadata = target.__metadata or {}
        target.__metadata[propertyKey] = target.__metadata[propertyKey] or {}
        target.__metadata[propertyKey][key] = value
    end
end

class User
    @metadata("required", true)
    @metadata("minLength", 3)
    name: string
end

-- Access metadata
const nameMetadata = User.__metadata.name
print(nameMetadata.required)   -- true
print(nameMetadata.minLength)  -- 3
```

### Compile-Time vs Runtime

Decorators are:

- **Evaluated at compile time** — Decorator expressions are resolved during compilation
- **Executed at runtime** — Decorator functions run when the class is defined

### Decorator Limitations

- Cannot add new properties to classes (Lua limitation)
- Cannot change property types
- Limited reflection capabilities

### Best Practices

1. **Keep decorators pure** — Avoid side effects
2. **Document behavior** — Clearly describe what decorators do
3. **Use type annotations** — Provide proper types for decorator functions
4. **Compose carefully** — Be mindful of decorator interaction
5. **Test thoroughly** — Decorators modify behavior in non-obvious ways

## See Also

- [Classes](classes.md) — Class and member declarations
- [Advanced Types](advanced-types.md) — Type utilities for decorators
- [Reflection](../reference/reflection.md) — Runtime reflection API
