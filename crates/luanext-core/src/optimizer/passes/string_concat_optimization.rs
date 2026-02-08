// =============================================================================
// O2: String Concatenation Optimization Pass
// =============================================================================

use crate::config::OptimizationLevel;
use crate::optimizer::{ExprVisitor, WholeProgramPass};
use crate::MutableProgram;
use bumpalo::Bump;
use std::sync::Arc;
use typedlua_parser::ast::expression::{
    Argument, ArrayElement, AssignmentOp, BinaryOp, Expression, ExpressionKind, Literal,
};
use typedlua_parser::ast::pattern::Pattern;
use typedlua_parser::ast::statement::{
    Block, ForStatement, Statement, VariableDeclaration, VariableKind,
};
use typedlua_parser::ast::Spanned;
use typedlua_parser::span::Span;
use typedlua_parser::string_interner::{StringId, StringInterner};

const MIN_CONCAT_PARTS_FOR_OPTIMIZATION: usize = 3;

pub struct StringConcatOptimizationPass {
    next_temp_id: usize,
    interner: Arc<StringInterner>,
}

impl StringConcatOptimizationPass {
    pub fn new(interner: Arc<StringInterner>) -> Self {
        Self {
            next_temp_id: 0,
            interner,
        }
    }
}

impl<'arena> ExprVisitor<'arena> for StringConcatOptimizationPass {
    fn visit_expr(&mut self, expr: &mut Expression<'arena>, arena: &'arena Bump) -> bool {
        // Check if this is a concat expression that can be optimized
        if let ExpressionKind::Binary(BinaryOp::Concatenate, _left, _right) = &expr.kind {
            let parts = self.flatten_concat_chain(expr);
            if parts.len() >= MIN_CONCAT_PARTS_FOR_OPTIMIZATION {
                self.replace_with_table_concat(expr, &parts, arena);
                return true;
            }
        }
        false
    }
}

impl<'arena> WholeProgramPass<'arena> for StringConcatOptimizationPass {
    fn name(&self) -> &'static str {
        "string-concat-optimization"
    }

    fn min_level(&self) -> OptimizationLevel {
        OptimizationLevel::O2
    }

    fn run(
        &mut self,
        program: &mut MutableProgram<'arena>,
        arena: &'arena Bump,
    ) -> Result<bool, String> {
        self.next_temp_id = 0;

        let mut changed = false;
        let mut i = 0;
        while i < program.statements.len() {
            if self.optimize_statement(&mut program.statements[i], arena) {
                changed = true;
            }
            i += 1;
        }

        // Also optimize loop-based string concatenation patterns
        if self.optimize_loop_string_concat(&mut program.statements, arena) {
            changed = true;
        }

        Ok(changed)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl StringConcatOptimizationPass {
    fn optimize_statement<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match stmt {
            Statement::Variable(decl) => self.optimize_concat_in_variable(decl, arena),
            Statement::Expression(expr) => self.optimize_concat_in_expression(expr, arena),
            Statement::Function(func) => {
                let mut changed = false;
                let mut stmts: Vec<Statement<'arena>> = func.body.statements.to_vec();
                let mut body_changed = false;
                for s in &mut stmts {
                    if self.optimize_statement(s, arena) {
                        body_changed = true;
                    }
                }
                if body_changed {
                    func.body.statements = arena.alloc_slice_clone(&stmts);
                    changed = true;
                }
                changed
            }
            Statement::Return(ret) => {
                let mut changed = false;
                let mut values: Vec<Expression<'arena>> = ret.values.to_vec();
                let mut vals_changed = false;
                for expr in &mut values {
                    if self.optimize_concat_expression(expr, arena) {
                        vals_changed = true;
                    }
                }
                if vals_changed {
                    ret.values = arena.alloc_slice_clone(&values);
                    changed = true;
                }
                changed
            }
            Statement::If(if_stmt) => {
                let mut changed = false;
                // then_block
                let mut then_stmts: Vec<Statement<'arena>> = if_stmt.then_block.statements.to_vec();
                let mut then_changed = false;
                for s in &mut then_stmts {
                    if self.optimize_statement(s, arena) {
                        then_changed = true;
                    }
                }
                if then_changed {
                    if_stmt.then_block.statements = arena.alloc_slice_clone(&then_stmts);
                    changed = true;
                }
                // else_ifs
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for else_if in &mut new_else_ifs {
                    let mut ei_stmts: Vec<Statement<'arena>> = else_if.block.statements.to_vec();
                    let mut ei_changed = false;
                    for s in &mut ei_stmts {
                        if self.optimize_statement(s, arena) {
                            ei_changed = true;
                        }
                    }
                    if ei_changed {
                        else_if.block.statements = arena.alloc_slice_clone(&ei_stmts);
                        eic = true;
                    }
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                    changed = true;
                }
                // else_block
                if let Some(else_block) = &mut if_stmt.else_block {
                    let mut else_stmts: Vec<Statement<'arena>> = else_block.statements.to_vec();
                    let mut else_changed = false;
                    for s in &mut else_stmts {
                        if self.optimize_statement(s, arena) {
                            else_changed = true;
                        }
                    }
                    if else_changed {
                        else_block.statements = arena.alloc_slice_clone(&else_stmts);
                        changed = true;
                    }
                }
                changed
            }
            Statement::While(while_stmt) => {
                let mut changed = false;
                let mut stmts: Vec<Statement<'arena>> = while_stmt.body.statements.to_vec();
                let mut body_changed = false;
                for s in &mut stmts {
                    if self.optimize_statement(s, arena) {
                        body_changed = true;
                    }
                }
                if body_changed {
                    while_stmt.body.statements = arena.alloc_slice_clone(&stmts);
                    changed = true;
                }
                changed
            }
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Generic(for_gen_ref) => {
                    let mut new_gen = for_gen_ref.clone();
                    let mut stmts: Vec<Statement<'arena>> = new_gen.body.statements.to_vec();
                    let mut body_changed = false;
                    for s in &mut stmts {
                        if self.optimize_statement(s, arena) {
                            body_changed = true;
                        }
                    }
                    if body_changed {
                        new_gen.body.statements = arena.alloc_slice_clone(&stmts);
                        *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                        true
                    } else {
                        false
                    }
                }
                ForStatement::Numeric(for_num_ref) => {
                    let mut new_num = (**for_num_ref).clone();
                    let mut stmts: Vec<Statement<'arena>> = new_num.body.statements.to_vec();
                    let mut body_changed = false;
                    for s in &mut stmts {
                        if self.optimize_statement(s, arena) {
                            body_changed = true;
                        }
                    }
                    if body_changed {
                        new_num.body.statements = arena.alloc_slice_clone(&stmts);
                        *stmt = Statement::For(
                            arena.alloc(ForStatement::Numeric(arena.alloc(new_num))),
                        );
                        true
                    } else {
                        false
                    }
                }
            },
            Statement::Repeat(repeat_stmt) => {
                let mut changed = false;
                let mut stmts: Vec<Statement<'arena>> = repeat_stmt.body.statements.to_vec();
                let mut body_changed = false;
                for s in &mut stmts {
                    if self.optimize_statement(s, arena) {
                        body_changed = true;
                    }
                }
                if body_changed {
                    repeat_stmt.body.statements = arena.alloc_slice_clone(&stmts);
                    changed = true;
                }
                changed
            }
            _ => false,
        }
    }

    fn optimize_concat_in_variable<'arena>(
        &mut self,
        decl: &mut VariableDeclaration<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        self.optimize_concat_expression(&mut decl.initializer, arena)
    }

    /// Optimizes loop-based string concatenation patterns
    /// Transforms: local s = ""; for ... do s = s .. value end
    /// Into: local t = {}; for ... do table.insert(t, value) end; local s = table.concat(t)
    fn optimize_loop_string_concat<'arena>(
        &mut self,
        statements: &mut Vec<Statement<'arena>>,
        arena: &'arena Bump,
    ) -> bool {
        let mut changed = false;
        let mut i = 0;

        while i < statements.len() {
            // Look for pattern: local s = "" followed by a loop with s = s .. value
            if let Some((concat_var, loop_idx)) =
                self.find_loop_string_concat_pattern(statements, i)
            {
                // Transform the pattern
                if let Some(new_stmts) =
                    self.transform_loop_string_concat(statements, i, loop_idx, concat_var, arena)
                {
                    // Replace the original statements with transformed ones
                    statements.splice(i..=loop_idx, new_stmts);
                    changed = true;
                    continue;
                }
            }
            i += 1;
        }

        changed
    }

    /// Finds the pattern: local s = "" followed by a loop containing s = s .. value
    fn find_loop_string_concat_pattern<'arena>(
        &self,
        statements: &[Statement<'arena>],
        start_idx: usize,
    ) -> Option<(StringId, usize)> {
        // Check for local s = "" at start_idx
        let concat_var = if let Statement::Variable(decl) = &statements[start_idx] {
            if let Pattern::Identifier(ident) = &decl.pattern {
                if let ExpressionKind::Literal(Literal::String(s)) = &decl.initializer.kind {
                    if s.is_empty() {
                        Some(ident.node)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }?;

        // Look for a loop at start_idx + 1 that contains s = s .. value
        if start_idx + 1 < statements.len() {
            let loop_stmt = &statements[start_idx + 1];
            if self.loop_contains_string_concat(loop_stmt, concat_var) {
                return Some((concat_var, start_idx + 1));
            }
        }

        None
    }

    /// Checks if a loop statement contains string concatenation on the given variable
    fn loop_contains_string_concat<'arena>(&self, stmt: &Statement<'arena>, var: StringId) -> bool {
        match stmt {
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Generic(for_gen) => {
                    self.block_contains_string_concat(&for_gen.body, var)
                }
                ForStatement::Numeric(for_num) => {
                    self.block_contains_string_concat(&for_num.body, var)
                }
            },
            Statement::While(while_stmt) => {
                self.block_contains_string_concat(&while_stmt.body, var)
            }
            Statement::Repeat(repeat_stmt) => {
                self.block_contains_string_concat(&repeat_stmt.body, var)
            }
            _ => false,
        }
    }

    /// Checks if a block contains string concatenation on the given variable
    fn block_contains_string_concat<'arena>(&self, block: &Block<'arena>, var: StringId) -> bool {
        block
            .statements
            .iter()
            .any(|stmt| self.statement_contains_string_concat(stmt, var))
    }

    /// Checks if a statement contains string concatenation on the given variable
    fn statement_contains_string_concat<'arena>(
        &self,
        stmt: &Statement<'arena>,
        var: StringId,
    ) -> bool {
        match stmt {
            Statement::Expression(expr) => self.expression_is_string_concat(expr, var),
            Statement::Block(block) => self.block_contains_string_concat(block, var),
            Statement::If(if_stmt) => {
                self.block_contains_string_concat(&if_stmt.then_block, var)
                    || if_stmt
                        .else_ifs
                        .iter()
                        .any(|ei| self.block_contains_string_concat(&ei.block, var))
                    || if_stmt
                        .else_block
                        .as_ref()
                        .is_some_and(|b| self.block_contains_string_concat(b, var))
            }
            _ => false,
        }
    }

    /// Checks if an expression is s = s .. value or s ..= value
    fn expression_is_string_concat<'arena>(
        &self,
        expr: &Expression<'arena>,
        var: StringId,
    ) -> bool {
        match &expr.kind {
            ExpressionKind::Assignment(target, AssignmentOp::Assign, value) => {
                // Check if target is the variable
                if let ExpressionKind::Identifier(target_id) = &target.kind {
                    if *target_id == var {
                        // Check if value is var .. something
                        if let ExpressionKind::Binary(BinaryOp::Concatenate, left, _) = &value.kind
                        {
                            if let ExpressionKind::Identifier(left_id) = &left.kind {
                                return *left_id == var;
                            }
                        }
                    }
                }
                false
            }
            ExpressionKind::Assignment(target, AssignmentOp::ConcatenateAssign, _) => {
                // Check if target is the variable
                if let ExpressionKind::Identifier(target_id) = &target.kind {
                    *target_id == var
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Transforms the loop-based string concatenation pattern
    fn transform_loop_string_concat<'arena>(
        &mut self,
        statements: &[Statement<'arena>],
        _var_decl_idx: usize,
        loop_idx: usize,
        concat_var: StringId,
        arena: &'arena Bump,
    ) -> Option<Vec<Statement<'arena>>> {
        let temp_table_var = self.next_temp_id;
        self.next_temp_id += 1;
        let temp_table_name = format!("__str_concat_{}", temp_table_var);
        let temp_table_id = self.interner.get_or_intern(&temp_table_name);

        // Create: local __str_concat_N = {}
        let table_decl = Statement::Variable(VariableDeclaration {
            kind: VariableKind::Local,
            pattern: Pattern::Identifier(Spanned::new(temp_table_id, Span::dummy())),
            type_annotation: None,
            initializer: Expression::new(
                ExpressionKind::Array(arena.alloc_slice_clone(&[])),
                Span::dummy(),
            ),
            span: Span::dummy(),
        });

        // Clone and transform the loop
        let mut transformed_loop = statements[loop_idx].clone();
        self.transform_loop_body(&mut transformed_loop, concat_var, temp_table_id, arena);

        // Create: local s = table.concat(__str_concat_N)
        let concat_decl = Statement::Variable(VariableDeclaration {
            kind: VariableKind::Local,
            pattern: Pattern::Identifier(Spanned::new(concat_var, Span::dummy())),
            type_annotation: None,
            initializer: Expression::new(
                ExpressionKind::Call(
                    arena.alloc(Expression::new(
                        ExpressionKind::Member(
                            arena.alloc(Expression::new(
                                ExpressionKind::Identifier(self.interner.get_or_intern("table")),
                                Span::dummy(),
                            )),
                            Spanned::new(self.interner.get_or_intern("concat"), Span::dummy()),
                        ),
                        Span::dummy(),
                    )),
                    arena.alloc_slice_clone(&[Argument {
                        value: Expression::new(
                            ExpressionKind::Identifier(temp_table_id),
                            Span::dummy(),
                        ),
                        is_spread: false,
                        span: Span::dummy(),
                    }]),
                    None,
                ),
                Span::dummy(),
            ),
            span: Span::dummy(),
        });

        Some(vec![table_decl, transformed_loop, concat_decl])
    }

    /// Transforms the loop body to use table.insert instead of string concatenation
    fn transform_loop_body<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        concat_var: StringId,
        table_var: StringId,
        arena: &'arena Bump,
    ) {
        match stmt {
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Generic(for_gen_ref) => {
                    let mut new_gen = for_gen_ref.clone();
                    self.transform_block(&mut new_gen.body, concat_var, table_var, arena);
                    *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                }
                ForStatement::Numeric(for_num_ref) => {
                    let mut new_num = (**for_num_ref).clone();
                    self.transform_block(&mut new_num.body, concat_var, table_var, arena);
                    *stmt =
                        Statement::For(arena.alloc(ForStatement::Numeric(arena.alloc(new_num))));
                }
            },
            Statement::While(while_stmt) => {
                self.transform_block(&mut while_stmt.body, concat_var, table_var, arena);
            }
            Statement::Repeat(repeat_stmt) => {
                self.transform_block(&mut repeat_stmt.body, concat_var, table_var, arena);
            }
            _ => {}
        }
    }

    /// Transforms a block to use table.insert instead of string concatenation
    fn transform_block<'arena>(
        &mut self,
        block: &mut Block<'arena>,
        concat_var: StringId,
        table_var: StringId,
        arena: &'arena Bump,
    ) {
        let mut stmts: Vec<Statement<'arena>> = block.statements.to_vec();
        let mut changed = false;
        for s in &mut stmts {
            if self.transform_statement(s, concat_var, table_var, arena) {
                changed = true;
            }
        }
        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
    }

    /// Transforms a statement to use table.insert instead of string concatenation
    /// Returns true if the statement was modified.
    fn transform_statement<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        concat_var: StringId,
        table_var: StringId,
        arena: &'arena Bump,
    ) -> bool {
        match stmt {
            Statement::Expression(expr) => {
                if let Some(new_stmt) =
                    self.transform_string_concat(expr, concat_var, table_var, arena)
                {
                    *stmt = new_stmt;
                    return true;
                }
                false
            }
            Statement::Block(block) => {
                self.transform_block(block, concat_var, table_var, arena);
                // transform_block mutates in place, we don't track changes here for simplicity
                false
            }
            Statement::If(if_stmt) => {
                self.transform_block(&mut if_stmt.then_block, concat_var, table_var, arena);
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for else_if in &mut new_else_ifs {
                    let mut ei_stmts: Vec<Statement<'arena>> = else_if.block.statements.to_vec();
                    let mut ei_changed = false;
                    for s in &mut ei_stmts {
                        if self.transform_statement(s, concat_var, table_var, arena) {
                            ei_changed = true;
                        }
                    }
                    if ei_changed {
                        else_if.block.statements = arena.alloc_slice_clone(&ei_stmts);
                        eic = true;
                    }
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                }
                if let Some(else_block) = &mut if_stmt.else_block {
                    self.transform_block(else_block, concat_var, table_var, arena);
                }
                false
            }
            _ => false,
        }
    }

    /// Transforms s = s .. value or s ..= value into table.insert(t, value)
    fn transform_string_concat<'arena>(
        &mut self,
        expr: &Expression<'arena>,
        concat_var: StringId,
        table_var: StringId,
        arena: &'arena Bump,
    ) -> Option<Statement<'arena>> {
        match &expr.kind {
            ExpressionKind::Assignment(target, AssignmentOp::Assign, value) => {
                if let ExpressionKind::Identifier(target_id) = &target.kind {
                    if *target_id == concat_var {
                        if let ExpressionKind::Binary(BinaryOp::Concatenate, _, right) = &value.kind
                        {
                            // Transform s = s .. right into table.insert(t, right)
                            return Some(Statement::Expression(Expression::new(
                                ExpressionKind::Call(
                                    arena.alloc(Expression::new(
                                        ExpressionKind::Member(
                                            arena.alloc(Expression::new(
                                                ExpressionKind::Identifier(
                                                    self.interner.get_or_intern("table"),
                                                ),
                                                Span::dummy(),
                                            )),
                                            Spanned::new(
                                                self.interner.get_or_intern("insert"),
                                                Span::dummy(),
                                            ),
                                        ),
                                        Span::dummy(),
                                    )),
                                    arena.alloc_slice_clone(&[
                                        Argument {
                                            value: Expression::new(
                                                ExpressionKind::Identifier(table_var),
                                                Span::dummy(),
                                            ),
                                            is_spread: false,
                                            span: Span::dummy(),
                                        },
                                        Argument {
                                            value: (**right).clone(),
                                            is_spread: false,
                                            span: Span::dummy(),
                                        },
                                    ]),
                                    None,
                                ),
                                Span::dummy(),
                            )));
                        }
                    }
                }
                None
            }
            ExpressionKind::Assignment(target, AssignmentOp::ConcatenateAssign, right) => {
                if let ExpressionKind::Identifier(target_id) = &target.kind {
                    if *target_id == concat_var {
                        // Transform s ..= right into table.insert(t, right)
                        return Some(Statement::Expression(Expression::new(
                            ExpressionKind::Call(
                                arena.alloc(Expression::new(
                                    ExpressionKind::Member(
                                        arena.alloc(Expression::new(
                                            ExpressionKind::Identifier(
                                                self.interner.get_or_intern("table"),
                                            ),
                                            Span::dummy(),
                                        )),
                                        Spanned::new(
                                            self.interner.get_or_intern("insert"),
                                            Span::dummy(),
                                        ),
                                    ),
                                    Span::dummy(),
                                )),
                                arena.alloc_slice_clone(&[
                                    Argument {
                                        value: Expression::new(
                                            ExpressionKind::Identifier(table_var),
                                            Span::dummy(),
                                        ),
                                        is_spread: false,
                                        span: Span::dummy(),
                                    },
                                    Argument {
                                        value: (**right).clone(),
                                        is_spread: false,
                                        span: Span::dummy(),
                                    },
                                ]),
                                None,
                            ),
                            Span::dummy(),
                        )));
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn optimize_concat_in_expression<'arena>(
        &mut self,
        expr: &mut Expression<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        self.optimize_concat_expression(expr, arena)
    }

    fn optimize_concat_expression<'arena>(
        &mut self,
        expr: &mut Expression<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        if let ExpressionKind::Binary(BinaryOp::Concatenate, _left, _right) = &expr.kind {
            let parts = self.flatten_concat_chain(expr);
            if parts.len() >= MIN_CONCAT_PARTS_FOR_OPTIMIZATION {
                self.replace_with_table_concat(expr, &parts, arena);
                return true;
            }
        }
        false
    }

    fn flatten_concat_chain<'arena>(&self, expr: &Expression<'arena>) -> Vec<Expression<'arena>> {
        fn flatten_inner<'arena>(expr: &Expression<'arena>, result: &mut Vec<Expression<'arena>>) {
            match &expr.kind {
                ExpressionKind::Binary(BinaryOp::Concatenate, left, right) => {
                    flatten_inner(left, result);
                    flatten_inner(right, result);
                }
                ExpressionKind::Parenthesized(inner) => {
                    flatten_inner(inner, result);
                }
                _ => {
                    result.push(expr.clone());
                }
            }
        }
        let mut parts = Vec::new();
        flatten_inner(expr, &mut parts);
        parts
    }

    fn replace_with_table_concat<'arena>(
        &self,
        expr: &mut Expression<'arena>,
        parts: &[Expression<'arena>],
        arena: &'arena Bump,
    ) {
        let elements: Vec<ArrayElement<'arena>> = parts
            .iter()
            .map(|p| ArrayElement::Expression(p.clone()))
            .collect();

        let table_expr = Expression::new(
            ExpressionKind::Array(arena.alloc_slice_clone(&elements)),
            Span::dummy(),
        );

        let concat_call = Expression::new(
            ExpressionKind::Call(
                arena.alloc(Expression::new(
                    ExpressionKind::Member(
                        arena.alloc(Expression::new(
                            ExpressionKind::Identifier(self.interner.get_or_intern("table")),
                            Span::dummy(),
                        )),
                        Spanned::new(self.interner.get_or_intern("concat"), Span::dummy()),
                    ),
                    Span::dummy(),
                )),
                arena.alloc_slice_clone(&[Argument {
                    value: table_expr,
                    is_spread: false,
                    span: Span::dummy(),
                }]),
                None,
            ),
            Span::dummy(),
        );

        *expr = concat_call;
    }
}

impl Default for StringConcatOptimizationPass {
    fn default() -> Self {
        Self::new(Arc::new(StringInterner::new()))
    }
}
