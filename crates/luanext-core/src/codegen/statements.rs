use super::super::config::OptimizationLevel;
use super::strategies::GlobalStyle;
use super::CodeGenerator;
use luanext_parser::ast::pattern::{
    ArrayPattern, ArrayPatternElement, ObjectPattern, Pattern, PatternWithDefault,
};
use luanext_parser::ast::statement::*;
use luanext_parser::prelude::Block;
use luanext_parser::prelude::Expression;

impl CodeGenerator {
    pub fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Variable(decl) => self.generate_variable_declaration(decl),
            Statement::Function(decl) => self.generate_function_declaration(decl),
            Statement::If(if_stmt) => self.generate_if_statement(if_stmt),
            Statement::While(while_stmt) => self.generate_while_statement(while_stmt),
            Statement::For(for_stmt) => self.generate_for_statement(for_stmt),
            Statement::Repeat(repeat_stmt) => self.generate_repeat_statement(repeat_stmt),
            Statement::Return(return_stmt) => self.generate_return_statement(return_stmt),
            Statement::Break(_) => {
                self.write_indent();
                self.writeln("break");
            }
            Statement::Continue(_) => {
                self.write_indent();
                let continue_code = self.strategy.generate_continue(None);
                self.writeln(&continue_code);
            }
            Statement::Expression(expr) => {
                self.write_indent();
                self.generate_expression(expr);
                self.writeln("");
            }
            Statement::Block(block) => self.generate_block(block),
            Statement::Interface(iface_decl) => self.generate_interface_declaration(iface_decl),
            Statement::TypeAlias(_) => {}
            Statement::Enum(decl) => self.generate_enum_declaration(decl),
            Statement::Class(class_decl) => self.generate_class_declaration(class_decl),
            Statement::Import(import) => self.generate_import(import),
            Statement::Export(export) => self.generate_export(export),
            Statement::DeclareFunction(_)
            | Statement::DeclareNamespace(_)
            | Statement::DeclareType(_)
            | Statement::DeclareInterface(_)
            | Statement::DeclareConst(_) => {}
            Statement::Throw(throw_stmt) => self.generate_throw_statement(throw_stmt),
            Statement::Try(try_stmt) => self.generate_try_statement(try_stmt),
            Statement::Rethrow(span) => self.generate_rethrow_statement(*span),
            Statement::Namespace(ns) => self.generate_namespace_declaration(ns),
            Statement::Label(label) => {
                self.write_indent();
                let name = self.interner.resolve(label.name.node);
                self.writeln(&format!("::{name}::"));
            }
            Statement::Goto(goto) => {
                self.write_indent();
                let name = self.interner.resolve(goto.target.node);
                self.writeln(&format!("goto {name}"));
            }
            Statement::MultiAssignment(multi) => {
                self.write_indent();
                for (i, target) in multi.targets.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_expression(target);
                }
                self.write(" = ");
                for (i, value) in multi.values.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.generate_expression(value);
                }
                self.writeln("");
            }
        }
    }

    /// Check if this global should use `rawset(_G, ...)` style.
    fn is_rawset_global(&self, kind: &VariableKind) -> bool {
        matches!(kind, VariableKind::Global) && self.strategy.global_style() == GlobalStyle::Rawset
    }

    /// Emit the appropriate variable declaration prefix for the given kind.
    /// For Local/Const: emits "local "
    /// For Global on Lua 5.5: emits "global "
    /// For Global with rawset: emits nothing (caller handles rawset)
    fn emit_var_prefix(&mut self, kind: &VariableKind) {
        match kind {
            VariableKind::Local | VariableKind::Const => {
                self.write("local ");
            }
            VariableKind::Global => match self.strategy.global_style() {
                GlobalStyle::NativeKeyword => self.write("global "),
                GlobalStyle::Rawset => {} // handled by emit_rawset_assignment
            },
        }
    }

    /// Emit a complete `rawset(_G, "name", <value>)` assignment.
    /// The caller provides the value by writing between `emit_rawset_start` and `emit_rawset_end`.
    /// Emit `rawset(_G, "name", ` — caller must follow with value and `emit_rawset_end()`.
    fn emit_rawset_start(&mut self, name: &str) {
        self.write(&format!("rawset(_G, \"{}\", ", name));
    }

    /// Emit the closing `)` for a rawset call.
    fn emit_rawset_end(&mut self) {
        self.write(")");
    }

    /// Emit a complete variable assignment: `name = value_str` or `rawset(_G, "name", value_str)`.
    /// Use this for simple string-based value expressions (no need for generate_expression).
    fn emit_var_assign(&mut self, kind: &VariableKind, name: &str, value_str: &str) {
        self.write_indent();
        if self.is_rawset_global(kind) {
            self.emit_rawset_start(name);
            self.write(value_str);
            self.emit_rawset_end();
        } else {
            self.emit_var_prefix(kind);
            self.write(name);
            self.write(" = ");
            self.write(value_str);
        }
        self.writeln("");
    }

    pub fn generate_variable_declaration(&mut self, decl: &VariableDeclaration) {
        match &decl.pattern {
            Pattern::Identifier(ident) => {
                self.write_indent();
                if self.is_rawset_global(&decl.kind) {
                    let name = self.resolve(ident.node);
                    self.emit_rawset_start(&name);
                    self.generate_expression(&decl.initializer);
                    self.emit_rawset_end();
                } else {
                    self.emit_var_prefix(&decl.kind);
                    self.generate_pattern(&decl.pattern);
                    self.write(" = ");
                    self.generate_expression(&decl.initializer);
                }
                self.writeln("");
            }
            Pattern::Wildcard(_) => {
                self.write_indent();
                self.emit_var_prefix(&decl.kind);
                self.generate_pattern(&decl.pattern);
                self.write(" = ");
                self.generate_expression(&decl.initializer);
                self.writeln("");
            }
            Pattern::Array(array_pattern) => {
                // Generate temporary variable and destructuring assignments
                // __temp is always local, even for global destructuring
                self.write_indent();
                self.write("local __temp = ");
                self.generate_expression(&decl.initializer);
                self.writeln("");
                self.generate_array_destructuring(array_pattern, "__temp", &decl.kind);
            }
            Pattern::Object(obj_pattern) => {
                // Generate temporary variable and destructuring assignments
                // __temp is always local, even for global destructuring
                self.write_indent();
                self.write("local __temp = ");
                self.generate_expression(&decl.initializer);
                self.writeln("");
                self.generate_object_destructuring(obj_pattern, "__temp", &decl.kind);
            }
            Pattern::Literal(_, _) => {
                // Literals in patterns don't bind variables - just evaluate the initializer
                self.write_indent();
                self.write("local _ = ");
                self.generate_expression(&decl.initializer);
                self.writeln("");
            }
            Pattern::Or(_) => {
                // Or-patterns should not appear in variable declarations
                // They are only valid in match expressions
                // Treat as wildcard (defensive programming)
                self.write_indent();
                self.write("local _ = ");
                self.generate_expression(&decl.initializer);
                self.writeln("");
            }
            Pattern::Template(_) => {
                // Template patterns should not appear in variable declarations
                // They are only valid in match expressions
                // Treat as wildcard (defensive programming)
                self.write_indent();
                self.write("local _ = ");
                self.generate_expression(&decl.initializer);
                self.writeln("");
            }
        }
    }

    /// Generate array destructuring assignments
    pub fn generate_array_destructuring(
        &mut self,
        pattern: &ArrayPattern,
        source: &str,
        kind: &VariableKind,
    ) {
        let mut index = 1; // Lua arrays are 1-indexed

        for elem in pattern.elements.iter() {
            match elem {
                ArrayPatternElement::Pattern(PatternWithDefault {
                    pattern: pat,
                    default,
                }) => {
                    match pat {
                        Pattern::Identifier(ident) => {
                            let resolved = self.resolve(ident.node);
                            if default.is_none() {
                                let value = format!("{}[{}]", source, index);
                                self.emit_var_assign(kind, &resolved, &value);
                            } else {
                                self.write_indent();
                                if self.is_rawset_global(kind) {
                                    self.emit_rawset_start(&resolved);
                                    self.write(&format!("{}[{}]", source, index));
                                    self.write(" or ");
                                    self.generate_expression(default.as_ref().unwrap());
                                    self.emit_rawset_end();
                                } else {
                                    self.emit_var_prefix(kind);
                                    self.write(&resolved);
                                    self.write(&format!(" = {}[{}]", source, index));
                                    self.write(" or ");
                                    self.generate_expression(default.as_ref().unwrap());
                                }
                                self.writeln("");
                            }
                        }
                        Pattern::Array(nested_array) => {
                            // Nested array destructuring — temp is always local
                            let temp_var = format!("__temp_{}", index);
                            let value = format!("{}[{}]", source, index);
                            self.emit_var_assign(&VariableKind::Local, &temp_var, &value);
                            self.generate_array_destructuring(nested_array, &temp_var, kind);
                        }
                        Pattern::Object(nested_obj) => {
                            // Nested object destructuring — temp is always local
                            let temp_var = format!("__temp_{}", index);
                            let value = format!("{}[{}]", source, index);
                            self.emit_var_assign(&VariableKind::Local, &temp_var, &value);
                            self.generate_object_destructuring(nested_obj, &temp_var, kind);
                        }
                        Pattern::Wildcard(_) | Pattern::Literal(_, _) => {
                            // Skip - don't bind anything
                        }
                        Pattern::Or(_) => {
                            // Or-patterns should not appear in destructuring
                            // Skip - don't bind anything
                        }
                        Pattern::Template(_) => {
                            // Template patterns should not appear in destructuring
                            // Skip - don't bind anything
                        }
                    }
                    index += 1;
                }
                ArrayPatternElement::Rest(ident) => {
                    // Rest element: collect remaining elements
                    let resolved = self.resolve(ident.node);
                    let value = format!("{{table.unpack({}, {})}}", source, index);
                    self.emit_var_assign(kind, &resolved, &value);
                    break; // Rest must be last
                }
                ArrayPatternElement::Hole => {
                    // Skip this element
                    index += 1;
                }
            }
        }
    }

    /// Write a property access expression: source.key or source[computed_expr]
    fn write_property_access(
        &mut self,
        source: &str,
        key_str: &str,
        computed_key: &Option<Expression>,
    ) {
        if let Some(expr) = computed_key {
            self.write(&format!("{}[", source));
            self.generate_expression(expr);
            self.write("]");
        } else {
            self.write(&format!("{}.{}", source, key_str));
        }
    }

    /// Emit a variable assignment where the value involves property access (possibly computed)
    /// and an optional default expression. Handles rawset for globals.
    fn emit_destructured_assign(
        &mut self,
        kind: &VariableKind,
        name: &str,
        source: &str,
        key_str: &str,
        computed_key: &Option<Expression>,
        default: Option<&Expression>,
    ) {
        // For non-computed keys without defaults, we can use the simple emit_var_assign
        if computed_key.is_none() && default.is_none() {
            let value = format!("{}.{}", source, key_str);
            self.emit_var_assign(kind, name, &value);
            return;
        }

        self.write_indent();
        if self.is_rawset_global(kind) {
            self.emit_rawset_start(name);
            self.write_property_access(source, key_str, computed_key);
            if let Some(default_expr) = default {
                self.write(" or ");
                self.generate_expression(default_expr);
            }
            self.emit_rawset_end();
        } else {
            self.emit_var_prefix(kind);
            self.write(name);
            self.write(" = ");
            self.write_property_access(source, key_str, computed_key);
            if let Some(default_expr) = default {
                self.write(" or ");
                self.generate_expression(default_expr);
            }
        }
        self.writeln("");
    }

    /// Generate object destructuring assignments
    pub fn generate_object_destructuring(
        &mut self,
        pattern: &ObjectPattern,
        source: &str,
        kind: &VariableKind,
    ) {
        for prop in pattern.properties.iter() {
            let key_str = self.resolve(prop.key.node);

            if let Some(value_pattern) = &prop.value {
                // { key: pattern } or { [expr]: pattern }
                match value_pattern {
                    Pattern::Identifier(ident) => {
                        let resolved = self.resolve(ident.node);
                        self.emit_destructured_assign(
                            kind,
                            &resolved,
                            source,
                            &key_str,
                            &prop.computed_key,
                            prop.default.as_ref(),
                        );
                    }
                    Pattern::Array(nested_array) => {
                        // Nested — temp is always local
                        let temp_var = format!("__temp_{}", key_str);
                        self.emit_destructured_assign(
                            &VariableKind::Local,
                            &temp_var,
                            source,
                            &key_str,
                            &prop.computed_key,
                            None,
                        );
                        self.generate_array_destructuring(nested_array, &temp_var, kind);
                    }
                    Pattern::Object(nested_obj) => {
                        // Nested — temp is always local
                        let temp_var = format!("__temp_{}", key_str);
                        self.emit_destructured_assign(
                            &VariableKind::Local,
                            &temp_var,
                            source,
                            &key_str,
                            &prop.computed_key,
                            None,
                        );
                        self.generate_object_destructuring(nested_obj, &temp_var, kind);
                    }
                    Pattern::Wildcard(_) | Pattern::Literal(_, _) => {}
                    Pattern::Or(_) => {}
                    Pattern::Template(_) => {}
                }
            } else {
                // Shorthand: { key } means { key: key }
                self.emit_destructured_assign(
                    kind,
                    &key_str,
                    source,
                    &key_str,
                    &prop.computed_key,
                    prop.default.as_ref(),
                );
            }
        }

        // Handle rest pattern: { a, ...rest }
        if let Some(rest_ident) = &pattern.rest {
            let rest_name = self.resolve(rest_ident.node);
            // Collect all non-destructured properties into a new table
            let destructured_keys: Vec<String> = pattern
                .properties
                .iter()
                .map(|p| self.resolve(p.key.node))
                .collect();

            self.emit_var_assign(kind, &rest_name, "{}");
            self.write_indent();
            self.writeln(&format!("for __k, __v in pairs({}) do", source));
            self.indent();

            if !destructured_keys.is_empty() {
                self.write_indent();
                let conditions: Vec<String> = destructured_keys
                    .iter()
                    .map(|k| format!("__k ~= \"{}\"", k))
                    .collect();
                self.writeln(&format!("if {} then", conditions.join(" and ")));
                self.indent();
            }

            self.write_indent();
            self.writeln(&format!("{}[__k] = __v", rest_name));

            if !destructured_keys.is_empty() {
                self.dedent();
                self.write_indent();
                self.writeln("end");
            }

            self.dedent();
            self.write_indent();
            self.writeln("end");
        }
    }

    pub fn generate_function_declaration(&mut self, decl: &FunctionDeclaration) {
        self.write_indent();
        self.write("local function ");
        let fn_name = self.resolve(decl.name.node);
        self.write(&fn_name);
        self.write("(");

        let mut rest_param_name: Option<luanext_parser::string_interner::StringId> = None;

        for (i, param) in decl.parameters.iter().enumerate() {
            if param.is_rest {
                // For rest parameters, just write ... in the parameter list
                if i > 0 {
                    self.write(", ");
                }
                self.write("...");
                // Save the parameter name to initialize it in the function body
                if let Pattern::Identifier(ident) = &param.pattern {
                    rest_param_name = Some(ident.node);
                }
            } else {
                if i > 0 {
                    self.write(", ");
                }
                self.generate_pattern(&param.pattern);
            }
        }

        self.writeln(")");
        self.indent();

        // If there's a rest parameter, initialize it from ...
        if let Some(rest_name) = rest_param_name {
            self.write_indent();
            self.write("local ");
            let rest_name_str = self.resolve(rest_name);
            self.write(&rest_name_str);
            self.writeln(" = {...}");
        }

        // Emit default parameter initialization
        for param in decl.parameters.iter() {
            if let Some(default_expr) = &param.default {
                if let Pattern::Identifier(ident) = &param.pattern {
                    let param_name = self.resolve(ident.node);
                    self.write_indent();
                    self.write("if ");
                    self.write(&param_name);
                    self.write(" == nil then ");
                    self.write(&param_name);
                    self.write(" = ");
                    self.generate_expression(default_expr);
                    self.writeln(" end");
                }
            }
        }

        self.generate_block(&decl.body);
        self.dedent();
        self.write_indent();
        self.writeln("end");

        // If in a namespace, attach the function to the namespace
        if let Some(ns_path) = &self.current_namespace {
            let ns_full_path = ns_path.join(".");
            self.namespace_exports
                .push((fn_name.clone(), ns_full_path.clone()));

            self.write_indent();
            self.writeln(&format!("{}.{} = {}", ns_full_path, fn_name, fn_name));
        }
    }

    pub fn generate_if_statement(&mut self, if_stmt: &IfStatement) {
        self.write_indent();
        self.write("if ");
        self.generate_expression(&if_stmt.condition);
        self.writeln(" then");
        self.indent();
        self.generate_block(&if_stmt.then_block);
        self.dedent();

        for else_if in if_stmt.else_ifs.iter() {
            self.write_indent();
            self.write("elseif ");
            self.generate_expression(&else_if.condition);
            self.writeln(" then");
            self.indent();
            self.generate_block(&else_if.block);
            self.dedent();
        }

        if let Some(else_block) = &if_stmt.else_block {
            self.write_indent();
            self.writeln("else");
            self.indent();
            self.generate_block(else_block);
            self.dedent();
        }

        self.write_indent();
        self.writeln("end");
    }

    pub fn generate_while_statement(&mut self, while_stmt: &WhileStatement) {
        let has_continue = block_contains_continue(&while_stmt.body);
        self.write_indent();
        self.write("while ");
        self.generate_expression(&while_stmt.condition);
        self.writeln(" do");
        self.indent();
        if has_continue
            && !self.strategy.supports_goto()
            && !self.strategy.supports_native_continue()
        {
            if block_contains_break(&while_stmt.body) {
                self.write_indent();
                self.writeln("error(\"LuaNext: continue and break in the same loop is not supported for Lua 5.1 target\")");
            }
            self.write_indent();
            self.writeln("repeat");
            self.indent();
        }
        self.generate_block(&while_stmt.body);
        if has_continue && !self.strategy.supports_native_continue() {
            if self.strategy.supports_goto() {
                self.write_indent();
                self.writeln("::__continue::");
            } else {
                self.dedent();
                self.write_indent();
                self.writeln("until true");
            }
        }
        self.dedent();
        self.write_indent();
        self.writeln("end");
    }

    pub fn generate_for_statement(&mut self, for_stmt: &ForStatement) {
        match for_stmt {
            ForStatement::Numeric(numeric) => {
                let has_continue = block_contains_continue(&numeric.body);
                self.write_indent();
                self.write("for ");
                let var_name = self.resolve(numeric.variable.node);
                self.write(&var_name);
                self.write(" = ");
                self.generate_expression(&numeric.start);
                self.write(", ");
                self.generate_expression(&numeric.end);
                if let Some(step) = &numeric.step {
                    self.write(", ");
                    self.generate_expression(step);
                }
                self.writeln(" do");
                self.indent();
                if has_continue
                    && !self.strategy.supports_goto()
                    && !self.strategy.supports_native_continue()
                {
                    if block_contains_break(&numeric.body) {
                        self.write_indent();
                        self.writeln("error(\"LuaNext: continue and break in the same loop is not supported for Lua 5.1 target\")");
                    }
                    self.write_indent();
                    self.writeln("repeat");
                    self.indent();
                }
                self.generate_block(&numeric.body);
                if has_continue && !self.strategy.supports_native_continue() {
                    if self.strategy.supports_goto() {
                        self.write_indent();
                        self.writeln("::__continue::");
                    } else {
                        self.dedent();
                        self.write_indent();
                        self.writeln("until true");
                    }
                }
                self.dedent();
                self.write_indent();
                self.writeln("end");
            }
            ForStatement::Generic(generic) => {
                let has_continue = block_contains_continue(&generic.body);
                if let Some(pattern) = &generic.pattern {
                    // Destructuring for loop: for [a, b] in items do ... end
                    // Desugars to: for _, __item in ipairs(items) do local a = __item[1] ... end
                    self.write_indent();
                    self.write("for _, __item in ipairs(");
                    for (i, iter) in generic.iterators.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.generate_expression(iter);
                    }
                    self.writeln(") do");
                    self.indent();
                    // Generate destructuring assignments at top of loop body
                    // For-loop variables are always local
                    match pattern {
                        Pattern::Array(array_pattern) => {
                            self.generate_array_destructuring(
                                array_pattern,
                                "__item",
                                &VariableKind::Local,
                            );
                        }
                        Pattern::Object(obj_pattern) => {
                            self.generate_object_destructuring(
                                obj_pattern,
                                "__item",
                                &VariableKind::Local,
                            );
                        }
                        _ => {}
                    }
                    if has_continue
                        && !self.strategy.supports_goto()
                        && !self.strategy.supports_native_continue()
                    {
                        if block_contains_break(&generic.body) {
                            self.write_indent();
                            self.writeln("error(\"LuaNext: continue and break in the same loop is not supported for Lua 5.1 target\")");
                        }
                        self.write_indent();
                        self.writeln("repeat");
                        self.indent();
                    }
                    self.generate_block(&generic.body);
                    if has_continue && !self.strategy.supports_native_continue() {
                        if self.strategy.supports_goto() {
                            self.write_indent();
                            self.writeln("::__continue::");
                        } else {
                            self.dedent();
                            self.write_indent();
                            self.writeln("until true");
                        }
                    }
                    self.dedent();
                    self.write_indent();
                    self.writeln("end");
                } else {
                    self.write_indent();
                    self.write("for ");
                    for (i, var) in generic.variables.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        let var_name = self.resolve(var.node);
                        self.write(&var_name);
                    }
                    self.write(" in ");
                    for (i, iter) in generic.iterators.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.generate_expression(iter);
                    }
                    self.writeln(" do");
                    self.indent();
                    if has_continue
                        && !self.strategy.supports_goto()
                        && !self.strategy.supports_native_continue()
                    {
                        if block_contains_break(&generic.body) {
                            self.write_indent();
                            self.writeln("error(\"LuaNext: continue and break in the same loop is not supported for Lua 5.1 target\")");
                        }
                        self.write_indent();
                        self.writeln("repeat");
                        self.indent();
                    }
                    self.generate_block(&generic.body);
                    if has_continue && !self.strategy.supports_native_continue() {
                        if self.strategy.supports_goto() {
                            self.write_indent();
                            self.writeln("::__continue::");
                        } else {
                            self.dedent();
                            self.write_indent();
                            self.writeln("until true");
                        }
                    }
                    self.dedent();
                    self.write_indent();
                    self.writeln("end");
                }
            }
        }
    }

    pub fn generate_repeat_statement(&mut self, repeat_stmt: &RepeatStatement) {
        let has_continue = block_contains_continue(&repeat_stmt.body);
        self.write_indent();
        self.writeln("repeat");
        self.indent();
        if has_continue
            && !self.strategy.supports_goto()
            && !self.strategy.supports_native_continue()
        {
            if block_contains_break(&repeat_stmt.body) {
                self.write_indent();
                self.writeln("error(\"LuaNext: continue and break in the same loop is not supported for Lua 5.1 target\")");
            }
            self.write_indent();
            self.writeln("repeat");
            self.indent();
        }
        self.generate_block(&repeat_stmt.body);
        if has_continue && !self.strategy.supports_native_continue() {
            if self.strategy.supports_goto() {
                self.write_indent();
                self.writeln("::__continue::");
            } else {
                self.dedent();
                self.write_indent();
                self.writeln("until true");
            }
        }
        self.dedent();
        self.write_indent();
        self.write("until ");
        self.generate_expression(&repeat_stmt.until);
        self.writeln("");
    }

    pub fn generate_return_statement(&mut self, return_stmt: &ReturnStatement) {
        self.write_indent();
        self.write("return");
        if !return_stmt.values.is_empty() {
            self.write(" ");
            for (i, value) in return_stmt.values.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.generate_expression(value);
            }
        }
        self.writeln("");
    }

    pub fn generate_block(&mut self, block: &Block) {
        // Emit forward declarations for all classes in this block.
        // This enables mutual recursion: class A can reference class B
        // even if B is defined later in the same block.
        self.emit_class_forward_declarations(block);

        for statement in block.statements.iter() {
            self.generate_statement(statement);
        }
    }

    /// Pre-scan a block for class declarations and emit `local ClassName`
    /// forward declarations. This allows classes to reference each other
    /// regardless of definition order.
    fn emit_class_forward_declarations(&mut self, block: &Block) {
        let class_names: Vec<String> = block
            .statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Class(class_decl) = stmt {
                    Some(self.resolve(class_decl.name.node).to_string())
                } else {
                    None
                }
            })
            .collect();

        // Only emit forward declarations when there are 2+ classes
        // (single class can't have mutual recursion)
        if class_names.len() >= 2 {
            for name in &class_names {
                self.write_indent();
                self.writeln(&format!("local {name}"));
                self.forward_declared_classes.insert(name.clone());
            }
        }
    }

    /// Pre-scan top-level statements for class declarations and emit forward
    /// declarations. Same logic as `emit_class_forward_declarations` but works
    /// with a slice of statements (top-level program has no Block wrapper).
    pub fn emit_top_level_class_forward_declarations(&mut self, statements: &[Statement]) {
        let class_names: Vec<String> = statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::Class(class_decl) = stmt {
                    Some(self.resolve(class_decl.name.node).to_string())
                } else {
                    None
                }
            })
            .collect();

        if class_names.len() >= 2 {
            for name in &class_names {
                self.write_indent();
                self.writeln(&format!("local {name}"));
                self.forward_declared_classes.insert(name.clone());
            }
        }
    }

    pub fn generate_throw_statement(
        &mut self,
        stmt: &luanext_parser::ast::statement::ThrowStatement,
    ) {
        self.write_indent();
        self.write("error(");
        self.generate_expression(&stmt.expression);
        self.writeln(")");
    }

    pub fn generate_rethrow_statement(&mut self, _span: luanext_parser::span::Span) {
        self.write_indent();
        self.writeln("error(__error)");
    }

    pub fn generate_try_statement(&mut self, stmt: &luanext_parser::ast::statement::TryStatement) {
        self.write_indent();
        self.writeln("-- try block");

        let has_typed_catches = stmt.catch_clauses.iter().any(|clause| {
            matches!(
                clause.pattern,
                luanext_parser::ast::statement::CatchPattern::Typed { .. }
                    | luanext_parser::ast::statement::CatchPattern::MultiTyped { .. }
            )
        });

        let has_finally = stmt.finally_block.is_some();
        let needs_xpcall = has_typed_catches || has_finally;
        let prefer_xpcall = matches!(
            self.optimization_level,
            OptimizationLevel::Moderate | OptimizationLevel::Aggressive
        );

        if needs_xpcall || prefer_xpcall {
            self.generate_try_xpcall(stmt);
        } else {
            self.generate_try_pcall(stmt);
        }
    }

    pub fn generate_try_pcall(&mut self, stmt: &luanext_parser::ast::statement::TryStatement) {
        self.write_indent();
        self.writeln("local __ok, __result = pcall(function()");

        self.indent();
        self.generate_block(&stmt.try_block);
        self.dedent();

        self.write_indent();
        self.writeln("end)");

        self.write_indent();
        self.writeln("if not __ok then");

        self.indent();
        self.write_indent();
        self.writeln("local __error = __result");

        let catch_count = stmt.catch_clauses.len();
        for (i, catch_clause) in stmt.catch_clauses.iter().enumerate() {
            self.generate_catch_clause_pcall(catch_clause, i == catch_count - 1);
        }
        self.dedent();

        self.write_indent();
        self.writeln("end");

        if let Some(finally_block) = &stmt.finally_block {
            self.generate_finally_block(finally_block);
        }
    }

    pub fn generate_try_xpcall(&mut self, stmt: &luanext_parser::ast::statement::TryStatement) {
        self.write_indent();
        self.writeln("local __error");
        self.write_indent();
        self.writeln("xpcall(function()");

        self.indent();
        self.generate_block(&stmt.try_block);
        self.dedent();

        self.write_indent();
        self.writeln("end, ");

        let has_typed_catches = stmt.catch_clauses.iter().any(|clause| {
            matches!(
                clause.pattern,
                luanext_parser::ast::statement::CatchPattern::Typed { .. }
                    | luanext_parser::ast::statement::CatchPattern::MultiTyped { .. }
            )
        });

        let use_debug_traceback = matches!(
            self.optimization_level,
            OptimizationLevel::Moderate | OptimizationLevel::Aggressive
        ) && !has_typed_catches;

        if use_debug_traceback {
            self.writeln("debug.traceback)");
        } else {
            self.writeln("function(__err)");
            self.indent();
            self.write_indent();
            self.writeln("__error = __err");

            for catch_clause in stmt.catch_clauses.iter() {
                self.generate_catch_clause_xpcall(catch_clause);
            }

            self.dedent();
            self.write_indent();
            self.writeln("end)");
        }

        if use_debug_traceback {
            self.write_indent();
            self.writeln("if __error == nil then return end");
            self.write_indent();
            self.writeln("local e = __error");
        }

        for catch_clause in stmt.catch_clauses.iter() {
            self.generate_block(&catch_clause.body);
        }

        if let Some(finally_block) = &stmt.finally_block {
            self.generate_finally_block(finally_block);
        }
    }

    pub fn generate_catch_clause_pcall(
        &mut self,
        clause: &luanext_parser::ast::statement::CatchClause,
        _is_last: bool,
    ) {
        let var_name = match &clause.pattern {
            luanext_parser::ast::statement::CatchPattern::Untyped { variable, .. }
            | luanext_parser::ast::statement::CatchPattern::Typed { variable, .. }
            | luanext_parser::ast::statement::CatchPattern::MultiTyped { variable, .. } => {
                self.resolve(variable.node)
            }
        };

        self.write_indent();
        self.writeln(&format!("local {} = __error", var_name));
        self.generate_block(&clause.body);
    }

    pub fn generate_catch_clause_xpcall(
        &mut self,
        clause: &luanext_parser::ast::statement::CatchClause,
    ) {
        let var_name = match &clause.pattern {
            luanext_parser::ast::statement::CatchPattern::Untyped { variable, .. }
            | luanext_parser::ast::statement::CatchPattern::Typed { variable, .. }
            | luanext_parser::ast::statement::CatchPattern::MultiTyped { variable, .. } => {
                self.resolve(variable.node)
            }
        };

        self.write_indent();
        self.writeln(&format!("if {} == nil then", var_name));
        self.indent();
        self.write_indent();
        self.writeln("return false");
        self.dedent();
        self.write_indent();
        self.writeln("end");
    }

    pub fn generate_finally_block(&mut self, block: &Block) {
        self.write_indent();
        self.writeln("-- finally block");
        self.generate_block(block);
    }
}

/// Check if a block contains a `continue` statement at the current loop level.
/// Recurses into if/else/try/block but NOT into nested loops (those continues
/// target the inner loop, not the outer one).
pub fn block_contains_continue(block: &Block) -> bool {
    for stmt in block.statements.iter() {
        match stmt {
            Statement::Continue(_) => return true,
            Statement::If(if_stmt) => {
                if block_contains_continue(&if_stmt.then_block) {
                    return true;
                }
                for else_if in if_stmt.else_ifs.iter() {
                    if block_contains_continue(&else_if.block) {
                        return true;
                    }
                }
                if let Some(else_block) = &if_stmt.else_block {
                    if block_contains_continue(else_block) {
                        return true;
                    }
                }
            }
            Statement::Block(inner_block) => {
                if block_contains_continue(inner_block) {
                    return true;
                }
            }
            Statement::Try(try_stmt) => {
                if block_contains_continue(&try_stmt.try_block) {
                    return true;
                }
                for catch in try_stmt.catch_clauses.iter() {
                    if block_contains_continue(&catch.body) {
                        return true;
                    }
                }
                if let Some(finally_block) = &try_stmt.finally_block {
                    if block_contains_continue(finally_block) {
                        return true;
                    }
                }
            }
            // Do NOT recurse into nested loops
            Statement::While(_) | Statement::For(_) | Statement::Repeat(_) => {}
            _ => {}
        }
    }
    false
}

/// Check if a block contains a `break` statement at the current loop level.
/// Same recursion rules as `block_contains_continue`.
pub fn block_contains_break(block: &Block) -> bool {
    for stmt in block.statements.iter() {
        match stmt {
            Statement::Break(_) => return true,
            Statement::If(if_stmt) => {
                if block_contains_break(&if_stmt.then_block) {
                    return true;
                }
                for else_if in if_stmt.else_ifs.iter() {
                    if block_contains_break(&else_if.block) {
                        return true;
                    }
                }
                if let Some(else_block) = &if_stmt.else_block {
                    if block_contains_break(else_block) {
                        return true;
                    }
                }
            }
            Statement::Block(inner_block) => {
                if block_contains_break(inner_block) {
                    return true;
                }
            }
            Statement::Try(try_stmt) => {
                if block_contains_break(&try_stmt.try_block) {
                    return true;
                }
                for catch in try_stmt.catch_clauses.iter() {
                    if block_contains_break(&catch.body) {
                        return true;
                    }
                }
                if let Some(finally_block) = &try_stmt.finally_block {
                    if block_contains_break(finally_block) {
                        return true;
                    }
                }
            }
            // Do NOT recurse into nested loops
            Statement::While(_) | Statement::For(_) | Statement::Repeat(_) => {}
            _ => {}
        }
    }
    false
}
