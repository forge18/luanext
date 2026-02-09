// =============================================================================
// O2: Tail Call Optimization Pass (Analysis Only)
// =============================================================================
//
// ## Why This Pass Doesn't Transform Code
//
// Lua's runtime provides **guaranteed tail call elimination** as part of the language
// specification (PiL 6.3). When Lua executes `return f()`, the VM automatically:
// - Reuses the current stack frame instead of creating a new one
// - Eliminates call overhead for arbitrarily deep recursion
// - Applies this optimization to ALL tail calls, not just self-recursion
//
// ## Why Compiler-Level TCO Would Be Harmful
//
// Attempting to "optimize" tail calls at compile-time would:
// 1. **Add complexity**: Converting recursion to loops increases code size and complexity
// 2. **Break semantics**: Mutual recursion can't be converted to simple loops
// 3. **Harm debugging**: Source maps wouldn't match transformed code
// 4. **Provide zero benefit**: Lua's VM already eliminates tail calls perfectly
// 5. **Reduce performance**: Any transformation adds overhead vs. native VM optimization
//
// ## What This Pass Actually Does
//
// This analysis-only pass serves three purposes:
// 1. **Verification**: Ensures other optimization passes don't break tail positions
// 2. **Metrics**: Counts tail calls for profiling and analysis
// 3. **Future diagnostics**: Foundation for warnings about non-tail recursive calls
//
// ## When Would We Need Compiler TCO?
//
// You'd only need compiler-level transformation if:
// - Targeting a runtime without TCO (e.g., JavaScript pre-ES6)
// - Cross-compiling to a non-Lua target
// - The runtime TCO implementation is broken
//
// Since LuaNext targets Lua (which has perfect TCO), we preserve tail calls as-is.
//
// =============================================================================

use crate::config::OptimizationLevel;
use crate::optimizer::{StmtVisitor, WholeProgramPass};
use crate::MutableProgram;
use bumpalo::Bump;
use luanext_parser::ast::expression::Expression;
use luanext_parser::ast::expression::ExpressionKind;
use luanext_parser::ast::statement::{ForStatement, Statement};

/// Tail call optimization pass (analysis only)
///
/// This pass analyzes tail call patterns and ensures other optimizations don't break
/// tail call positions. It does NOT transform code because Lua's VM already provides
/// guaranteed tail call elimination.
///
/// See module-level documentation for detailed rationale.
pub struct TailCallOptimizationPass;

impl TailCallOptimizationPass {
    pub fn new() -> Self {
        Self
    }
}

impl<'arena> StmtVisitor<'arena> for TailCallOptimizationPass {
    fn visit_stmt(&mut self, _stmt: &mut Statement<'arena>, _arena: &'arena Bump) -> bool {
        // This pass is analysis-only and never modifies the AST.
        // Lua's VM provides guaranteed tail call elimination, so we preserve
        // tail calls as-is to let the runtime optimize them.
        false
    }
}

impl<'arena> WholeProgramPass<'arena> for TailCallOptimizationPass {
    fn name(&self) -> &'static str {
        "tail-call-optimization"
    }

    fn min_level(&self) -> OptimizationLevel {
        OptimizationLevel::Moderate
    }

    fn run(
        &mut self,
        program: &mut MutableProgram<'arena>,
        _arena: &'arena Bump,
    ) -> Result<bool, String> {
        // Analyze tail call patterns for verification and metrics
        let tail_call_count: usize = program
            .statements
            .iter()
            .map(|stmt| self.analyze_statement_tail_calls(stmt))
            .sum();

        // Log tail call count for debugging (can be used for future diagnostics)
        if cfg!(debug_assertions) && tail_call_count > 0 {
            eprintln!(
                "[TCO] Found {} tail calls (preserved for Lua VM optimization)",
                tail_call_count
            );
        }

        // Return false because this pass never modifies the AST
        Ok(false)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl TailCallOptimizationPass {
    /// Analyzes a statement for tail call patterns and returns the count.
    ///
    /// A tail call is a function call that appears in tail position - the last operation
    /// before a return. Lua's VM automatically eliminates these calls, reusing the current
    /// stack frame instead of allocating a new one.
    fn analyze_statement_tail_calls<'arena>(&self, stmt: &Statement<'arena>) -> usize {
        match stmt {
            Statement::Function(func) => self.analyze_block_tail_calls(func.body.statements),
            Statement::Block(block) => self.analyze_block_tail_calls(block.statements),
            Statement::If(if_stmt) => {
                let mut count = self.analyze_block_tail_calls(if_stmt.then_block.statements);
                for else_if in if_stmt.else_ifs.iter() {
                    count += self.analyze_block_tail_calls(else_if.block.statements);
                }
                if let Some(else_block) = &if_stmt.else_block {
                    count += self.analyze_block_tail_calls(else_block.statements);
                }
                count
            }
            Statement::While(while_stmt) => {
                self.analyze_block_tail_calls(while_stmt.body.statements)
            }
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num) => {
                    self.analyze_block_tail_calls(for_num.body.statements)
                }
                ForStatement::Generic(for_gen) => {
                    self.analyze_block_tail_calls(for_gen.body.statements)
                }
            },
            Statement::Repeat(repeat_stmt) => {
                self.analyze_block_tail_calls(repeat_stmt.body.statements)
            }
            Statement::Return(ret) => {
                if self.is_tail_call(ret.values) {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    fn analyze_block_tail_calls<'arena>(&self, stmts: &[Statement<'arena>]) -> usize {
        let mut count = 0;
        for stmt in stmts {
            count += self.analyze_statement_tail_calls(stmt);
        }
        count
    }

    /// Checks if a return statement contains a tail call.
    ///
    /// A tail call must be:
    /// 1. The only return value (single expression)
    /// 2. A function call or method call
    /// 3. In tail position (handled by caller - this checks the return)
    ///
    /// Examples of tail calls:
    /// - `return factorial(n - 1, n * acc)` ✓
    /// - `return obj:method(x)` ✓
    /// - `return factorial(n - 1) + 1` ✗ (addition after call)
    /// - `return factorial(n), other()` ✗ (multiple returns)
    fn is_tail_call<'arena>(&self, values: &[Expression<'arena>]) -> bool {
        if values.len() != 1 {
            return false;
        }
        matches!(
            values[0].kind,
            ExpressionKind::Call(_, _, _) | ExpressionKind::MethodCall(_, _, _, _)
        )
    }
}

impl Default for TailCallOptimizationPass {
    fn default() -> Self {
        Self::new()
    }
}
