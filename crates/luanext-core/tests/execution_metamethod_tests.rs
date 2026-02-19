//! Execution tests for metamethods on custom tables.
//!
//! These tests demonstrate that raw Lua `setmetatable()` works correctly
//! for defining custom metamethods on plain tables. The type checker allows
//! `setmetatable()` calls (declared in builtins.d.tl), and the runtime
//! behavior is standard Lua metamethod dispatch.
//!
//! Note: These tests use raw Lua constructs since the type checker does not
//! track metamethod-enhanced behavior on plain tables (only classes use the
//! `operator` keyword for type-checked metamethods).

use luanext_test_helpers::LuaExecutor;

#[test]
fn test_metamethod_index_custom_lookup() {
    // __index metamethod for custom property lookup on a table
    let lua_code = r#"
        local defaults = { color = "red", size = 10 }
        local obj = {}
        setmetatable(obj, { __index = defaults })
        result = obj.color
    "#;

    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(lua_code, "result").unwrap();
    assert_eq!(result, "red");
}

#[test]
fn test_metamethod_index_function() {
    // __index as a function for computed property lookup
    let lua_code = r#"
        local obj = {}
        setmetatable(obj, {
            __index = function(t, k)
                return k .. "_value"
            end
        })
        result = obj.hello
    "#;

    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(lua_code, "result").unwrap();
    assert_eq!(result, "hello_value");
}

#[test]
fn test_metamethod_newindex_custom_assignment() {
    // __newindex metamethod intercepts property assignment
    let lua_code = r#"
        local log = {}
        local obj = {}
        setmetatable(obj, {
            __newindex = function(t, k, v)
                rawset(t, k, v * 2)
                table.insert(log, k)
            end
        })
        obj.x = 5
        obj.y = 10
        result_x = obj.x
        result_y = obj.y
        result_log = table.concat(log, ",")
    "#;

    let executor = LuaExecutor::new().unwrap();
    let result_x: i64 = executor.execute_and_get(lua_code, "result_x").unwrap();
    let result_y: i64 = executor.execute_and_get(lua_code, "result_y").unwrap();
    let result_log: String = executor.execute_and_get(lua_code, "result_log").unwrap();
    assert_eq!(result_x, 10);
    assert_eq!(result_y, 20);
    assert_eq!(result_log, "x,y");
}

#[test]
fn test_metamethod_len_custom_length() {
    // __len metamethod for custom # operator behavior
    let lua_code = r#"
        local obj = { items = {1, 2, 3, 4, 5} }
        setmetatable(obj, {
            __len = function(t)
                return #t.items
            end
        })
        result = #obj
    "#;

    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(lua_code, "result").unwrap();
    assert_eq!(result, 5);
}

#[test]
fn test_metamethod_add_on_table() {
    // __add metamethod for custom + operator on plain tables
    let lua_code = r#"
        local mt = {
            __add = function(a, b)
                return { x = a.x + b.x, y = a.y + b.y }
            end
        }
        local v1 = setmetatable({ x = 1, y = 2 }, mt)
        local v2 = setmetatable({ x = 3, y = 4 }, mt)
        local v3 = v1 + v2
        result_x = v3.x
        result_y = v3.y
    "#;

    let executor = LuaExecutor::new().unwrap();
    let result_x: i64 = executor.execute_and_get(lua_code, "result_x").unwrap();
    let result_y: i64 = executor.execute_and_get(lua_code, "result_y").unwrap();
    assert_eq!(result_x, 4);
    assert_eq!(result_y, 6);
}

#[test]
fn test_metamethod_call_callable_table() {
    // __call metamethod makes a table callable
    let lua_code = r#"
        local counter = { count = 0 }
        setmetatable(counter, {
            __call = function(t)
                t.count = t.count + 1
                return t.count
            end
        })
        local r1 = counter()
        local r2 = counter()
        local r3 = counter()
        result = r3
    "#;

    let executor = LuaExecutor::new().unwrap();
    let result: i64 = executor.execute_and_get(lua_code, "result").unwrap();
    assert_eq!(result, 3);
}

#[test]
fn test_metamethod_tostring() {
    // __tostring metamethod for custom string representation
    let lua_code = r#"
        local point = { x = 10, y = 20 }
        setmetatable(point, {
            __tostring = function(t)
                return "(" .. t.x .. ", " .. t.y .. ")"
            end
        })
        result = tostring(point)
    "#;

    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(lua_code, "result").unwrap();
    assert_eq!(result, "(10, 20)");
}

#[test]
fn test_metamethod_index_chain_inheritance() {
    // Metatable inheritance via __index pointing to another table
    let lua_code = r#"
        local base = { greet = function(self) return "Hello from " .. self.name end }
        local derived_mt = { __index = base }
        local obj = setmetatable({ name = "World" }, derived_mt)
        result = obj:greet()
    "#;

    let executor = LuaExecutor::new().unwrap();
    let result: String = executor.execute_and_get(lua_code, "result").unwrap();
    assert_eq!(result, "Hello from World");
}
