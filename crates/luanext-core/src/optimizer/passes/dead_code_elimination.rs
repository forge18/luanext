use crate::optimizer::BlockVisitor;
use bumpalo::Bump;
use typedlua_parser::ast::statement::{Block, ForStatement, Statement};

pub struct DeadCodeEliminationPass;

impl DeadCodeEliminationPass {
    pub fn new() -> Self {
        Self
    }
}

impl<'arena> BlockVisitor<'arena> for DeadCodeEliminationPass {
    fn visit_block_stmts(
        &mut self,
        stmts: &mut Vec<Statement<'arena>>,
        arena: &'arena Bump,
    ) -> bool {
        self.eliminate_dead_code_vec(stmts, arena)
    }
}

impl DeadCodeEliminationPass {
    fn eliminate_dead_code_vec<'arena>(
        &mut self,
        stmts: &mut Vec<Statement<'arena>>,
        arena: &'arena Bump,
    ) -> bool {
        let mut changed = false;
        let mut i = 0;

        while i < stmts.len() {
            let is_terminal = matches!(
                stmts[i],
                Statement::Return(_) | Statement::Break(_) | Statement::Continue(_)
            );

            if is_terminal {
                let new_len = i + 1;
                if stmts.len() > new_len {
                    stmts.truncate(new_len);
                    changed = true;
                }
                break;
            }

            changed |= self.eliminate_in_stmt(&mut stmts[i], arena);

            i += 1;
        }

        changed
    }

    fn eliminate_in_stmt<'arena>(
        &mut self,
        stmt: &mut Statement<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        match stmt {
            Statement::If(if_stmt) => {
                let mut local_changed = self.eliminate_in_block(&mut if_stmt.then_block, arena);
                let mut new_else_ifs: Vec<_> = if_stmt.else_ifs.to_vec();
                let mut eic = false;
                for else_if in &mut new_else_ifs {
                    eic |= self.eliminate_in_block(&mut else_if.block, arena);
                }
                if eic {
                    if_stmt.else_ifs = arena.alloc_slice_clone(&new_else_ifs);
                    local_changed = true;
                }
                if let Some(else_block) = &mut if_stmt.else_block {
                    local_changed |= self.eliminate_in_block(else_block, arena);
                }
                local_changed
            }
            Statement::While(while_stmt) => self.eliminate_in_block(&mut while_stmt.body, arena),
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num_ref) => {
                    let mut new_num = (**for_num_ref).clone();
                    let fc = self.eliminate_in_block(&mut new_num.body, arena);
                    if fc {
                        *stmt = Statement::For(
                            arena.alloc(ForStatement::Numeric(arena.alloc(new_num))),
                        );
                    }
                    fc
                }
                ForStatement::Generic(for_gen_ref) => {
                    let mut new_gen = for_gen_ref.clone();
                    let fc = self.eliminate_in_block(&mut new_gen.body, arena);
                    if fc {
                        *stmt = Statement::For(arena.alloc(ForStatement::Generic(new_gen)));
                    }
                    fc
                }
            },
            Statement::Function(func) => self.eliminate_in_block(&mut func.body, arena),
            _ => false,
        }
    }

    fn eliminate_in_block<'arena>(
        &mut self,
        block: &mut Block<'arena>,
        arena: &'arena Bump,
    ) -> bool {
        let mut stmts: Vec<_> = block.statements.to_vec();
        let changed = self.eliminate_dead_code_vec(&mut stmts, arena);
        if changed {
            block.statements = arena.alloc_slice_clone(&stmts);
        }
        changed
    }
}

impl Default for DeadCodeEliminationPass {
    fn default() -> Self {
        Self::new()
    }
}
