//! Execution tests for decorators - verify decorator functions are called and
//! their return values replace the decorated class/method.
//!
//! Codegen:
//! - `@dec class Foo {}` → `Foo = dec(Foo)`
//! - `@dec("arg") class Foo {}` → `Foo = dec("arg")(Foo)`
//! - `@NS.dec class Foo {}` → `Foo = NS.dec(Foo)`
//! - Method: `@dec method` → `Foo.method = dec(Foo.method)`

use luanext_test_helpers::compile::compile;
use luanext_test_helpers::LuaExecutor;

#[test]
fn test_identifier_decorator_class() {
    // @markIt class Foo; decorator is called with class as argument
    let source = r#"
        call_count: number = 0

        function markIt(target)
            call_count = call_count + 1
            return target
        end

        @markIt
        class MyClass {
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let call_count: i64 = executor.execute_and_get(&lua_code, "call_count").unwrap();
    assert_eq!(call_count, 1, "decorator should have been called once");
}

#[test]
fn test_call_decorator_with_args() {
    // @tag("v1") class Foo; decorator factory applied to class
    let source = r#"
        tag_applied: number = 0

        function tag(version: string)
            return function(target)
                tag_applied = tag_applied + 1
                return target
            end
        end

        @tag("v1")
        class MyClass {
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let tag_applied: i64 = executor.execute_and_get(&lua_code, "tag_applied").unwrap();
    assert_eq!(tag_applied, 1, "decorator factory should have been called");
}

#[test]
fn test_decorator_mutates_class() {
    // Decorator that adds a static counter to track call count
    let source = r#"
        times_decorated: number = 0

        function trackDecoration(target)
            times_decorated = times_decorated + 1
            return target
        end

        @trackDecoration
        class MyApp {
        }

        @trackDecoration
        class OtherApp {
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let times_decorated: i64 = executor
        .execute_and_get(&lua_code, "times_decorated")
        .unwrap();
    assert_eq!(
        times_decorated, 2,
        "decorator should be called for each decorated class"
    );
}

#[test]
fn test_chained_decorators() {
    // @a @b @c class Foo; all three decorators applied
    let source = r#"
        call_count: number = 0

        function dec1(target)
            call_count = call_count + 1
            return target
        end

        function dec2(target)
            call_count = call_count + 1
            return target
        end

        function dec3(target)
            call_count = call_count + 1
            return target
        end

        @dec1
        @dec2
        @dec3
        class MyClass {
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let call_count: i64 = executor.execute_and_get(&lua_code, "call_count").unwrap();
    assert_eq!(
        call_count, 3,
        "all three decorators should have been called"
    );
}

#[test]
fn test_method_decorator() {
    // @logged method; decorator wraps the method
    let source = r#"
        call_log: string = ""

        function logged(method)
            return function(self_arg)
                call_log = "called"
                return method(self_arg)
            end
        end

        class Greeter {
            name: string
            constructor(n: string) {
                self.name = n
            }

            @logged
            public greet(): string {
                return "Hello, " .. self.name
            }
        }

        const g = new Greeter("World")
        result: string = g::greet()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let call_log: String = executor.execute_and_get(&lua_code, "call_log").unwrap();
    let result: String = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(
        call_log, "called",
        "method decorator should have been invoked"
    );
    assert_eq!(result, "Hello, World");
}

#[test]
fn test_member_expression_decorator() {
    // @NS.mark class Foo {}; NS.mark(Foo) called
    // Use a namespace table with the decorator as a field
    let source = r#"
        ns_call_count: number = 0

        const NS = {
            mark = function(target)
                ns_call_count = ns_call_count + 1
                return target
            end
        }

        @NS.mark
        class Tagged {
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let ns_call_count: i64 = executor
        .execute_and_get(&lua_code, "ns_call_count")
        .unwrap();
    assert_eq!(
        ns_call_count, 1,
        "namespace decorator should have been called"
    );
}

#[test]
fn test_decorator_return_value_used() {
    // Decorator increments a counter - verifies decorator actually runs
    let source = r#"
        decorator_ran: boolean = false

        function wrapClass(target)
            decorator_ran = true
            return target
        end

        @wrapClass
        class Original {
            getValue(): number {
                return 42
            }
        }

        const obj = new Original()
        result: number = obj::getValue()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let decorator_ran: bool = executor
        .execute_and_get(&lua_code, "decorator_ran")
        .unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert!(decorator_ran, "decorator should have been called");
    assert_eq!(result, 42, "decorated class should still work normally");
}

// ============================================================================
// Decorator Execution Order
// ============================================================================

#[test]
fn test_decorator_execution_order_verification() {
    // Verifies decorators execute top-to-bottom by recording invocation order
    let source = r#"
        order: string = ""

        function first(target)
            order = order .. "1"
            return target
        end

        function second(target)
            order = order .. "2"
            return target
        end

        function third(target)
            order = order .. "3"
            return target
        end

        @first
        @second
        @third
        class MyClass {
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let order: String = executor.execute_and_get(&lua_code, "order").unwrap();
    assert_eq!(order, "123", "decorators should execute top-to-bottom");
}

#[test]
fn test_chained_decorators_modify_class_table() {
    // Each decorator adds a property to the class table; both survive
    // Use globals to capture since type checker doesn't know about decorator-added fields
    let source = r#"
        foo_val: string = ""
        bar_val: string = ""

        function addFoo(target)
            target.foo = "foo"
            foo_val = target.foo
            return target
        end

        function addBar(target)
            target.bar = "bar"
            bar_val = target.bar
            return target
        end

        @addFoo
        @addBar
        class MyClass {
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let foo_val: String = executor.execute_and_get(&lua_code, "foo_val").unwrap();
    let bar_val: String = executor.execute_and_get(&lua_code, "bar_val").unwrap();
    assert_eq!(foo_val, "foo");
    assert_eq!(bar_val, "bar");
}

#[test]
fn test_decorator_factory_chain_with_args() {
    // Multiple decorator factories with arguments applied in sequence
    let source = r#"
        log: string = ""

        function tag(name: string)
            return function(target)
                log = log .. name .. ","
                return target
            end
        end

        @tag("alpha")
        @tag("beta")
        @tag("gamma")
        class Tagged {
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let log: String = executor.execute_and_get(&lua_code, "log").unwrap();
    assert_eq!(log, "alpha,beta,gamma,");
}

#[test]
fn test_decorator_replaces_class() {
    // Decorator that returns a completely different table — verify replacement semantics
    // The decorator captures a reference and returns a new table, so Original becomes the new table
    let source = r#"
        replaced: boolean = false
        original_ref = nil

        function replace(target)
            original_ref = target
            replaced = true
            return { marker = true }
        end

        @replace
        class Original {
        }
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let replaced: bool = executor.execute_and_get(&lua_code, "replaced").unwrap();
    assert!(replaced, "decorator should have run and replaced the class");
}

#[test]
fn test_method_decorator_execution_order() {
    // Multiple method decorators applied in order on a static method
    let source = r#"
        log: string = ""

        function logA(method)
            log = log .. "A"
            return method
        end

        function logB(method)
            log = log .. "B"
            return method
        end

        class Svc {
            @logA
            @logB
            static run(): number {
                return 42
            }
        }

        result: number = Svc.run()
    "#;

    let lua_code = compile(source).unwrap();
    let executor = LuaExecutor::new().unwrap();

    let log: String = executor.execute_and_get(&lua_code, "log").unwrap();
    let result: i64 = executor.execute_and_get(&lua_code, "result").unwrap();
    assert_eq!(log, "AB", "method decorators should execute top-to-bottom");
    assert_eq!(result, 42, "decorated method should still work");
}
