//! Class hierarchy analysis for devirtualization
//!
//! Provides class hierarchy information used for cross-module optimizations.
//! The actual devirtualization pass is temporarily disabled during arena migration
//! (see devirtualization.rs.pre_arena for the full implementation).

use luanext_parser::ast::expression::{Expression, ExpressionKind};
use luanext_parser::ast::statement::{Block, ClassMember, Statement};
use luanext_parser::ast::types::TypeKind;
use luanext_parser::ast::Program;
use luanext_parser::string_interner::StringId;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use tracing::debug;

use ExpressionKind::*;

/// Class hierarchy information for devirtualization safety analysis
#[derive(Debug, Default, Clone)]
pub struct ClassHierarchy {
    /// class -> parent (None if no extends)
    parent_of: FxHashMap<StringId, Option<StringId>>,
    /// parent -> list of direct children
    children_of: FxHashMap<StringId, Vec<StringId>>,
    /// class -> is_final
    is_final: FxHashMap<StringId, bool>,
    /// (class, method) -> is_final
    final_methods: FxHashMap<(StringId, StringId), bool>,
    /// (class, method) -> method is declared here (not inherited)
    declares_method: FxHashMap<(StringId, StringId), bool>,
    /// Set of all known class names (to distinguish from interfaces)
    known_classes: FxHashMap<StringId, bool>,
    /// RTA: class -> set of subclasses that are instantiated
    instantiated_subclasses: FxHashMap<StringId, FxHashSet<StringId>>,
    /// RTA: For each class, the single instantiated subclass if there's exactly one
    single_instantiated_subclass: FxHashMap<StringId, StringId>,
    /// RTA: Total count of instantiations per class
    instantiation_counts: FxHashMap<StringId, usize>,
    /// RTA: Set of all classes that have any instantiations
    classes_with_instantiations: FxHashSet<StringId>,
}

impl ClassHierarchy {
    /// Build class hierarchy by scanning all class declarations in the program
    pub fn build<'arena>(program: &Program<'arena>) -> Self {
        let mut hierarchy = ClassHierarchy::default();

        for stmt in program.statements.iter() {
            if let Statement::Class(class) = stmt {
                let class_id = class.name.node;
                hierarchy.known_classes.insert(class_id, true);
                hierarchy.is_final.insert(class_id, class.is_final);

                let parent_id = class.extends.as_ref().and_then(|ext| {
                    if let TypeKind::Reference(type_ref) = &ext.kind {
                        Some(type_ref.name.node)
                    } else {
                        None
                    }
                });
                hierarchy.parent_of.insert(class_id, parent_id);

                if let Some(parent) = parent_id {
                    hierarchy
                        .children_of
                        .entry(parent)
                        .or_default()
                        .push(class_id);
                }

                for member in class.members.iter() {
                    if let ClassMember::Method(method) = member {
                        let method_id = method.name.node;
                        hierarchy
                            .declares_method
                            .insert((class_id, method_id), true);
                        if method.is_final {
                            hierarchy.final_methods.insert((class_id, method_id), true);
                        }
                    }
                }
            }
        }

        hierarchy
    }

    /// Build class hierarchy by scanning all class declarations across multiple modules
    pub fn build_multi_module<'arena>(programs: &[&Program<'arena>]) -> Self {
        let mut hierarchy = ClassHierarchy::default();

        for program in programs {
            for stmt in program.statements.iter() {
                if let Statement::Class(class) = stmt {
                    let class_id = class.name.node;
                    hierarchy.known_classes.insert(class_id, true);
                    hierarchy.is_final.insert(class_id, class.is_final);

                    let parent_id = class.extends.as_ref().and_then(|ext| {
                        if let TypeKind::Reference(type_ref) = &ext.kind {
                            Some(type_ref.name.node)
                        } else {
                            None
                        }
                    });
                    hierarchy.parent_of.insert(class_id, parent_id);

                    if let Some(parent) = parent_id {
                        hierarchy
                            .children_of
                            .entry(parent)
                            .or_default()
                            .push(class_id);
                    }

                    for member in class.members.iter() {
                        if let ClassMember::Method(method) = member {
                            let method_id = method.name.node;
                            hierarchy
                                .declares_method
                                .insert((class_id, method_id), true);
                            if method.is_final {
                                hierarchy.final_methods.insert((class_id, method_id), true);
                            }
                        }
                    }
                }
            }
        }

        // Second pass: collect all instantiations for RTA
        for program in programs {
            hierarchy.collect_instantiations(program);
        }

        // Compute single instantiated subclass for each base class
        hierarchy.compute_single_instantiated_subclasses();

        hierarchy
    }

    /// Collect all `new ClassName()` instantiations from a program
    fn collect_instantiations<'arena>(&mut self, program: &Program<'arena>) {
        for stmt in program.statements.iter() {
            self.collect_instantiations_from_statement(stmt);
        }
    }

    fn collect_instantiations_from_statement<'arena>(&mut self, stmt: &Statement<'arena>) {
        use luanext_parser::ast::statement::ForStatement;

        match stmt {
            Statement::Function(func) => {
                for s in func.body.statements.iter() {
                    self.collect_instantiations_from_statement(s);
                }
            }
            Statement::If(if_stmt) => {
                self.collect_instantiations_from_expression(&if_stmt.condition);
                for s in if_stmt.then_block.statements.iter() {
                    self.collect_instantiations_from_statement(s);
                }
                for else_if in if_stmt.else_ifs.iter() {
                    self.collect_instantiations_from_expression(&else_if.condition);
                    for s in else_if.block.statements.iter() {
                        self.collect_instantiations_from_statement(s);
                    }
                }
                if let Some(else_block) = &if_stmt.else_block {
                    for s in else_block.statements.iter() {
                        self.collect_instantiations_from_statement(s);
                    }
                }
            }
            Statement::While(while_stmt) => {
                self.collect_instantiations_from_expression(&while_stmt.condition);
                for s in while_stmt.body.statements.iter() {
                    self.collect_instantiations_from_statement(s);
                }
            }
            Statement::For(for_stmt) => match for_stmt {
                ForStatement::Numeric(for_num) => {
                    self.collect_instantiations_from_expression(&for_num.start);
                    self.collect_instantiations_from_expression(&for_num.end);
                    if let Some(step) = &for_num.step {
                        self.collect_instantiations_from_expression(step);
                    }
                    for s in for_num.body.statements.iter() {
                        self.collect_instantiations_from_statement(s);
                    }
                }
                ForStatement::Generic(for_gen) => {
                    for expr in for_gen.iterators.iter() {
                        self.collect_instantiations_from_expression(expr);
                    }
                    for s in for_gen.body.statements.iter() {
                        self.collect_instantiations_from_statement(s);
                    }
                }
            },
            Statement::Repeat(repeat_stmt) => {
                self.collect_instantiations_from_expression(&repeat_stmt.until);
                for s in repeat_stmt.body.statements.iter() {
                    self.collect_instantiations_from_statement(s);
                }
            }
            Statement::Return(ret) => {
                for expr in ret.values.iter() {
                    self.collect_instantiations_from_expression(expr);
                }
            }
            Statement::Variable(var) => {
                self.collect_instantiations_from_expression(&var.initializer);
            }
            Statement::Expression(expr) => {
                self.collect_instantiations_from_expression(expr);
            }
            Statement::Block(block) => {
                for s in block.statements.iter() {
                    self.collect_instantiations_from_statement(s);
                }
            }
            Statement::Class(class) => {
                for member in class.members.iter() {
                    if let ClassMember::Method(method) = member {
                        if let Some(body) = &method.body {
                            for s in body.statements.iter() {
                                self.collect_instantiations_from_statement(s);
                            }
                        }
                    }
                    if let ClassMember::Constructor(ctor) = member {
                        for s in ctor.body.statements.iter() {
                            self.collect_instantiations_from_statement(s);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn collect_instantiations_from_expression<'arena>(&mut self, expr: &Expression<'arena>) {
        match &expr.kind {
            New(callee, args, _) => {
                self.record_instantiation_from_expression(callee);
                for arg in args.iter() {
                    self.collect_instantiations_from_expression(&arg.value);
                }
            }
            Call(func, args, _) => {
                self.collect_instantiations_from_expression(func);
                for arg in args.iter() {
                    self.collect_instantiations_from_expression(&arg.value);
                }
            }
            MethodCall(obj, _name, args, _) => {
                self.collect_instantiations_from_expression(obj);
                for arg in args.iter() {
                    self.collect_instantiations_from_expression(&arg.value);
                }
            }
            Binary(_op, left, right) => {
                self.collect_instantiations_from_expression(left);
                self.collect_instantiations_from_expression(right);
            }
            Unary(_op, operand) => {
                self.collect_instantiations_from_expression(operand);
            }
            Assignment(left, _op, right) => {
                self.collect_instantiations_from_expression(left);
                self.collect_instantiations_from_expression(right);
            }
            Conditional(cond, then_expr, else_expr) => {
                self.collect_instantiations_from_expression(cond);
                self.collect_instantiations_from_expression(then_expr);
                self.collect_instantiations_from_expression(else_expr);
            }
            Pipe(left, right) => {
                self.collect_instantiations_from_expression(left);
                self.collect_instantiations_from_expression(right);
            }
            Match(match_expr) => {
                self.collect_instantiations_from_expression(match_expr.value);
                for arm in match_expr.arms.iter() {
                    match &arm.body {
                        luanext_parser::ast::expression::MatchArmBody::Expression(e) => {
                            self.collect_instantiations_from_expression(e);
                        }
                        luanext_parser::ast::expression::MatchArmBody::Block(block) => {
                            for s in block.statements.iter() {
                                self.collect_instantiations_from_statement(s);
                            }
                        }
                    }
                }
            }
            Arrow(arrow) => {
                for param in arrow.parameters.iter() {
                    if let Some(default) = &param.default {
                        self.collect_instantiations_from_expression(default);
                    }
                }
                match &arrow.body {
                    luanext_parser::ast::expression::ArrowBody::Expression(e) => {
                        self.collect_instantiations_from_expression(e);
                    }
                    luanext_parser::ast::expression::ArrowBody::Block(block) => {
                        for s in block.statements.iter() {
                            self.collect_instantiations_from_statement(s);
                        }
                    }
                }
            }
            Try(try_expr) => {
                self.collect_instantiations_from_expression(try_expr.expression);
                self.collect_instantiations_from_expression(try_expr.catch_expression);
            }
            ErrorChain(left, right) => {
                self.collect_instantiations_from_expression(left);
                self.collect_instantiations_from_expression(right);
            }
            OptionalMember(obj, _) => {
                self.collect_instantiations_from_expression(obj);
            }
            OptionalIndex(obj, index) => {
                self.collect_instantiations_from_expression(obj);
                self.collect_instantiations_from_expression(index);
            }
            OptionalCall(obj, args, _) => {
                self.collect_instantiations_from_expression(obj);
                for arg in args.iter() {
                    self.collect_instantiations_from_expression(&arg.value);
                }
            }
            OptionalMethodCall(obj, _name, args, _) => {
                self.collect_instantiations_from_expression(obj);
                for arg in args.iter() {
                    self.collect_instantiations_from_expression(&arg.value);
                }
            }
            Member(obj, _) => {
                self.collect_instantiations_from_expression(obj);
            }
            Index(obj, index) => {
                self.collect_instantiations_from_expression(obj);
                self.collect_instantiations_from_expression(index);
            }
            Array(elements) => {
                for elem in elements.iter() {
                    match elem {
                        luanext_parser::ast::expression::ArrayElement::Expression(e) => {
                            self.collect_instantiations_from_expression(e);
                        }
                        luanext_parser::ast::expression::ArrayElement::Spread(e) => {
                            self.collect_instantiations_from_expression(e);
                        }
                    }
                }
            }
            Object(props) => {
                for prop in props.iter() {
                    match prop {
                        luanext_parser::ast::expression::ObjectProperty::Property {
                            value, ..
                        } => {
                            self.collect_instantiations_from_expression(value);
                        }
                        luanext_parser::ast::expression::ObjectProperty::Computed {
                            key,
                            value,
                            ..
                        } => {
                            self.collect_instantiations_from_expression(key);
                            self.collect_instantiations_from_expression(value);
                        }
                        luanext_parser::ast::expression::ObjectProperty::Spread {
                            value, ..
                        } => {
                            self.collect_instantiations_from_expression(value);
                        }
                    }
                }
            }
            Parenthesized(inner) => {
                self.collect_instantiations_from_expression(inner);
            }
            _ => {}
        }
    }

    fn record_instantiation_from_expression<'arena>(&mut self, expr: &Expression<'arena>) {
        if let Call(func, _args, _) = &expr.kind {
            self.record_instantiation_from_callee(func);
        }
    }

    fn record_instantiation_from_callee<'arena>(&mut self, expr: &Expression<'arena>) {
        match &expr.kind {
            ExpressionKind::Identifier(id) => {
                let class_id = *id;
                self.classes_with_instantiations.insert(class_id);
                *self.instantiation_counts.entry(class_id).or_insert(0) += 1;
                self.add_instantiation_to_hierarchy(class_id, class_id);
            }
            ExpressionKind::Member(_obj, member) => {
                let class_id = member.node;
                self.classes_with_instantiations.insert(class_id);
                *self.instantiation_counts.entry(class_id).or_insert(0) += 1;
                self.add_instantiation_to_hierarchy(class_id, class_id);
            }
            _ => {}
        }
    }

    fn add_instantiation_to_hierarchy(
        &mut self,
        instantiated_class: StringId,
        original_class: StringId,
    ) {
        let mut current = original_class;
        while let Some(&parent) = self.parent_of.get(&current) {
            if let Some(parent_id) = parent {
                self.instantiated_subclasses
                    .entry(parent_id)
                    .or_default()
                    .insert(instantiated_class);
                current = parent_id;
            } else {
                break;
            }
        }
    }

    fn compute_single_instantiated_subclasses(&mut self) {
        for (base_class, subclasses) in &self.instantiated_subclasses {
            if subclasses.len() == 1 {
                if let Some(&single_subclass) = subclasses.iter().next() {
                    self.single_instantiated_subclass
                        .insert(*base_class, single_subclass);
                    debug!(
                        "RTA: Base class {:?} has single instantiated subclass {:?}",
                        base_class, single_subclass
                    );
                }
            }
        }
    }

    pub fn is_known_class(&self, class: StringId) -> bool {
        self.known_classes.contains_key(&class)
    }

    pub fn can_devirtualize(&self, class: StringId, method: StringId) -> bool {
        if self.is_final.get(&class) == Some(&true) {
            return true;
        }
        if self.final_methods.get(&(class, method)) == Some(&true) {
            return true;
        }
        !self.any_descendant_overrides(class, method)
    }

    fn any_descendant_overrides(&self, class: StringId, method: StringId) -> bool {
        if let Some(children) = self.children_of.get(&class) {
            for &child in children {
                if self.declares_method.get(&(child, method)) == Some(&true) {
                    return true;
                }
                if self.any_descendant_overrides(child, method) {
                    return true;
                }
            }
        }
        false
    }

    pub fn record_instantiation(&mut self, class_name: StringId) {
        self.classes_with_instantiations.insert(class_name);
        *self.instantiation_counts.entry(class_name).or_insert(0) += 1;
    }

    pub fn set_instantiated_subclasses(
        &mut self,
        base_class: StringId,
        subclasses: FxHashSet<StringId>,
    ) {
        self.instantiated_subclasses
            .insert(base_class, subclasses.clone());
        if subclasses.len() == 1 {
            if let Some(&single_subclass) = subclasses.iter().next() {
                self.single_instantiated_subclass
                    .insert(base_class, single_subclass);
            }
        }
    }

    pub fn get_single_instantiated_subclass(&self, class: StringId) -> Option<StringId> {
        self.single_instantiated_subclass.get(&class).copied()
    }

    pub fn has_instantiation_info(&self, class: StringId) -> bool {
        self.instantiated_subclasses.contains_key(&class)
            || self.classes_with_instantiations.contains(&class)
    }

    pub fn can_devirtualize_with_rta(
        &self,
        class: StringId,
        method: StringId,
    ) -> (bool, Option<StringId>) {
        if self.is_final.get(&class) == Some(&true) {
            return (true, None);
        }
        if self.final_methods.get(&(class, method)) == Some(&true) {
            return (true, None);
        }
        if let Some(subclass) = self.single_instantiated_subclass.get(&class) {
            return (true, Some(*subclass));
        }
        let has_overrides = self.any_descendant_overrides(class, method);
        if !has_overrides {
            return (true, None);
        }
        (false, None)
    }
}

// =============================================================================
// DevirtualizationPass — stub during arena migration
// =============================================================================

use crate::config::OptimizationLevel;
use crate::MutableProgram;
use bumpalo::Bump;
use luanext_parser::string_interner::StringInterner;
use std::sync::Arc;

use super::{AstFeatures, WholeProgramPass};

/// Devirtualization optimization pass (O3).
///
/// Replaces virtual method calls with direct calls when the class hierarchy
/// allows safe devirtualization. Currently a stub during arena migration.
pub struct DevirtualizationPass {
    class_hierarchy: Option<ClassHierarchy>,
}

impl DevirtualizationPass {
    pub fn new(_interner: Arc<StringInterner>) -> Self {
        Self {
            class_hierarchy: None,
        }
    }

    pub fn set_class_hierarchy(&mut self, hierarchy: ClassHierarchy) {
        self.class_hierarchy = Some(hierarchy);
    }

    fn devirtualize_in_statement<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match stmt {
            Statement::Function(func) => {
                let mut stmts: Vec<_> = func.body.statements.to_vec();
                let mut changed = false;
                for s in &mut stmts {
                    changed |= self.devirtualize_in_statement(s, arena);
                }
                if changed {
                    func.body.statements = arena.alloc_slice_clone(&stmts);
                }
                changed
            }
            Statement::If(if_stmt) => {
                let mut changed = self.devirtualize_in_expression(&mut if_stmt.condition, arena);
                changed |= self.devirtualize_in_block(&mut if_stmt.then_block, arena);
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for else_if in &mut new_else_ifs {
                    eic |= self.devirtualize_in_expression(&mut else_if.condition, arena);
                    eic |= self.devirtualize_in_block(&mut else_if.block, arena);
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                    changed = true;
                }
                if let Some(else_block) = &mut if_stmt.else_block {
                    changed |= self.devirtualize_in_block(else_block, arena);
                }
                changed
            }
            Statement::While(while_stmt) => {
                let mut changed = self.devirtualize_in_expression(&mut while_stmt.condition, arena);
                changed |= self.devirtualize_in_block(&mut while_stmt.body, arena);
                changed
            }
            Statement::For(for_stmt) => {
                use luanext_parser::ast::statement::ForStatement;
                match &**for_stmt {
                    ForStatement::Numeric(for_num_ref) => {
                        let mut new_num = (**for_num_ref).clone();
                        let changed = self.devirtualize_in_block(&mut new_num.body, arena);
                        if changed {
                            *stmt = Statement::For(
                                arena.alloc(ForStatement::Numeric(arena.alloc(new_num))),
                            );
                        }
                        changed
                    }
                    ForStatement::Generic(for_gen_ref) => {
                        let mut new_gen = for_gen_ref.clone();
                        let changed = self.devirtualize_in_block(&mut new_gen.body, arena);
                        if changed {
                            *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                        }
                        changed
                    }
                }
            }
            Statement::Repeat(repeat_stmt) => {
                let mut changed = self.devirtualize_in_expression(&mut repeat_stmt.until, arena);
                changed |= self.devirtualize_in_block(&mut repeat_stmt.body, arena);
                changed
            }
            Statement::Return(return_stmt) => {
                let mut vals: Vec<_> = return_stmt.values.to_vec();
                let mut changed = false;
                for value in &mut vals {
                    changed |= self.devirtualize_in_expression(value, arena);
                }
                if changed {
                    return_stmt.values = arena.alloc_slice_clone(&vals);
                }
                changed
            }
            Statement::Expression(expr) => self.devirtualize_in_expression(expr, arena),
            Statement::Block(block) => self.devirtualize_in_block(block, arena),
            Statement::Try(try_stmt) => {
                let mut changed = self.devirtualize_in_block(&mut try_stmt.try_block, arena);
                let mut new_clauses: Vec<_> = try_stmt.catch_clauses.to_vec();
                let mut clauses_changed = false;
                for clause in &mut new_clauses {
                    clauses_changed |= self.devirtualize_in_block(&mut clause.body, arena);
                }
                if clauses_changed {
                    try_stmt.catch_clauses = arena.alloc_slice_clone(&new_clauses);
                    changed = true;
                }
                if let Some(finally) = &mut try_stmt.finally_block {
                    changed |= self.devirtualize_in_block(finally, arena);
                }
                changed
            }
            Statement::Class(class) => {
                use luanext_parser::ast::statement::ClassMember;
                let mut changed = false;
                let mut new_members: Vec<_> = class.members.to_vec();
                let mut members_changed = false;
                for member in &mut new_members {
                    match member {
                        ClassMember::Method(method) => {
                            if let Some(body) = &mut method.body {
                                members_changed |= self.devirtualize_in_block(body, arena);
                            }
                        }
                        ClassMember::Constructor(ctor) => {
                            members_changed |= self.devirtualize_in_block(&mut ctor.body, arena);
                        }
                        _ => {}
                    }
                }
                if members_changed {
                    class.members = arena.alloc_slice_clone(&new_members);
                    changed = true;
                }
                changed
            }
            _ => false,
        }
    }

    fn devirtualize_in_block<'arena>(
        &mut self,
        block: &mut Block<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        let mut stmts: Vec<_> = block.statements.to_vec();
        let mut changed = false;
        for stmt in &mut stmts {
            changed |= self.devirtualize_in_statement(stmt, arena);
        }
        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
        changed
    }

    fn devirtualize_in_expression<'arena>(
        &mut self,
        expr: &mut Expression<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match &expr.kind {
            ExpressionKind::MethodCall(obj, method_name, args, type_args) => {
                let method_name_clone = method_name.clone();
                let mut new_obj = (**obj).clone();
                let mut changed = self.devirtualize_in_expression(&mut new_obj, arena);

                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.devirtualize_in_expression(&mut arg.value, arena);
                }

                // Attempt devirtualization
                if let Some(hierarchy) = &self.class_hierarchy {
                    if let Some(receiver_info) = &expr.receiver_class {
                        let class_id = receiver_info.class_name;
                        let method_id = method_name_clone.node;

                        let (can_devirt, target_class) =
                            hierarchy.can_devirtualize_with_rta(class_id, method_id);

                        if can_devirt {
                            // Determine which class to use
                            let effective_class = target_class.unwrap_or(class_id);

                            // Build the devirtualized call: ClassName.methodName(obj, ...)
                            let class_expr = Expression {
                                kind: ExpressionKind::Identifier(effective_class),
                                span: expr.span,
                                annotated_type: None,
                                receiver_class: None,
                            };

                            let member_expr = Expression {
                                kind: ExpressionKind::Member(
                                    arena.alloc(class_expr),
                                    method_name_clone.clone(),
                                ),
                                span: expr.span,
                                annotated_type: None,
                                receiver_class: None,
                            };

                            // Prepend object as first argument
                            let devirt_args: Vec<_> =
                                std::iter::once(luanext_parser::ast::expression::Argument {
                                    value: new_obj.clone(),
                                    is_spread: false,
                                    span: expr.span,
                                })
                                .chain(new_args.iter().cloned())
                                .collect();

                            expr.kind = ExpressionKind::Call(
                                arena.alloc(member_expr),
                                arena.alloc_slice_clone(&devirt_args),
                                *type_args,
                            );
                            expr.receiver_class = None;

                            debug!(
                                "Devirtualized method call {:?}.{:?} -> {:?}.{:?}",
                                class_id, method_id, effective_class, method_id
                            );
                            return true;
                        }
                    }
                }

                // If not devirtualized, update with traversed children
                if changed || args_changed {
                    expr.kind = ExpressionKind::MethodCall(
                        arena.alloc(new_obj),
                        method_name_clone,
                        arena.alloc_slice_clone(&new_args),
                        *type_args,
                    );
                    changed = true;
                }

                changed
            }
            ExpressionKind::Call(func, args, type_args) => {
                let mut new_func = (**func).clone();
                let mut changed = self.devirtualize_in_expression(&mut new_func, arena);
                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.devirtualize_in_expression(&mut arg.value, arena);
                }
                if changed || args_changed {
                    expr.kind = ExpressionKind::Call(
                        arena.alloc(new_func),
                        arena.alloc_slice_clone(&new_args),
                        *type_args,
                    );
                    changed = true;
                }
                changed
            }
            ExpressionKind::Binary(op, left, right) => {
                let op = *op;
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let left_changed = self.devirtualize_in_expression(&mut new_left, arena);
                let right_changed = self.devirtualize_in_expression(&mut new_right, arena);
                if left_changed || right_changed {
                    expr.kind =
                        ExpressionKind::Binary(op, arena.alloc(new_left), arena.alloc(new_right));
                }
                left_changed || right_changed
            }
            ExpressionKind::Unary(op, operand) => {
                let op = *op;
                let mut new_operand = (**operand).clone();
                let changed = self.devirtualize_in_expression(&mut new_operand, arena);
                if changed {
                    expr.kind = ExpressionKind::Unary(op, arena.alloc(new_operand));
                }
                changed
            }
            ExpressionKind::Assignment(left, op, right) => {
                let op = *op;
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let left_changed = self.devirtualize_in_expression(&mut new_left, arena);
                let right_changed = self.devirtualize_in_expression(&mut new_right, arena);
                if left_changed || right_changed {
                    expr.kind = ExpressionKind::Assignment(
                        arena.alloc(new_left),
                        op,
                        arena.alloc(new_right),
                    );
                }
                left_changed || right_changed
            }
            ExpressionKind::Conditional(cond, then_expr, else_expr) => {
                let mut new_cond = (**cond).clone();
                let mut new_then = (**then_expr).clone();
                let mut new_else = (**else_expr).clone();
                let c1 = self.devirtualize_in_expression(&mut new_cond, arena);
                let c2 = self.devirtualize_in_expression(&mut new_then, arena);
                let c3 = self.devirtualize_in_expression(&mut new_else, arena);
                if c1 || c2 || c3 {
                    expr.kind = ExpressionKind::Conditional(
                        arena.alloc(new_cond),
                        arena.alloc(new_then),
                        arena.alloc(new_else),
                    );
                }
                c1 || c2 || c3
            }
            ExpressionKind::Pipe(left, right) => {
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let left_changed = self.devirtualize_in_expression(&mut new_left, arena);
                let right_changed = self.devirtualize_in_expression(&mut new_right, arena);
                if left_changed || right_changed {
                    expr.kind = ExpressionKind::Pipe(arena.alloc(new_left), arena.alloc(new_right));
                }
                left_changed || right_changed
            }
            ExpressionKind::Match(match_expr) => {
                let mut new_value = (*match_expr.value).clone();
                let mut changed = self.devirtualize_in_expression(&mut new_value, arena);
                let mut new_arms: Vec<_> = match_expr.arms.to_vec();
                let mut arms_changed = false;
                for arm in &mut new_arms {
                    match &mut arm.body {
                        luanext_parser::ast::expression::MatchArmBody::Expression(arm_expr) => {
                            let mut new_arm_expr = (**arm_expr).clone();
                            if self.devirtualize_in_expression(&mut new_arm_expr, arena) {
                                arm.body =
                                    luanext_parser::ast::expression::MatchArmBody::Expression(
                                        arena.alloc(new_arm_expr),
                                    );
                                arms_changed = true;
                            }
                        }
                        luanext_parser::ast::expression::MatchArmBody::Block(block) => {
                            arms_changed |= self.devirtualize_in_block(block, arena);
                        }
                    }
                }
                if changed || arms_changed {
                    expr.kind =
                        ExpressionKind::Match(luanext_parser::ast::expression::MatchExpression {
                            value: arena.alloc(new_value),
                            arms: arena.alloc_slice_clone(&new_arms),
                            span: match_expr.span,
                        });
                    changed = true;
                }
                changed
            }
            ExpressionKind::Arrow(arrow) => {
                let mut new_arrow = arrow.clone();
                let mut changed = false;
                let mut new_params: Vec<_> = new_arrow.parameters.to_vec();
                let mut params_changed = false;
                for param in &mut new_params {
                    if let Some(default) = &mut param.default {
                        params_changed |= self.devirtualize_in_expression(default, arena);
                    }
                }
                if params_changed {
                    new_arrow.parameters = arena.alloc_slice_clone(&new_params);
                    changed = true;
                }
                match &mut new_arrow.body {
                    luanext_parser::ast::expression::ArrowBody::Expression(body_expr) => {
                        let mut new_body = (**body_expr).clone();
                        if self.devirtualize_in_expression(&mut new_body, arena) {
                            new_arrow.body = luanext_parser::ast::expression::ArrowBody::Expression(
                                arena.alloc(new_body),
                            );
                            changed = true;
                        }
                    }
                    luanext_parser::ast::expression::ArrowBody::Block(block) => {
                        changed |= self.devirtualize_in_block(block, arena);
                    }
                }
                if changed {
                    expr.kind = ExpressionKind::Arrow(new_arrow);
                }
                changed
            }
            ExpressionKind::New(callee, args, type_args) => {
                let mut new_callee = (**callee).clone();
                let mut changed = self.devirtualize_in_expression(&mut new_callee, arena);
                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.devirtualize_in_expression(&mut arg.value, arena);
                }
                if changed || args_changed {
                    expr.kind = ExpressionKind::New(
                        arena.alloc(new_callee),
                        arena.alloc_slice_clone(&new_args),
                        *type_args,
                    );
                    changed = true;
                }
                changed
            }
            ExpressionKind::Try(try_expr) => {
                let mut new_expression = (*try_expr.expression).clone();
                let mut new_catch = (*try_expr.catch_expression).clone();
                let c1 = self.devirtualize_in_expression(&mut new_expression, arena);
                let c2 = self.devirtualize_in_expression(&mut new_catch, arena);
                if c1 || c2 {
                    expr.kind =
                        ExpressionKind::Try(luanext_parser::ast::expression::TryExpression {
                            expression: arena.alloc(new_expression),
                            catch_variable: try_expr.catch_variable.clone(),
                            catch_expression: arena.alloc(new_catch),
                            span: try_expr.span,
                        });
                }
                c1 || c2
            }
            ExpressionKind::ErrorChain(left, right) => {
                let mut new_left = (**left).clone();
                let mut new_right = (**right).clone();
                let left_changed = self.devirtualize_in_expression(&mut new_left, arena);
                let right_changed = self.devirtualize_in_expression(&mut new_right, arena);
                if left_changed || right_changed {
                    expr.kind =
                        ExpressionKind::ErrorChain(arena.alloc(new_left), arena.alloc(new_right));
                }
                left_changed || right_changed
            }
            ExpressionKind::OptionalMember(obj, member) => {
                let member = member.clone();
                let mut new_obj = (**obj).clone();
                let changed = self.devirtualize_in_expression(&mut new_obj, arena);
                if changed {
                    expr.kind = ExpressionKind::OptionalMember(arena.alloc(new_obj), member);
                }
                changed
            }
            ExpressionKind::OptionalIndex(obj, index) => {
                let mut new_obj = (**obj).clone();
                let mut new_index = (**index).clone();
                let c1 = self.devirtualize_in_expression(&mut new_obj, arena);
                let c2 = self.devirtualize_in_expression(&mut new_index, arena);
                if c1 || c2 {
                    expr.kind =
                        ExpressionKind::OptionalIndex(arena.alloc(new_obj), arena.alloc(new_index));
                }
                c1 || c2
            }
            ExpressionKind::OptionalCall(obj, args, type_args) => {
                let mut new_obj = (**obj).clone();
                let mut changed = self.devirtualize_in_expression(&mut new_obj, arena);
                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.devirtualize_in_expression(&mut arg.value, arena);
                }
                if changed || args_changed {
                    expr.kind = ExpressionKind::OptionalCall(
                        arena.alloc(new_obj),
                        arena.alloc_slice_clone(&new_args),
                        *type_args,
                    );
                    changed = true;
                }
                changed
            }
            ExpressionKind::OptionalMethodCall(obj, method_name, args, type_args) => {
                let method_name = method_name.clone();
                let mut new_obj = (**obj).clone();
                let mut changed = self.devirtualize_in_expression(&mut new_obj, arena);
                let mut new_args: Vec<_> = args.to_vec();
                let mut args_changed = false;
                for arg in &mut new_args {
                    args_changed |= self.devirtualize_in_expression(&mut arg.value, arena);
                }
                if changed || args_changed {
                    expr.kind = ExpressionKind::OptionalMethodCall(
                        arena.alloc(new_obj),
                        method_name,
                        arena.alloc_slice_clone(&new_args),
                        *type_args,
                    );
                    changed = true;
                }
                changed
            }
            ExpressionKind::Member(obj, member) => {
                let member = member.clone();
                let mut new_obj = (**obj).clone();
                let changed = self.devirtualize_in_expression(&mut new_obj, arena);
                if changed {
                    expr.kind = ExpressionKind::Member(arena.alloc(new_obj), member);
                }
                changed
            }
            ExpressionKind::Index(obj, index) => {
                let mut new_obj = (**obj).clone();
                let mut new_index = (**index).clone();
                let c1 = self.devirtualize_in_expression(&mut new_obj, arena);
                let c2 = self.devirtualize_in_expression(&mut new_index, arena);
                if c1 || c2 {
                    expr.kind = ExpressionKind::Index(arena.alloc(new_obj), arena.alloc(new_index));
                }
                c1 || c2
            }
            ExpressionKind::Array(_) | ExpressionKind::Object(_) => false,
            ExpressionKind::Parenthesized(inner) => {
                let mut new_inner = (**inner).clone();
                let changed = self.devirtualize_in_expression(&mut new_inner, arena);
                if changed {
                    expr.kind = ExpressionKind::Parenthesized(arena.alloc(new_inner));
                }
                changed
            }
            ExpressionKind::Identifier(_)
            | ExpressionKind::Literal(_)
            | ExpressionKind::SelfKeyword
            | ExpressionKind::SuperKeyword
            | ExpressionKind::Template(_)
            | ExpressionKind::TypeAssertion(..)
            | ExpressionKind::Function(_) => false,
        }
    }
}

impl<'arena> WholeProgramPass<'arena> for DevirtualizationPass {
    fn name(&self) -> &'static str {
        "devirtualization"
    }

    fn min_level(&self) -> OptimizationLevel {
        OptimizationLevel::Aggressive
    }

    fn required_features(&self) -> AstFeatures {
        AstFeatures::HAS_CLASSES
    }

    fn run(
        &mut self,
        program: &mut MutableProgram<'arena>,
        arena: &'arena Bump,
    ) -> Result<bool, String> {
        let mut changed = false;
        for stmt in &mut program.statements {
            changed |= self.devirtualize_in_statement(stmt, arena);
        }
        Ok(changed)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luanext_parser::ast::expression::ExpressionKind;
    use luanext_parser::ast::statement::Statement;
    use luanext_parser::ast::Spanned;
    use luanext_parser::span::Span;

    #[test]
    fn test_devirtualization_final_class() {
        let arena = Bump::new();
        let interner = Arc::new(StringInterner::new());
        let mut pass = DevirtualizationPass::new(interner.clone());

        // Create a simple class hierarchy: final class FinalClass
        let class_id = interner.get_or_intern("FinalClass");
        let method_id = interner.get_or_intern("compute");

        let mut hierarchy = ClassHierarchy::default();
        hierarchy.is_final.insert(class_id, true);
        hierarchy.known_classes.insert(class_id, true);
        hierarchy
            .declares_method
            .insert((class_id, method_id), true);
        pass.set_class_hierarchy(hierarchy);

        // Create method call: obj.compute()
        let obj_id = interner.get_or_intern("obj");
        let obj_expr = Expression {
            kind: ExpressionKind::Identifier(obj_id),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        };

        let method_call = Expression {
            kind: ExpressionKind::MethodCall(
                arena.alloc(obj_expr),
                Spanned::new(method_id, Span::dummy()),
                arena.alloc_slice_clone(&[]),
                None,
            ),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: Some(luanext_parser::ast::expression::ReceiverClassInfo {
                class_name: class_id,
                is_static: false,
            }),
        };

        let mut expr = method_call;
        let changed = pass.devirtualize_in_expression(&mut expr, &arena);

        assert!(changed, "Should have devirtualized the method call");
        assert!(
            matches!(expr.kind, ExpressionKind::Call(..)),
            "Should be converted to Call"
        );
        assert!(
            expr.receiver_class.is_none(),
            "receiver_class should be cleared"
        );
    }

    #[test]
    fn test_devirtualization_final_method() {
        let arena = Bump::new();
        let interner = Arc::new(StringInterner::new());
        let mut pass = DevirtualizationPass::new(interner.clone());

        let class_id = interner.get_or_intern("BaseClass");
        let method_id = interner.get_or_intern("finalMethod");

        let mut hierarchy = ClassHierarchy::default();
        hierarchy.is_final.insert(class_id, false);
        hierarchy.known_classes.insert(class_id, true);
        hierarchy.final_methods.insert((class_id, method_id), true);
        hierarchy
            .declares_method
            .insert((class_id, method_id), true);
        pass.set_class_hierarchy(hierarchy);

        let obj_id = interner.get_or_intern("obj");
        let obj_expr = Expression {
            kind: ExpressionKind::Identifier(obj_id),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        };

        let method_call = Expression {
            kind: ExpressionKind::MethodCall(
                arena.alloc(obj_expr),
                Spanned::new(method_id, Span::dummy()),
                arena.alloc_slice_clone(&[]),
                None,
            ),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: Some(luanext_parser::ast::expression::ReceiverClassInfo {
                class_name: class_id,
                is_static: false,
            }),
        };

        let mut expr = method_call;
        let changed = pass.devirtualize_in_expression(&mut expr, &arena);

        assert!(changed, "Should have devirtualized the final method call");
        assert!(
            matches!(expr.kind, ExpressionKind::Call(..)),
            "Should be converted to Call"
        );
    }

    #[test]
    fn test_devirtualization_rta_single_subclass() {
        let arena = Bump::new();
        let interner = Arc::new(StringInterner::new());
        let mut pass = DevirtualizationPass::new(interner.clone());

        let base_class_id = interner.get_or_intern("BaseClass");
        let sub_class_id = interner.get_or_intern("SubClass");
        let method_id = interner.get_or_intern("method");

        let mut hierarchy = ClassHierarchy::default();
        hierarchy.is_final.insert(base_class_id, false);
        hierarchy.known_classes.insert(base_class_id, true);
        hierarchy.known_classes.insert(sub_class_id, true);
        hierarchy
            .parent_of
            .insert(sub_class_id, Some(base_class_id));
        hierarchy
            .declares_method
            .insert((base_class_id, method_id), true);
        hierarchy
            .declares_method
            .insert((sub_class_id, method_id), true);

        // Set up RTA: only SubClass is instantiated
        let mut subclasses = FxHashSet::default();
        subclasses.insert(sub_class_id);
        hierarchy.set_instantiated_subclasses(base_class_id, subclasses);

        pass.set_class_hierarchy(hierarchy);

        let obj_id = interner.get_or_intern("obj");
        let obj_expr = Expression {
            kind: ExpressionKind::Identifier(obj_id),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        };

        let method_call = Expression {
            kind: ExpressionKind::MethodCall(
                arena.alloc(obj_expr),
                Spanned::new(method_id, Span::dummy()),
                arena.alloc_slice_clone(&[]),
                None,
            ),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: Some(luanext_parser::ast::expression::ReceiverClassInfo {
                class_name: base_class_id,
                is_static: false,
            }),
        };

        let mut expr = method_call;
        let changed = pass.devirtualize_in_expression(&mut expr, &arena);

        assert!(
            changed,
            "Should have devirtualized using RTA single-subclass info"
        );
        assert!(
            matches!(expr.kind, ExpressionKind::Call(..)),
            "Should be converted to Call"
        );
    }

    #[test]
    fn test_no_devirtualization_virtual_method() {
        let arena = Bump::new();
        let interner = Arc::new(StringInterner::new());
        let mut pass = DevirtualizationPass::new(interner.clone());

        let base_class_id = interner.get_or_intern("BaseClass");
        let sub_class1_id = interner.get_or_intern("SubClass1");
        let sub_class2_id = interner.get_or_intern("SubClass2");
        let method_id = interner.get_or_intern("virtualMethod");

        let mut hierarchy = ClassHierarchy::default();
        hierarchy.is_final.insert(base_class_id, false);
        hierarchy.known_classes.insert(base_class_id, true);
        hierarchy.known_classes.insert(sub_class1_id, true);
        hierarchy.known_classes.insert(sub_class2_id, true);
        hierarchy
            .parent_of
            .insert(sub_class1_id, Some(base_class_id));
        hierarchy
            .parent_of
            .insert(sub_class2_id, Some(base_class_id));
        hierarchy
            .children_of
            .entry(base_class_id)
            .or_default()
            .push(sub_class1_id);
        hierarchy
            .children_of
            .entry(base_class_id)
            .or_default()
            .push(sub_class2_id);
        hierarchy
            .declares_method
            .insert((base_class_id, method_id), true);
        hierarchy
            .declares_method
            .insert((sub_class1_id, method_id), true);
        hierarchy
            .declares_method
            .insert((sub_class2_id, method_id), true);

        // Both subclasses instantiated
        let mut subclasses = FxHashSet::default();
        subclasses.insert(sub_class1_id);
        subclasses.insert(sub_class2_id);
        hierarchy.set_instantiated_subclasses(base_class_id, subclasses);

        pass.set_class_hierarchy(hierarchy);

        let obj_id = interner.get_or_intern("obj");
        let obj_expr = Expression {
            kind: ExpressionKind::Identifier(obj_id),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        };

        let method_call = Expression {
            kind: ExpressionKind::MethodCall(
                arena.alloc(obj_expr),
                Spanned::new(method_id, Span::dummy()),
                arena.alloc_slice_clone(&[]),
                None,
            ),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: Some(luanext_parser::ast::expression::ReceiverClassInfo {
                class_name: base_class_id,
                is_static: false,
            }),
        };

        let mut expr = method_call;
        let changed = pass.devirtualize_in_expression(&mut expr, &arena);

        assert!(
            !changed,
            "Should NOT devirtualize when multiple subclasses override"
        );
        assert!(
            matches!(expr.kind, ExpressionKind::MethodCall(..)),
            "Should remain a MethodCall"
        );
    }

    #[test]
    fn test_devirtualization_in_program() {
        let arena = Bump::new();
        let interner = Arc::new(StringInterner::new());
        let mut pass = DevirtualizationPass::new(interner.clone());

        let class_id = interner.get_or_intern("FinalClass");
        let method_id = interner.get_or_intern("compute");

        let mut hierarchy = ClassHierarchy::default();
        hierarchy.is_final.insert(class_id, true);
        hierarchy.known_classes.insert(class_id, true);
        hierarchy
            .declares_method
            .insert((class_id, method_id), true);
        pass.set_class_hierarchy(hierarchy);

        let obj_id = interner.get_or_intern("obj");
        let obj_expr = Expression {
            kind: ExpressionKind::Identifier(obj_id),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        };

        let method_call_expr = Expression {
            kind: ExpressionKind::MethodCall(
                arena.alloc(obj_expr),
                Spanned::new(method_id, Span::dummy()),
                arena.alloc_slice_clone(&[]),
                None,
            ),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: Some(luanext_parser::ast::expression::ReceiverClassInfo {
                class_name: class_id,
                is_static: false,
            }),
        };

        let stmt = Statement::Expression(method_call_expr);
        let mut program = MutableProgram {
            statements: vec![stmt],
            span: Span::dummy(),
        };

        let result = pass.run(&mut program, &arena);
        assert!(result.is_ok());
        assert!(result.unwrap(), "Should have made changes");

        if let Statement::Expression(expr) = &program.statements[0] {
            assert!(
                matches!(expr.kind, ExpressionKind::Call(..)),
                "Method call should be devirtualized to Call"
            );
        } else {
            panic!("Expected Expression statement");
        }
    }
}
