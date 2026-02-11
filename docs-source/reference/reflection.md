# Reflection API Reference

Runtime reflection utilities for inspecting types, classes, and metadata in LuaNext.

## Overview

LuaNext provides a reflection API for runtime type inspection, primarily used with classes and decorators. The `Reflect` namespace offers functions to query type information, fields, methods, and metadata.

**Key features:**

- Runtime type checking
- Field and method inspection
- Metadata access (from decorators)
- Instance checking

## Reflection Modes

Configure reflection metadata generation in `luanext.config.yaml`:

```yaml
compilerOptions:
  reflection: "selective"  # Default: metadata for decorated items only
  # OR
  reflection: "full"       # Metadata for all types and functions
  # OR
  reflection: "none"       # No reflection metadata
```

**Modes:**

- `selective` — Include metadata only for items with decorators (minimal overhead)
- `full` — Include metadata for all types, classes, functions (complete introspection)
- `none` — No metadata (smallest bundle size, reflection APIs return nil)

## Reflect Namespace

### `Reflect.getType(obj: unknown): string | nil`

Returns the LuaNext type name of an object.

```lua
import {Reflect} from "luanext:reflect"

class User
    name: string
    age: number
end

const user = User.new()
const typeName = Reflect.getType(user)  -- "User"

print(typeName)  -- "User"
```

**Returns:**

- Type name (e.g., `"User"`, `"Vector"`)
- `nil` if object has no reflection metadata

### `Reflect.getFields(obj: unknown): {[string]: FieldInfo} | nil`

Returns all fields (including inherited) of an object.

```lua
interface FieldInfo
    name: string
    type: string
    readonly: boolean
    visibility: "public" | "private" | "protected"
end

const user = User.new()
const fields = Reflect.getFields(user)

if fields ~= nil then
    for name, info in pairs(fields) do
        print(`${name}: ${info.type} (${info.visibility})`)
    end
end
-- Output:
-- name: string (public)
-- age: number (public)
```

**Includes inherited fields from parent classes.**

### `Reflect.getOwnFields(obj: unknown): {[string]: FieldInfo} | nil`

Returns only fields defined directly on the object's class (excluding inherited).

```lua
class Animal
    name: string
end

class Dog extends Animal
    breed: string
end

const dog = Dog.new()

const allFields = Reflect.getFields(dog)      -- {name, breed}
const ownFields = Reflect.getOwnFields(dog)   -- {breed} only
```

### `Reflect.getMethods(obj: unknown): {[string]: MethodInfo} | nil`

Returns all methods (including inherited) of an object.

```lua
interface MethodInfo
    name: string
    parameters: ParameterInfo[]
    returnType: string
    visibility: "public" | "private" | "protected"
end

interface ParameterInfo
    name: string
    type: string
    optional: boolean
end

const user = User.new()
const methods = Reflect.getMethods(user)

if methods ~= nil then
    for name, info in pairs(methods) do
        print(`${name}: (${formatParams(info.parameters)}) => ${info.returnType}`)
    end
end
```

### `Reflect.getOwnMethods(obj: unknown): {[string]: MethodInfo} | nil`

Returns only methods defined directly on the object's class (excluding inherited).

```lua
class Animal
    function speak(): void end
end

class Dog extends Animal
    function bark(): void end
end

const dog = Dog.new()

const allMethods = Reflect.getMethods(dog)      -- {speak, bark}
const ownMethods = Reflect.getOwnMethods(dog)   -- {bark} only
```

### `Reflect.isInstance(obj: unknown, classRef: unknown): boolean`

Checks if an object is an instance of a class.

```lua
class Animal end
class Dog extends Animal end
class Cat extends Animal end

const dog = Dog.new()

print(Reflect.isInstance(dog, Dog))     -- true
print(Reflect.isInstance(dog, Animal))  -- true (parent class)
print(Reflect.isInstance(dog, Cat))     -- false
```

**More precise than `instanceof` operator** when working with dynamic types.

## Decorator Metadata

Decorators can store metadata accessible via reflection.

### Storing Metadata

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

    @metadata("required", true)
    @metadata("min", 18)
    age: number
end
```

### Accessing Metadata

```lua
function validate(obj: User): boolean
    const fields = Reflect.getFields(obj)

    if fields ~= nil then
        for fieldName, fieldInfo in pairs(fields) do
            const meta = User.__metadata[fieldName]

            if meta ~= nil then
                if meta.required and obj[fieldName] == nil then
                    error(`Field ${fieldName} is required`)
                end

                if meta.minLength ~= nil and #obj[fieldName] < meta.minLength then
                    error(`Field ${fieldName} must be at least ${meta.minLength} characters`)
                end
            end
        end
    end

    return true
end

const user = User.new()
user.name = "AB"  -- Too short
validate(user)    -- Error: Field name must be at least 3 characters
```

## Practical Examples

### Serialization

```lua
function serialize(obj: unknown): string
    const typeName = Reflect.getType(obj)

    if typeName == nil then
        return tostring(obj)
    end

    const fields = Reflect.getFields(obj)
    if fields == nil then
        return tostring(obj)
    end

    const parts: string[] = {`{type="${typeName}"`}

    for name, info in pairs(fields) do
        const value = obj[name]
        table.insert(parts, `${name}=${serializeValue(value)}`)
    end

    return table.concat(parts, ", ") .. "}"
end
```

### Validation Framework

```lua
function validateObject<T>(obj: T): ValidationResult
    const errors: string[] = {}
    const fields = Reflect.getFields(obj)

    if fields ~= nil then
        for name, info in pairs(fields) do
            -- Check required fields
            if info.required and obj[name] == nil then
                table.insert(errors, `${name} is required`)
            end

            -- Type validation
            const actualType = type(obj[name])
            if not isTypeCompatible(actualType, info.type) then
                table.insert(errors, `${name} has wrong type: expected ${info.type}, got ${actualType}`)
            end
        end
    end

    return {valid = #errors == 0, errors = errors}
end
```

### Dependency Injection

```lua
function inject(container: Container, target: any): void
    const fields = Reflect.getFields(target)

    if fields ~= nil then
        for name, info in pairs(fields) do
            const meta = target.__metadata?[name]

            if meta?.inject == true then
                const dependency = container:resolve(info.type)
                if dependency ~= nil then
                    target[name] = dependency
                end
            end
        end
    end
end

@injectable
class UserService
    @inject
    database: Database

    @inject
    logger: Logger
end

const service = UserService.new()
inject(container, service)  -- Auto-injects dependencies
```

### Auto-mapping

```lua
function mapTo<T>(source: any, targetClass: any): T
    const target = targetClass.new()
    const sourceFields = Reflect.getFields(source)
    const targetFields = Reflect.getFields(target)

    if sourceFields ~= nil and targetFields ~= nil then
        for name, sourceInfo in pairs(sourceFields) do
            const targetInfo = targetFields[name]

            if targetInfo ~= nil and sourceInfo.type == targetInfo.type then
                target[name] = source[name]
            end
        end
    end

    return target
end

const dto = UserDTO.new()
dto.name = "Alice"
dto.email = "alice@example.com"

const entity: User = mapTo(dto, User)
```

## Performance Considerations

### Reflection Overhead

**Metadata storage:**

- `selective` mode — Minimal (only decorated items)
- `full` mode — Moderate (all types)
- `none` mode — Zero (no metadata)

**Runtime cost:**

- Reflection calls involve table lookups
- Use caching for frequently accessed metadata
- Avoid reflection in hot paths

### Optimization Tips

```lua
-- ✅ Good: Cache metadata
const fieldsCache = {}

function getCachedFields(obj: unknown): any
    const typeName = Reflect.getType(obj)

    if typeName == nil then
        return nil
    end

    if fieldsCache[typeName] == nil then
        fieldsCache[typeName] = Reflect.getFields(obj)
    end

    return fieldsCache[typeName]
end

-- ❌ Bad: Repeated reflection in loop
for i = 1, 10000 do
    const fields = Reflect.getFields(obj)  -- Called 10000 times!
end
```

### Bundle Size Impact

**Reflection mode comparison:**

```
none:      100 KB (baseline)
selective: 105 KB (+5%, metadata for decorated items only)
full:      130 KB (+30%, metadata for all types)
```

**Recommendation:** Use `selective` mode unless you need full introspection.

## Limitations

1. **Primitive types** — No reflection metadata for `number`, `string`, `boolean`, etc.
2. **External types** — No reflection for plain Lua tables or external C types
3. **Private access** — Reflection respects visibility modifiers in generated code
4. **Dynamic types** — Reflection works on compiled types, not runtime `any`/`unknown`

## Type Guards with Reflection

Combine reflection with type guards:

```lua
function isUser(obj: unknown): obj is User
    const typeName = Reflect.getType(obj)
    return typeName == "User"
end

function processValue(value: unknown): void
    if isUser(value) then
        -- value is User here
        print(value.name)
    end
end
```

## Debugging

Enable reflection debugging:

```bash
RUST_LOG=debug luanext main.luax
```

Check generated metadata:

```lua
-- Inspect raw metadata
const user = User.new()
print(user.__metadata)       -- Decorator metadata
print(user.__type)           -- Type name
print(user.__fields)         -- Field definitions
```

## See Also

- [Decorators](../language/decorators.md) — Using decorators with reflection
- [Classes](../language/classes.md) — Class definitions
- [Configuration](configuration.md) — Setting reflection mode
