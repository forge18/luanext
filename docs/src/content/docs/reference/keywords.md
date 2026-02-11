---
title: Keywords Reference
---

# Keywords Reference

Complete list of reserved keywords in LuaNext.

## Reserved Keywords

Keywords cannot be used as identifiers (variable names, function names, etc.).

### Lua Keywords

Standard Lua keywords (inherited):

```text
and       break     do        else      elseif
end       false     for       function  goto
if        in        local     nil       not
or        repeat    return    then      true
until     while
```

### LuaNext Keywords

Additional keywords introduced by LuaNext:

```text
as          class       const       declare     decorator
export      extends     from        implements  import
instanceof  interface   namespace   new         override
readonly    type        typeof
```

### Feature-Specific Keywords

Keywords enabled with specific features:

**Pattern Matching:**

```text
match
```

**Error Handling:**

```text
catch     finally     rethrow     throw     throws     try
```

**Enums:**

```text
enum
```

## Usage

### Cannot Use as Identifiers

```lua
-- ❌ Error: Cannot use keyword as identifier
const function = 5
const class = "test"
local type = "string"
```

### Valid Alternatives

```lua
-- ✅ OK: Use descriptive names instead
const func = 5
const className = "test"
local dataType = "string"
```

## Contextual Keywords

Some keywords are only reserved in specific contexts:

### `declare`

Only reserved in declaration contexts:

```lua
-- ✅ OK: declare as property name
const obj = {declare = true}

-- ❌ Error: declare statement
declare function print(x: unknown): void
```

### `readonly`

Only reserved as a modifier:

```lua
-- ✅ OK: readonly as property name
const obj = {readonly = false}

-- ❌ Error: readonly modifier
interface User
    readonly id: string
end
```

### `new`

Reserved for constructor calls:

```lua
-- ❌ Error: new is reserved
const new = 5

-- ✅ OK: Used for constructors
const obj = MyClass.new()
```

## Soft Keywords

Some identifiers are not strictly reserved but should be avoided:

```text
never      unknown     void       _
```

These have special meaning in type contexts but can technically be used as identifiers (not recommended).

## Future Reserved

These keywords may be reserved in future versions:

```text
abstract    async    await    final    private
protected   public   static   super
```

**Avoid using these** to ensure forward compatibility.

## See Also

- [Basics](../language/basics.md) — Variable declarations
- [Type System](../language/type-system.md) — Type keywords
- [Classes](../language/classes.md) — Class keywords
