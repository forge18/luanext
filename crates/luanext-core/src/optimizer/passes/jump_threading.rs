//! Jump Threading Optimization Pass
//!
//! Eliminates branches with statically-known conditions at the statement level.
//! This pass works on `if` statements whose conditions are compile-time constants
//! (boolean literals or nil), replacing them with the appropriate branch body.
//!
//! # Examples
//!
//! ```lua
//! -- Before:
//! if true then
//!     print("hello")
//! else
//!     print("world")
//! end
//!
//! -- After:
//! print("hello")
//! ```
//!
//! ```lua
//! -- Before:
//! if false then
//!     print("dead")
//! end
//!
//! -- After:
//! (statement removed entirely)
//! ```

use crate::optimizer::BlockVisitor;
use bumpalo::Bump;
use luanext_parser::ast::expression::{ExpressionKind, Literal};
use luanext_parser::ast::statement::{Block, ForStatement, IfStatement, Statement};

/// Jump threading optimization pass.
///
/// Replaces `if` statements with constant conditions by inlining
/// the taken branch and discarding the dead branch.
pub struct JumpThreadingPass;

impl JumpThreadingPass {
    pub fn new() -> Self {
        Self
    }

    /// Returns whether a condition expression is statically truthy.
    /// In Lua, `nil` and `false` are falsy; everything else is truthy.
    fn is_constant_truthy(condition: &ExpressionKind) -> Option<bool> {
        match condition {
            ExpressionKind::Literal(Literal::Boolean(b)) => Some(*b),
            ExpressionKind::Literal(Literal::Nil) => Some(false),
            // Other literals (numbers, strings) are always truthy in Lua
            ExpressionKind::Literal(Literal::Number(_))
            | ExpressionKind::Literal(Literal::Integer(_))
            | ExpressionKind::Literal(Literal::String(_)) => Some(true),
            _ => None,
        }
    }

    /// Process a block of statements, threading jumps in `if`/`while` statements.
    /// Returns true if any changes were made.
    fn thread_in_vec<'arena>(
        &mut self,
        stmts: &mut Vec<Statement<'arena>>,
        arena: &'arena Bump,
    ) -> bool {
        let mut changed = false;
        let mut i = 0;

        while i < stmts.len() {
            // First, recurse into child blocks of the current statement
            changed |= self.thread_in_children(&mut stmts[i], arena);

            match &stmts[i] {
                Statement::If(if_stmt) => {
                    if let Some(truthy) = Self::is_constant_truthy(&if_stmt.condition.kind) {
                        if truthy {
                            // Condition is always true: replace with then_block statements
                            let body_stmts: Vec<_> = if_stmt.then_block.statements.to_vec();
                            stmts.splice(i..=i, body_stmts.into_iter());
                            changed = true;
                            // Don't increment i — re-process spliced statements
                            continue;
                        } else {
                            // Condition is always false
                            if !if_stmt.else_ifs.is_empty() {
                                // Promote first else-if to become the if condition
                                let first_else_if = &if_stmt.else_ifs[0];
                                let remaining_else_ifs = &if_stmt.else_ifs[1..];
                                let new_if = IfStatement {
                                    condition: first_else_if.condition.clone(),
                                    then_block: first_else_if.block.clone(),
                                    else_ifs: arena.alloc_slice_clone(remaining_else_ifs),
                                    else_block: if_stmt.else_block.clone(),
                                    span: if_stmt.span,
                                };
                                stmts[i] = Statement::If(new_if);
                                changed = true;
                                // Don't increment — re-evaluate the new if statement
                                continue;
                            } else if let Some(else_block) = &if_stmt.else_block {
                                // Has else: replace with else_block statements
                                let body_stmts: Vec<_> = else_block.statements.to_vec();
                                stmts.splice(i..=i, body_stmts.into_iter());
                                changed = true;
                                continue;
                            } else {
                                // No else-ifs, no else: remove entirely
                                stmts.remove(i);
                                changed = true;
                                continue;
                            }
                        }
                    }
                }
                Statement::While(while_stmt) => {
                    if let Some(false) = Self::is_constant_truthy(&while_stmt.condition.kind) {
                        // while false do ... end → remove entirely
                        stmts.remove(i);
                        changed = true;
                        continue;
                    }
                }
                _ => {}
            }

            i += 1;
        }

        changed
    }

    /// Recurse into child blocks of a statement to thread jumps at all nesting levels.
    fn thread_in_children<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match stmt {
            Statement::If(if_stmt) => {
                let mut changed = self.thread_in_block(&mut if_stmt.then_block, arena);

                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for else_if in &mut new_else_ifs {
                    eic |= self.thread_in_block(&mut else_if.block, arena);
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                    changed = true;
                }

                if let Some(else_block) = &mut if_stmt.else_block {
                    changed |= self.thread_in_block(else_block, arena);
                }

                changed
            }
            Statement::While(while_stmt) => self.thread_in_block(&mut while_stmt.body, arena),
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num_ref) => {
                    let mut new_num = (**for_num_ref).clone();
                    let fc = self.thread_in_block(&mut new_num.body, arena);
                    if fc {
                        *stmt = Statement::For(
                            arena.alloc(ForStatement::Numeric(arena.alloc(new_num))),
                        );
                    }
                    fc
                }
                ForStatement::Generic(for_gen_ref) => {
                    let mut new_gen = for_gen_ref.clone();
                    let fc = self.thread_in_block(&mut new_gen.body, arena);
                    if fc {
                        *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                    }
                    fc
                }
            },
            Statement::Repeat(repeat_stmt) => self.thread_in_block(&mut repeat_stmt.body, arena),
            Statement::Function(func) => self.thread_in_block(&mut func.body, arena),
            Statement::Block(block) => self.thread_in_block(block, arena),
            _ => false,
        }
    }

    /// Thread jumps within a block.
    fn thread_in_block<'arena>(&mut self, block: &mut Block<'arena>, arena: &'arena Bump) -> bool {
        let mut stmts: Vec<_> = block.statements.to_vec();
        let changed = self.thread_in_vec(&mut stmts, arena);
        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
        changed
    }
}

impl Default for JumpThreadingPass {
    fn default() -> Self {
        Self::new()
    }
}

impl<'arena> BlockVisitor<'arena> for JumpThreadingPass {
    fn visit_block_stmts(
        &mut self,
        stmts: &mut Vec<Statement<'arena>>,
        arena: &'arena Bump,
    ) -> bool {
        self.thread_in_vec(stmts, arena)
    }
}
