//! Lua code execution helpers for testing generated code
//!
//! This module provides utilities for executing generated Lua code
//! and validating its correctness.
//!
//! Note: The Lua version is determined at compile-time by mlua features.
//! By default, Lua 5.4 is used (via workspace mlua dependency).

use mlua::{FromLua, Lua, Value};

/// Executor for running generated Lua code in tests
///
/// Provides methods for executing Lua code and extracting results.
/// The Lua version is determined at compile time by mlua cargo features.
pub struct LuaExecutor {
    lua: Lua,
}

impl LuaExecutor {
    /// Creates a new Lua executor
    ///
    /// # Errors
    ///
    /// Returns an error if Lua initialization fails
    pub fn new() -> Result<Self, String> {
        let lua = Lua::new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default())
            .map_err(|e| format!("Failed to create Lua instance: {e}"))?;

        Ok(Self { lua })
    }

    /// Returns the Lua version being used (compile-time determined)
    ///
    /// Note: mlua features are set at the workspace level. By default, Lua 5.4 is used.
    pub fn version_info(&self) -> &'static str {
        // mlua embeds the Lua version info, we just return a static string
        // The actual version depends on which mlua feature is enabled at compile-time
        "Lua (version determined by mlua workspace feature)"
    }

    /// Executes Lua code without returning a value
    ///
    /// # Arguments
    ///
    /// * `code` - The Lua code to execute
    ///
    /// # Errors
    ///
    /// Returns an error if the code fails to execute
    ///
    /// # Example
    ///
    /// ```ignore
    /// let executor = LuaExecutor::new()?;
    /// executor.execute("local x = 10")?;
    /// ```
    pub fn execute(&self, code: &str) -> Result<(), String> {
        self.lua
            .load(code)
            .exec()
            .map_err(|e| format!("Lua execution failed: {e}"))
    }

    /// Executes Lua code and retrieves a global variable
    ///
    /// # Arguments
    ///
    /// * `code` - The Lua code to execute
    /// * `var_name` - The name of the global variable to retrieve
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails or the variable cannot be converted
    ///
    /// # Example
    ///
    /// ```ignore
    /// let executor = LuaExecutor::new()?;
    /// let result: i64 = executor.execute_and_get("x = 42", "x")?;
    /// assert_eq!(result, 42);
    /// ```
    pub fn execute_and_get<T: FromLua>(&self, code: &str, var_name: &str) -> Result<T, String> {
        // Execute the code
        self.execute(code)?;

        // Retrieve the global variable
        let globals = self.lua.globals();
        let value: T = globals
            .get(var_name)
            .map_err(|e| format!("Failed to get variable '{var_name}': {e}"))?;

        Ok(value)
    }

    /// Executes Lua code and returns the result of the expression
    ///
    /// # Arguments
    ///
    /// * `code` - The Lua code to evaluate (should return a value)
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails or the result cannot be converted
    ///
    /// # Example
    ///
    /// ```ignore
    /// let executor = LuaExecutor::new()?;
    /// let result: i64 = executor.execute_with_result("return 1 + 2")?;
    /// assert_eq!(result, 3);
    /// ```
    pub fn execute_with_result<T: FromLua>(&self, code: &str) -> Result<T, String> {
        self.lua
            .load(code)
            .eval()
            .map_err(|e| format!("Lua execution failed: {e}"))
    }

    /// Checks if the code executes successfully (ignores result)
    ///
    /// # Arguments
    ///
    /// * `code` - The Lua code to execute
    ///
    /// # Returns
    ///
    /// `true` if execution succeeded, `false` otherwise
    ///
    /// # Example
    ///
    /// ```ignore
    /// let executor = LuaExecutor::new()?;
    /// assert!(executor.execute_ok("local x = 10"));
    /// assert!(!executor.execute_ok("syntax error here"));
    /// ```
    pub fn execute_ok(&self, code: &str) -> bool {
        self.execute(code).is_ok()
    }

    /// Gets access to the underlying mlua::Lua instance for advanced usage
    ///
    /// This is useful for custom testing scenarios that need direct
    /// Lua API access.
    pub fn lua(&self) -> &Lua {
        &self.lua
    }
}

/// Extension trait for convenient value extraction from mlua::Value
pub trait LuaValueExt {
    /// Attempts to extract an i64 from the value
    fn as_i64(&self) -> Option<i64>;

    /// Attempts to extract an f64 from the value
    fn as_f64(&self) -> Option<f64>;

    /// Attempts to extract a String from the value
    fn as_string(&self) -> Option<String>;

    /// Attempts to extract a bool from the value
    fn as_bool(&self) -> Option<bool>;
}

impl LuaValueExt for Value {
    fn as_i64(&self) -> Option<i64> {
        // In mlua 0.10, we use pattern matching on Value enum
        match self {
            Value::Integer(i) => Some(*i),
            Value::Number(n) => Some(*n as i64),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<String> {
        match self {
            Value::String(ref s) => s.to_str().ok().map(|s| s.to_owned()),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = LuaExecutor::new();
        assert!(executor.is_ok());
    }

    #[test]
    fn test_version_info() {
        let executor = LuaExecutor::new().unwrap();
        // Should return version info
        let version = executor.version_info();
        assert!(version.contains("Lua"));
    }

    #[test]
    fn test_basic_execution() {
        let executor = LuaExecutor::new().unwrap();
        assert!(executor.execute("local x = 10").is_ok());
    }

    #[test]
    fn test_execute_and_get() {
        let executor = LuaExecutor::new().unwrap();
        let result: i64 = executor.execute_and_get("x = 42", "x").unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_execute_with_result() {
        let executor = LuaExecutor::new().unwrap();
        let result: i64 = executor.execute_with_result("return 1 + 2").unwrap();
        assert_eq!(result, 3);
    }

    #[test]
    fn test_execute_ok() {
        let executor = LuaExecutor::new().unwrap();
        assert!(executor.execute_ok("local x = 10"));
        assert!(!executor.execute_ok("this is not valid lua"));
    }

    #[test]
    fn test_value_ext_i64() {
        let executor = LuaExecutor::new().unwrap();
        executor.execute("x = 42").unwrap();
        let value: Value = executor.lua().globals().get("x").unwrap();
        assert_eq!(value.as_i64(), Some(42));
    }

    #[test]
    fn test_value_ext_f64() {
        let executor = LuaExecutor::new().unwrap();
        executor.execute("x = 1.234").unwrap();
        let value: Value = executor.lua().globals().get("x").unwrap();
        assert_eq!(value.as_f64(), Some(1.234));
    }

    #[test]
    fn test_value_ext_string() {
        let executor = LuaExecutor::new().unwrap();
        executor.execute("x = 'hello'").unwrap();
        let value: Value = executor.lua().globals().get("x").unwrap();
        let result = value.as_string();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_value_ext_bool() {
        let executor = LuaExecutor::new().unwrap();
        executor.execute("x = true").unwrap();
        let value: Value = executor.lua().globals().get("x").unwrap();
        assert_eq!(value.as_bool(), Some(true));
    }
}
