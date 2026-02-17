//! Control Flow Graph (CFG) construction from AST.
//!
//! Builds basic blocks and edges from a function body or top-level statement list.
//! Required for: jump threading, SCCP, advanced dead code elimination.
//!
//! Design: Uses `usize` statement indices (not `&'arena` references) to stay
//! decoupled from arena lifetimes. Analysis consumers look up actual AST nodes
//! via `program.statements[index]` when needed.

use luanext_parser::ast::statement::{ForStatement, Statement};
use luanext_parser::span::Span;
use luanext_parser::string_interner::StringId;
use rustc_hash::FxHashMap;

/// Unique identifier for a basic block within a CFG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

impl BlockId {
    /// The entry block — control flow begins here.
    pub const ENTRY: BlockId = BlockId(0);
    /// The exit block — control flow ends here (function return / fall-through).
    pub const EXIT: BlockId = BlockId(1);
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "B{}", self.0)
    }
}

/// A basic block: a maximal sequence of statements with no internal branching.
///
/// Statements within a block execute sequentially; control flow enters at the
/// top and leaves via the terminator.
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// This block's unique identifier.
    pub id: BlockId,
    /// Indices into the source statement list. These reference the original AST
    /// by position, avoiding lifetime coupling with arena-allocated nodes.
    pub statement_indices: Vec<usize>,
    /// The span covering all statements in this block.
    pub span: Span,
    /// How this block terminates (where control goes next).
    pub terminator: Terminator,
}

/// How control leaves a basic block.
#[derive(Debug, Clone)]
pub enum Terminator {
    /// Unconditional jump to another block.
    Goto(BlockId),
    /// Conditional branch: if condition is truthy go to `true_target`, else `false_target`.
    Branch {
        /// Index of the statement containing the condition expression.
        condition_stmt_index: usize,
        /// Target block when condition is true.
        true_target: BlockId,
        /// Target block when condition is false.
        false_target: BlockId,
    },
    /// Return from function.
    Return,
    /// Unreachable code (dead code after return/break/continue).
    Unreachable,
    /// Loop back-edge: a `Goto` that represents a loop iteration.
    LoopBack(BlockId),
    /// Fall-through to exit (function body without explicit return).
    FallThrough,
    /// Exception handling: normal flow continues to `normal`, exceptions
    /// go to `catch_targets`.
    TryCatch {
        /// Normal continuation block.
        normal: BlockId,
        /// Catch handler blocks.
        catch_targets: Vec<BlockId>,
    },
}

/// The control flow graph for a single function or top-level scope.
#[derive(Debug)]
pub struct ControlFlowGraph {
    /// All basic blocks, indexed by `BlockId`.
    pub blocks: Vec<BasicBlock>,
    /// Predecessor map: block -> list of blocks that can jump here.
    pub predecessors: FxHashMap<BlockId, Vec<BlockId>>,
    /// Successor map: block -> list of blocks reachable from here.
    pub successors: FxHashMap<BlockId, Vec<BlockId>>,
    /// Map from statement index to the block containing it.
    pub stmt_to_block: FxHashMap<usize, BlockId>,
    /// Block IDs identified as loop headers (targets of back-edges).
    pub loop_headers: Vec<BlockId>,
}

impl ControlFlowGraph {
    /// Returns the number of basic blocks (including ENTRY and EXIT sentinels).
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Returns the block with the given ID, if it exists.
    pub fn block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(id.0 as usize)
    }

    /// Returns the predecessors of a block.
    pub fn preds(&self, id: BlockId) -> &[BlockId] {
        self.predecessors
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns the successors of a block.
    pub fn succs(&self, id: BlockId) -> &[BlockId] {
        self.successors
            .get(&id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Returns blocks in reverse postorder (useful for dataflow algorithms).
    pub fn reverse_postorder(&self) -> Vec<BlockId> {
        let mut visited = vec![false; self.blocks.len()];
        let mut postorder = Vec::with_capacity(self.blocks.len());
        self.dfs_postorder(BlockId::ENTRY, &mut visited, &mut postorder);
        postorder.reverse();
        postorder
    }

    fn dfs_postorder(&self, block: BlockId, visited: &mut Vec<bool>, postorder: &mut Vec<BlockId>) {
        let idx = block.0 as usize;
        if idx >= visited.len() || visited[idx] {
            return;
        }
        visited[idx] = true;

        for &succ in self.succs(block) {
            self.dfs_postorder(succ, visited, postorder);
        }
        postorder.push(block);
    }
}

/// Builder that constructs a CFG from a sequence of statements.
pub struct CfgBuilder {
    blocks: Vec<BasicBlock>,
    next_id: u32,
    /// Stack of (loop_header_block, loop_exit_block) for break/continue resolution.
    loop_stack: Vec<(BlockId, BlockId)>,
    /// Map from label name to the block that starts at that label.
    label_map: FxHashMap<StringId, BlockId>,
}

impl CfgBuilder {
    /// Build a CFG from a flat list of statements (function body or top-level scope).
    pub fn build(statements: &[Statement<'_>]) -> ControlFlowGraph {
        let mut builder = CfgBuilder {
            blocks: Vec::new(),
            next_id: 0,
            loop_stack: Vec::new(),
            label_map: FxHashMap::default(),
        };

        // Create ENTRY and EXIT sentinel blocks
        let entry = builder.new_block();
        debug_assert_eq!(entry, BlockId::ENTRY);
        let exit = builder.new_block();
        debug_assert_eq!(exit, BlockId::EXIT);

        // First pass: pre-create blocks for all labels
        for (i, stmt) in statements.iter().enumerate() {
            if let Statement::Label(label) = stmt {
                let label_block = builder.new_block();
                builder.label_map.insert(label.name.node, label_block);
                // The label statement itself will be in this block
                builder.blocks[label_block.0 as usize]
                    .statement_indices
                    .push(i);
                builder.blocks[label_block.0 as usize].span = label.span;
            }
        }

        // Create the first real code block (after ENTRY)
        let first_code_block = builder.new_block();
        builder.set_terminator(BlockId::ENTRY, Terminator::Goto(first_code_block));

        // Second pass: walk statements and build blocks
        let mut current = first_code_block;
        for (i, stmt) in statements.iter().enumerate() {
            current = builder.process_statement(i, stmt, current);
        }

        // Final block falls through to EXIT
        if builder.blocks[current.0 as usize].terminator_is_open() {
            builder.set_terminator(current, Terminator::FallThrough);
        }

        // Set EXIT as having a Return terminator (sentinel)
        builder.set_terminator(BlockId::EXIT, Terminator::Return);

        builder.finalize()
    }

    fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.next_id);
        self.next_id += 1;
        self.blocks.push(BasicBlock {
            id,
            statement_indices: Vec::new(),
            span: Span::dummy(),
            terminator: Terminator::Unreachable,
        });
        id
    }

    fn set_terminator(&mut self, block: BlockId, term: Terminator) {
        self.blocks[block.0 as usize].terminator = term;
    }

    fn add_stmt_to_block(&mut self, block: BlockId, stmt_index: usize, span: Span) {
        let b = &mut self.blocks[block.0 as usize];
        b.statement_indices.push(stmt_index);
        if b.span.start == 0 && b.span.end == 0 {
            b.span = span;
        } else {
            b.span = b.span.merge(&span);
        }
    }

    /// Process a single statement, potentially splitting/creating blocks.
    /// Returns the block ID that subsequent statements should be added to.
    fn process_statement(
        &mut self,
        index: usize,
        stmt: &Statement<'_>,
        current: BlockId,
    ) -> BlockId {
        // If current block already has a terminator, start a new (dead) block
        if !self.blocks[current.0 as usize].terminator_is_open() {
            let dead = self.new_block();
            // Dead code — no edge from current to dead
            return self.process_statement(index, stmt, dead);
        }

        match stmt {
            // Control flow: If/ElseIf/Else
            Statement::If(if_stmt) => {
                self.add_stmt_to_block(current, index, if_stmt.span);

                let join_block = self.new_block();

                // Then branch
                let then_block = self.new_block();
                let then_end = self.build_block_stmts(&if_stmt.then_block, then_block);
                if self.blocks[then_end.0 as usize].terminator_is_open() {
                    self.set_terminator(then_end, Terminator::Goto(join_block));
                }

                // Determine false target: first else-if, else block, or join
                if !if_stmt.else_ifs.is_empty() {
                    let first_elseif_block = self.new_block();
                    self.set_terminator(
                        current,
                        Terminator::Branch {
                            condition_stmt_index: index,
                            true_target: then_block,
                            false_target: first_elseif_block,
                        },
                    );

                    // Process else-if chain
                    let mut prev_false = first_elseif_block;
                    for (ei_idx, else_if) in if_stmt.else_ifs.iter().enumerate() {
                        let ei_body = self.new_block();
                        let ei_end = self.build_block_stmts(&else_if.block, ei_body);
                        if self.blocks[ei_end.0 as usize].terminator_is_open() {
                            self.set_terminator(ei_end, Terminator::Goto(join_block));
                        }

                        let next_false = if ei_idx + 1 < if_stmt.else_ifs.len() {
                            self.new_block()
                        } else if let Some(else_block) = &if_stmt.else_block {
                            let eb = self.new_block();
                            let eb_end = self.build_block_stmts(else_block, eb);
                            if self.blocks[eb_end.0 as usize].terminator_is_open() {
                                self.set_terminator(eb_end, Terminator::Goto(join_block));
                            }
                            eb
                        } else {
                            join_block
                        };

                        self.set_terminator(
                            prev_false,
                            Terminator::Branch {
                                condition_stmt_index: index,
                                true_target: ei_body,
                                false_target: next_false,
                            },
                        );

                        if ei_idx + 1 < if_stmt.else_ifs.len() {
                            prev_false = next_false;
                        }
                    }
                } else if let Some(else_block) = &if_stmt.else_block {
                    let else_bb = self.new_block();
                    let else_end = self.build_block_stmts(else_block, else_bb);
                    if self.blocks[else_end.0 as usize].terminator_is_open() {
                        self.set_terminator(else_end, Terminator::Goto(join_block));
                    }
                    self.set_terminator(
                        current,
                        Terminator::Branch {
                            condition_stmt_index: index,
                            true_target: then_block,
                            false_target: else_bb,
                        },
                    );
                } else {
                    // No else — false goes straight to join
                    self.set_terminator(
                        current,
                        Terminator::Branch {
                            condition_stmt_index: index,
                            true_target: then_block,
                            false_target: join_block,
                        },
                    );
                }

                join_block
            }

            // Control flow: While loop
            Statement::While(while_stmt) => {
                let header = self.new_block();
                let body = self.new_block();
                let exit = self.new_block();

                // Current -> header
                self.set_terminator(current, Terminator::Goto(header));

                // Header: condition check
                self.blocks[header.0 as usize].span = while_stmt.span;
                self.set_terminator(
                    header,
                    Terminator::Branch {
                        condition_stmt_index: index,
                        true_target: body,
                        false_target: exit,
                    },
                );

                // Body -> header (back-edge)
                self.loop_stack.push((header, exit));
                let body_end = self.build_block_stmts(&while_stmt.body, body);
                self.loop_stack.pop();
                if self.blocks[body_end.0 as usize].terminator_is_open() {
                    self.set_terminator(body_end, Terminator::LoopBack(header));
                }

                self.add_stmt_to_block(current, index, while_stmt.span);
                exit
            }

            // Control flow: For loops
            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num) => {
                    let header = self.new_block();
                    let body = self.new_block();
                    let exit = self.new_block();

                    self.add_stmt_to_block(current, index, for_num.span);
                    self.set_terminator(current, Terminator::Goto(header));

                    self.blocks[header.0 as usize].span = for_num.span;
                    self.set_terminator(
                        header,
                        Terminator::Branch {
                            condition_stmt_index: index,
                            true_target: body,
                            false_target: exit,
                        },
                    );

                    self.loop_stack.push((header, exit));
                    let body_end = self.build_block_stmts(&for_num.body, body);
                    self.loop_stack.pop();
                    if self.blocks[body_end.0 as usize].terminator_is_open() {
                        self.set_terminator(body_end, Terminator::LoopBack(header));
                    }

                    exit
                }
                ForStatement::Generic(for_gen) => {
                    let header = self.new_block();
                    let body = self.new_block();
                    let exit = self.new_block();

                    self.add_stmt_to_block(current, index, for_gen.span);
                    self.set_terminator(current, Terminator::Goto(header));

                    self.blocks[header.0 as usize].span = for_gen.span;
                    self.set_terminator(
                        header,
                        Terminator::Branch {
                            condition_stmt_index: index,
                            true_target: body,
                            false_target: exit,
                        },
                    );

                    self.loop_stack.push((header, exit));
                    let body_end = self.build_block_stmts(&for_gen.body, body);
                    self.loop_stack.pop();
                    if self.blocks[body_end.0 as usize].terminator_is_open() {
                        self.set_terminator(body_end, Terminator::LoopBack(header));
                    }

                    exit
                }
            },

            // Control flow: Repeat...until
            Statement::Repeat(repeat_stmt) => {
                let body = self.new_block();
                let exit = self.new_block();

                self.add_stmt_to_block(current, index, repeat_stmt.span);
                self.set_terminator(current, Terminator::Goto(body));

                // In repeat...until, the body block IS the loop header
                self.loop_stack.push((body, exit));
                let body_end = self.build_block_stmts(&repeat_stmt.body, body);
                self.loop_stack.pop();

                // After body: check condition
                if self.blocks[body_end.0 as usize].terminator_is_open() {
                    // until <condition>: if true -> exit, if false -> loop back
                    self.set_terminator(
                        body_end,
                        Terminator::Branch {
                            condition_stmt_index: index,
                            true_target: exit,
                            false_target: body, // back-edge to body start
                        },
                    );
                }

                exit
            }

            // Control flow: Return
            Statement::Return(ret_stmt) => {
                self.add_stmt_to_block(current, index, ret_stmt.span);
                self.set_terminator(current, Terminator::Return);
                // Return a new block for any dead code after return
                self.new_block()
            }

            // Control flow: Break
            Statement::Break(span) => {
                self.add_stmt_to_block(current, index, *span);
                if let Some(&(_, exit)) = self.loop_stack.last() {
                    self.set_terminator(current, Terminator::Goto(exit));
                } else {
                    // Break outside loop — unreachable in valid code, but handle gracefully
                    self.set_terminator(current, Terminator::Unreachable);
                }
                self.new_block()
            }

            // Control flow: Continue
            Statement::Continue(span) => {
                self.add_stmt_to_block(current, index, *span);
                if let Some(&(header, _)) = self.loop_stack.last() {
                    self.set_terminator(current, Terminator::LoopBack(header));
                } else {
                    self.set_terminator(current, Terminator::Unreachable);
                }
                self.new_block()
            }

            // Control flow: Goto
            Statement::Goto(goto_stmt) => {
                self.add_stmt_to_block(current, index, goto_stmt.span);
                if let Some(&target) = self.label_map.get(&goto_stmt.target.node) {
                    self.set_terminator(current, Terminator::Goto(target));
                } else {
                    // Unknown label — unreachable in valid code
                    self.set_terminator(current, Terminator::Unreachable);
                }
                self.new_block()
            }

            // Control flow: Label
            Statement::Label(label) => {
                if let Some(&label_block) = self.label_map.get(&label.name.node) {
                    // End current block with goto to label block
                    if self.blocks[current.0 as usize].terminator_is_open() {
                        self.set_terminator(current, Terminator::Goto(label_block));
                    }
                    // Statement was already added during pre-scan
                    label_block
                } else {
                    // Should not happen — labels are pre-scanned
                    self.add_stmt_to_block(current, index, label.span);
                    current
                }
            }

            // Control flow: Try/Catch/Finally
            Statement::Try(try_stmt) => {
                let try_body = self.new_block();
                let join = self.new_block();

                // Build catch target blocks
                let mut catch_targets = Vec::with_capacity(try_stmt.catch_clauses.len());
                for catch_clause in try_stmt.catch_clauses.iter() {
                    let catch_block = self.new_block();
                    let catch_end = self.build_block_stmts(&catch_clause.body, catch_block);
                    if self.blocks[catch_end.0 as usize].terminator_is_open() {
                        self.set_terminator(catch_end, Terminator::Goto(join));
                    }
                    catch_targets.push(catch_block);
                }

                // Try body → normal continuation or catch targets
                let try_end = self.build_block_stmts(&try_stmt.try_block, try_body);
                let normal_cont = if let Some(finally_block) = &try_stmt.finally_block {
                    let finally_bb = self.new_block();
                    let finally_end = self.build_block_stmts(finally_block, finally_bb);
                    if self.blocks[finally_end.0 as usize].terminator_is_open() {
                        self.set_terminator(finally_end, Terminator::Goto(join));
                    }
                    finally_bb
                } else {
                    join
                };

                if self.blocks[try_end.0 as usize].terminator_is_open() {
                    self.set_terminator(try_end, Terminator::Goto(normal_cont));
                }

                self.add_stmt_to_block(current, index, try_stmt.span);
                self.set_terminator(
                    current,
                    Terminator::TryCatch {
                        normal: try_body,
                        catch_targets,
                    },
                );

                join
            }

            // Control flow: Throw (terminates block like return)
            Statement::Throw(throw_stmt) => {
                self.add_stmt_to_block(current, index, throw_stmt.span);
                self.set_terminator(current, Terminator::Unreachable);
                self.new_block()
            }

            // Control flow: Rethrow (terminates block)
            Statement::Rethrow(span) => {
                self.add_stmt_to_block(current, index, *span);
                self.set_terminator(current, Terminator::Unreachable);
                self.new_block()
            }

            // Block (do...end) — recurse into sub-statements
            Statement::Block(block) => {
                self.add_stmt_to_block(current, index, block.span);
                self.build_block_stmts(block, current)
            }

            // Non-control-flow statements: just add to current block
            Statement::Variable(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::Function(func) => {
                self.add_stmt_to_block(current, index, func.span);
                current
            }
            Statement::Expression(expr) => {
                self.add_stmt_to_block(current, index, expr.span);
                current
            }
            Statement::Class(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::Interface(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::TypeAlias(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::Enum(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::Import(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::Export(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::Namespace(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::DeclareFunction(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::DeclareNamespace(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::DeclareType(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::DeclareInterface(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::DeclareConst(decl) => {
                self.add_stmt_to_block(current, index, decl.span);
                current
            }
            Statement::MultiAssignment(multi) => {
                self.add_stmt_to_block(current, index, multi.span);
                current
            }
        }
    }

    /// Build CFG blocks from the statements inside a Block AST node.
    ///
    /// Note: The statements in a `Block` are `&'arena [Statement]` (a sub-slice
    /// of arena-allocated nodes), not part of the top-level statement list. We use
    /// a synthetic index scheme — statements within sub-blocks get indices relative
    /// to the parent scope. This is handled by encoding the parent statement index
    /// in the block itself.
    /// Process a block's statements, creating CFG blocks as needed.
    /// Returns the "current" block after processing — the block where
    /// subsequent statements would be placed (may differ from `into` if
    /// control-flow statements created new blocks).
    fn build_block_stmts(
        &mut self,
        block: &luanext_parser::ast::statement::Block<'_>,
        into: BlockId,
    ) -> BlockId {
        // For sub-blocks, we process statements in sequence but they don't have
        // direct indices into the top-level list. We use the block's span to
        // track them without indexing.
        let mut current = into;
        for stmt in block.statements.iter() {
            // Sub-block statements don't get statement indices (they're nested)
            // We still model their control flow for the CFG
            match stmt {
                Statement::If(_)
                | Statement::While(_)
                | Statement::For(_)
                | Statement::Repeat(_)
                | Statement::Try(_) => {
                    // These create new blocks within the sub-scope
                    current = self.process_nested_statement(stmt, current);
                }
                Statement::Return(ret) => {
                    self.blocks[current.0 as usize].span =
                        self.blocks[current.0 as usize].span.merge(&ret.span);
                    self.set_terminator(current, Terminator::Return);
                    current = self.new_block();
                }
                Statement::Break(span) => {
                    self.blocks[current.0 as usize].span =
                        self.blocks[current.0 as usize].span.merge(span);
                    if let Some(&(_, exit)) = self.loop_stack.last() {
                        self.set_terminator(current, Terminator::Goto(exit));
                    } else {
                        self.set_terminator(current, Terminator::Unreachable);
                    }
                    current = self.new_block();
                }
                Statement::Continue(span) => {
                    self.blocks[current.0 as usize].span =
                        self.blocks[current.0 as usize].span.merge(span);
                    if let Some(&(header, _)) = self.loop_stack.last() {
                        self.set_terminator(current, Terminator::LoopBack(header));
                    } else {
                        self.set_terminator(current, Terminator::Unreachable);
                    }
                    current = self.new_block();
                }
                Statement::Goto(goto_stmt) => {
                    if let Some(&target) = self.label_map.get(&goto_stmt.target.node) {
                        self.set_terminator(current, Terminator::Goto(target));
                    } else {
                        self.set_terminator(current, Terminator::Unreachable);
                    }
                    current = self.new_block();
                }
                Statement::Throw(_) | Statement::Rethrow(_) => {
                    self.set_terminator(current, Terminator::Unreachable);
                    current = self.new_block();
                }
                Statement::Block(inner_block) => {
                    current = self.build_block_stmts(inner_block, current);
                }
                // Non-control-flow: just extend the span
                _ => {
                    let stmt_span = statement_span(stmt);
                    let b = &mut self.blocks[current.0 as usize];
                    if b.span.start == 0 && b.span.end == 0 {
                        b.span = stmt_span;
                    } else {
                        b.span = b.span.merge(&stmt_span);
                    }
                }
            }
        }
        current
    }

    /// Process a nested control-flow statement (inside a sub-block).
    fn process_nested_statement(&mut self, stmt: &Statement<'_>, current: BlockId) -> BlockId {
        match stmt {
            Statement::If(if_stmt) => {
                let join_block = self.new_block();

                let then_block = self.new_block();
                let then_end = self.build_block_stmts(&if_stmt.then_block, then_block);
                if self.blocks[then_end.0 as usize].terminator_is_open() {
                    self.set_terminator(then_end, Terminator::Goto(join_block));
                }

                if !if_stmt.else_ifs.is_empty() {
                    let first_elseif = self.new_block();
                    self.set_terminator(
                        current,
                        Terminator::Branch {
                            condition_stmt_index: 0, // Nested — no top-level index
                            true_target: then_block,
                            false_target: first_elseif,
                        },
                    );

                    let mut prev_false = first_elseif;
                    for (ei_idx, else_if) in if_stmt.else_ifs.iter().enumerate() {
                        let ei_body = self.new_block();
                        let ei_end = self.build_block_stmts(&else_if.block, ei_body);
                        if self.blocks[ei_end.0 as usize].terminator_is_open() {
                            self.set_terminator(ei_end, Terminator::Goto(join_block));
                        }

                        let next_false = if ei_idx + 1 < if_stmt.else_ifs.len() {
                            self.new_block()
                        } else if let Some(else_block) = &if_stmt.else_block {
                            let eb = self.new_block();
                            let eb_end = self.build_block_stmts(else_block, eb);
                            if self.blocks[eb_end.0 as usize].terminator_is_open() {
                                self.set_terminator(eb_end, Terminator::Goto(join_block));
                            }
                            eb
                        } else {
                            join_block
                        };

                        self.set_terminator(
                            prev_false,
                            Terminator::Branch {
                                condition_stmt_index: 0,
                                true_target: ei_body,
                                false_target: next_false,
                            },
                        );

                        if ei_idx + 1 < if_stmt.else_ifs.len() {
                            prev_false = next_false;
                        }
                    }
                } else if let Some(else_block) = &if_stmt.else_block {
                    let else_bb = self.new_block();
                    let else_end = self.build_block_stmts(else_block, else_bb);
                    if self.blocks[else_end.0 as usize].terminator_is_open() {
                        self.set_terminator(else_end, Terminator::Goto(join_block));
                    }
                    self.set_terminator(
                        current,
                        Terminator::Branch {
                            condition_stmt_index: 0,
                            true_target: then_block,
                            false_target: else_bb,
                        },
                    );
                } else {
                    self.set_terminator(
                        current,
                        Terminator::Branch {
                            condition_stmt_index: 0,
                            true_target: then_block,
                            false_target: join_block,
                        },
                    );
                }

                join_block
            }

            Statement::While(while_stmt) => {
                let header = self.new_block();
                let body = self.new_block();
                let exit = self.new_block();

                self.set_terminator(current, Terminator::Goto(header));
                self.blocks[header.0 as usize].span = while_stmt.span;
                self.set_terminator(
                    header,
                    Terminator::Branch {
                        condition_stmt_index: 0,
                        true_target: body,
                        false_target: exit,
                    },
                );

                self.loop_stack.push((header, exit));
                let body_end = self.build_block_stmts(&while_stmt.body, body);
                self.loop_stack.pop();
                if self.blocks[body_end.0 as usize].terminator_is_open() {
                    self.set_terminator(body_end, Terminator::LoopBack(header));
                }

                exit
            }

            Statement::For(for_stmt) => match &**for_stmt {
                ForStatement::Numeric(for_num) => {
                    let header = self.new_block();
                    let body = self.new_block();
                    let exit = self.new_block();

                    self.set_terminator(current, Terminator::Goto(header));
                    self.blocks[header.0 as usize].span = for_num.span;
                    self.set_terminator(
                        header,
                        Terminator::Branch {
                            condition_stmt_index: 0,
                            true_target: body,
                            false_target: exit,
                        },
                    );

                    self.loop_stack.push((header, exit));
                    let body_end = self.build_block_stmts(&for_num.body, body);
                    self.loop_stack.pop();
                    if self.blocks[body_end.0 as usize].terminator_is_open() {
                        self.set_terminator(body_end, Terminator::LoopBack(header));
                    }

                    exit
                }
                ForStatement::Generic(for_gen) => {
                    let header = self.new_block();
                    let body = self.new_block();
                    let exit = self.new_block();

                    self.set_terminator(current, Terminator::Goto(header));
                    self.blocks[header.0 as usize].span = for_gen.span;
                    self.set_terminator(
                        header,
                        Terminator::Branch {
                            condition_stmt_index: 0,
                            true_target: body,
                            false_target: exit,
                        },
                    );

                    self.loop_stack.push((header, exit));
                    let body_end = self.build_block_stmts(&for_gen.body, body);
                    self.loop_stack.pop();
                    if self.blocks[body_end.0 as usize].terminator_is_open() {
                        self.set_terminator(body_end, Terminator::LoopBack(header));
                    }

                    exit
                }
            },

            Statement::Repeat(repeat_stmt) => {
                let body = self.new_block();
                let exit = self.new_block();

                self.set_terminator(current, Terminator::Goto(body));

                self.loop_stack.push((body, exit));
                let body_end = self.build_block_stmts(&repeat_stmt.body, body);
                self.loop_stack.pop();

                if self.blocks[body_end.0 as usize].terminator_is_open() {
                    self.set_terminator(
                        body_end,
                        Terminator::Branch {
                            condition_stmt_index: 0,
                            true_target: exit,
                            false_target: body, // back-edge to body start
                        },
                    );
                }

                exit
            }

            Statement::Try(try_stmt) => {
                let try_body = self.new_block();
                let join = self.new_block();

                let mut catch_targets = Vec::with_capacity(try_stmt.catch_clauses.len());
                for catch_clause in try_stmt.catch_clauses.iter() {
                    let catch_block = self.new_block();
                    let catch_end = self.build_block_stmts(&catch_clause.body, catch_block);
                    if self.blocks[catch_end.0 as usize].terminator_is_open() {
                        self.set_terminator(catch_end, Terminator::Goto(join));
                    }
                    catch_targets.push(catch_block);
                }

                let try_end = self.build_block_stmts(&try_stmt.try_block, try_body);
                let normal_cont = if let Some(finally_block) = &try_stmt.finally_block {
                    let finally_bb = self.new_block();
                    let finally_end = self.build_block_stmts(finally_block, finally_bb);
                    if self.blocks[finally_end.0 as usize].terminator_is_open() {
                        self.set_terminator(finally_end, Terminator::Goto(join));
                    }
                    finally_bb
                } else {
                    join
                };

                if self.blocks[try_end.0 as usize].terminator_is_open() {
                    self.set_terminator(try_end, Terminator::Goto(normal_cont));
                }

                self.set_terminator(
                    current,
                    Terminator::TryCatch {
                        normal: try_body,
                        catch_targets,
                    },
                );

                join
            }

            _ => current,
        }
    }

    /// Finalize: compute predecessors, successors, stmt_to_block, and loop_headers.
    fn finalize(self) -> ControlFlowGraph {
        let mut successors: FxHashMap<BlockId, Vec<BlockId>> = FxHashMap::default();
        let mut predecessors: FxHashMap<BlockId, Vec<BlockId>> = FxHashMap::default();
        let mut stmt_to_block: FxHashMap<usize, BlockId> = FxHashMap::default();
        let mut loop_headers: Vec<BlockId> = Vec::new();

        // Initialize empty entries for all blocks
        for block in &self.blocks {
            successors.insert(block.id, Vec::new());
            predecessors.insert(block.id, Vec::new());

            // Build stmt_to_block
            for &idx in &block.statement_indices {
                stmt_to_block.insert(idx, block.id);
            }
        }

        // Compute successors and predecessors from terminators
        for block in &self.blocks {
            let succs = terminator_targets(&block.terminator);
            for &target in &succs {
                successors.get_mut(&block.id).unwrap().push(target);
                predecessors.entry(target).or_default().push(block.id);
            }

            // Detect loop headers from LoopBack terminators
            if let Terminator::LoopBack(header) = &block.terminator {
                if !loop_headers.contains(header) {
                    loop_headers.push(*header);
                }
            }
            // Also detect from Branch with self-referencing (repeat...until)
            if let Terminator::Branch { false_target, .. } = &block.terminator {
                if *false_target == block.id && !loop_headers.contains(&block.id) {
                    loop_headers.push(block.id);
                }
            }
        }

        ControlFlowGraph {
            blocks: self.blocks,
            predecessors,
            successors,
            stmt_to_block,
            loop_headers,
        }
    }
}

impl BasicBlock {
    /// Returns true if this block's terminator hasn't been explicitly set yet.
    ///
    /// New blocks start with `Unreachable` and `is_open = true`. Once a
    /// terminator is explicitly set via `set_terminator`, the block is closed.
    fn terminator_is_open(&self) -> bool {
        // We use a simple flag approach: all blocks start as Unreachable,
        // and we track whether the terminator was explicitly set.
        // However, since we don't have a separate flag, we rely on the
        // convention that Unreachable is only set explicitly for throw/rethrow
        // which also have statements. A fresh block has Unreachable + no stmts.
        // For simplicity, we track openness by whether statement_indices is empty
        // AND terminator is Unreachable (fresh block), OR check explicitly.
        //
        // Actually, the simplest correct approach: a block is "open" if its
        // terminator is Unreachable. When we explicitly want Unreachable (throw),
        // we set it and immediately create a new block, so we never try to add
        // more statements to a thrown block.
        matches!(self.terminator, Terminator::Unreachable)
    }
}

/// Extract all target block IDs from a terminator.
fn terminator_targets(term: &Terminator) -> Vec<BlockId> {
    match term {
        Terminator::Goto(target) => vec![*target],
        Terminator::Branch {
            true_target,
            false_target,
            ..
        } => vec![*true_target, *false_target],
        Terminator::LoopBack(target) => vec![*target],
        Terminator::FallThrough => vec![BlockId::EXIT],
        Terminator::TryCatch {
            normal,
            catch_targets,
        } => {
            let mut targets = vec![*normal];
            targets.extend_from_slice(catch_targets);
            targets
        }
        Terminator::Return => vec![BlockId::EXIT],
        Terminator::Unreachable => vec![],
    }
}

/// Extract the span from any statement variant.
fn statement_span(stmt: &Statement<'_>) -> Span {
    match stmt {
        Statement::Variable(d) => d.span,
        Statement::Function(d) => d.span,
        Statement::Class(d) => d.span,
        Statement::Interface(d) => d.span,
        Statement::TypeAlias(d) => d.span,
        Statement::Enum(d) => d.span,
        Statement::Import(d) => d.span,
        Statement::Export(d) => d.span,
        Statement::If(d) => d.span,
        Statement::While(d) => d.span,
        Statement::For(f) => match &**f {
            ForStatement::Numeric(n) => n.span,
            ForStatement::Generic(g) => g.span,
        },
        Statement::Repeat(d) => d.span,
        Statement::Return(d) => d.span,
        Statement::Break(s) => *s,
        Statement::Continue(s) => *s,
        Statement::Label(d) => d.span,
        Statement::Goto(d) => d.span,
        Statement::Expression(e) => e.span,
        Statement::Block(b) => b.span,
        Statement::Throw(d) => d.span,
        Statement::Try(d) => d.span,
        Statement::Rethrow(s) => *s,
        Statement::Namespace(d) => d.span,
        Statement::DeclareFunction(d) => d.span,
        Statement::DeclareNamespace(d) => d.span,
        Statement::DeclareType(d) => d.span,
        Statement::DeclareInterface(d) => d.span,
        Statement::DeclareConst(d) => d.span,
        Statement::MultiAssignment(m) => m.span,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luanext_parser::ast::expression::{Expression, ExpressionKind, Literal};
    use luanext_parser::ast::statement::{
        Block, ForGeneric, ForNumeric, IfStatement, RepeatStatement, ReturnStatement,
        VariableDeclaration, VariableKind, WhileStatement,
    };
    use luanext_parser::ast::Ident;
    use luanext_parser::string_interner::StringInterner;

    fn make_ident(interner: &StringInterner, name: &str) -> Ident {
        Ident {
            node: interner.get_or_intern(name),
            span: Span::dummy(),
        }
    }

    fn make_expr_literal_true() -> Expression<'static> {
        Expression {
            kind: ExpressionKind::Literal(Literal::Boolean(true)),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        }
    }

    fn make_expr_nil() -> Expression<'static> {
        Expression {
            kind: ExpressionKind::Literal(Literal::Nil),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        }
    }

    fn make_var_decl<'a>(interner: &StringInterner, name: &str) -> Statement<'a> {
        Statement::Variable(VariableDeclaration {
            kind: VariableKind::Local,
            pattern: luanext_parser::ast::pattern::Pattern::Identifier(make_ident(interner, name)),
            type_annotation: None,
            initializer: make_expr_nil(),
            span: Span::new(0, 10, 1, 1),
        })
    }

    fn empty_block() -> Block<'static> {
        Block {
            statements: &[],
            span: Span::dummy(),
        }
    }

    #[test]
    fn test_linear_code_single_block() {
        let interner = StringInterner::new();
        let stmts = vec![
            make_var_decl(&interner, "x"),
            make_var_decl(&interner, "y"),
            make_var_decl(&interner, "z"),
        ];

        let cfg = CfgBuilder::build(&stmts);

        // ENTRY -> code_block -> EXIT
        assert!(cfg.block_count() >= 3);
        // ENTRY has Goto to code block
        assert!(matches!(
            cfg.block(BlockId::ENTRY).unwrap().terminator,
            Terminator::Goto(_)
        ));
        // All 3 statements should be in the same block
        let code_block_id = BlockId(2);
        assert_eq!(cfg.block(code_block_id).unwrap().statement_indices.len(), 3);
        // Code block falls through to EXIT
        assert!(matches!(
            cfg.block(code_block_id).unwrap().terminator,
            Terminator::FallThrough
        ));
    }

    #[test]
    fn test_return_terminates_block() {
        let interner = StringInterner::new();
        let stmts = vec![
            make_var_decl(&interner, "x"),
            Statement::Return(ReturnStatement {
                values: &[],
                span: Span::new(10, 20, 2, 1),
            }),
            make_var_decl(&interner, "dead"), // dead code after return
        ];

        let cfg = CfgBuilder::build(&stmts);

        // Find the block with the return
        let mut found_return = false;
        for block in &cfg.blocks {
            if matches!(block.terminator, Terminator::Return) && block.id != BlockId::EXIT {
                found_return = true;
                // Should have 2 statements (x and return)
                assert_eq!(block.statement_indices.len(), 2);
            }
        }
        assert!(found_return, "Should have a block with Return terminator");
    }

    #[test]
    fn test_if_else_diamond() {
        // if cond then ... else ... end
        // Creates a diamond pattern: current -> then/else -> join
        let stmts = vec![Statement::If(IfStatement {
            condition: make_expr_literal_true(),
            then_block: empty_block(),
            else_ifs: &[],
            else_block: Some(empty_block()),
            span: Span::new(0, 50, 1, 1),
        })];

        let cfg = CfgBuilder::build(&stmts);

        // Should have: ENTRY, EXIT, code_block (with branch), then_block, else_block, join_block
        assert!(cfg.block_count() >= 5);

        // Find the branch
        let mut found_branch = false;
        for block in &cfg.blocks {
            if let Terminator::Branch {
                true_target,
                false_target,
                ..
            } = &block.terminator
            {
                found_branch = true;
                // Both targets should eventually reach the join block
                assert_ne!(true_target, false_target);
            }
        }
        assert!(found_branch, "Should have a Branch terminator");
    }

    #[test]
    fn test_while_loop() {
        let stmts = vec![Statement::While(WhileStatement {
            condition: make_expr_literal_true(),
            body: empty_block(),
            span: Span::new(0, 30, 1, 1),
        })];

        let cfg = CfgBuilder::build(&stmts);

        // Should have loop_headers
        assert!(
            !cfg.loop_headers.is_empty(),
            "While loop should create a loop header"
        );

        // Find the LoopBack terminator (body -> header)
        let mut found_loopback = false;
        for block in &cfg.blocks {
            if matches!(block.terminator, Terminator::LoopBack(_)) {
                found_loopback = true;
            }
        }
        assert!(found_loopback, "Should have a LoopBack terminator");
    }

    #[test]
    fn test_for_numeric() {
        let arena = bumpalo::Bump::new();
        let for_num = arena.alloc(ForNumeric {
            variable: Ident {
                node: StringInterner::new().get_or_intern("i"),
                span: Span::dummy(),
            },
            start: make_expr_literal_true(),
            end: make_expr_literal_true(),
            step: None,
            body: empty_block(),
            span: Span::new(0, 30, 1, 1),
        });
        let stmts = vec![Statement::For(arena.alloc(ForStatement::Numeric(for_num)))];

        let cfg = CfgBuilder::build(&stmts);

        assert!(
            !cfg.loop_headers.is_empty(),
            "For loop should create a loop header"
        );
    }

    #[test]
    fn test_for_generic() {
        let arena = bumpalo::Bump::new();
        let stmts = vec![Statement::For(arena.alloc(ForStatement::Generic(
            ForGeneric {
                variables: &[],
                pattern: None,
                iterators: &[],
                body: empty_block(),
                span: Span::new(0, 30, 1, 1),
            },
        )))];

        let cfg = CfgBuilder::build(&stmts);

        assert!(
            !cfg.loop_headers.is_empty(),
            "Generic for loop should create a loop header"
        );
    }

    #[test]
    fn test_repeat_until() {
        let stmts = vec![Statement::Repeat(RepeatStatement {
            body: empty_block(),
            until: make_expr_literal_true(),
            span: Span::new(0, 30, 1, 1),
        })];

        let cfg = CfgBuilder::build(&stmts);

        // Repeat...until: body is the loop header
        assert!(
            !cfg.loop_headers.is_empty(),
            "Repeat loop should create a loop header"
        );
    }

    #[test]
    fn test_predecessor_successor_consistency() {
        // Build a simple if/else and verify predecessor/successor consistency
        let stmts = vec![Statement::If(IfStatement {
            condition: make_expr_literal_true(),
            then_block: empty_block(),
            else_ifs: &[],
            else_block: Some(empty_block()),
            span: Span::new(0, 50, 1, 1),
        })];

        let cfg = CfgBuilder::build(&stmts);

        // For every successor edge A -> B, B should have A as a predecessor
        for block in &cfg.blocks {
            for succ in cfg.succs(block.id) {
                assert!(
                    cfg.preds(*succ).contains(&block.id),
                    "Block {} is a successor of {} but {} is not a predecessor of {}",
                    succ,
                    block.id,
                    block.id,
                    succ,
                );
            }
        }

        // For every predecessor edge B <- A, A should have B as a successor
        for block in &cfg.blocks {
            for pred in cfg.preds(block.id) {
                assert!(
                    cfg.succs(*pred).contains(&block.id),
                    "Block {} is a predecessor of {} but {} is not a successor of {}",
                    pred,
                    block.id,
                    block.id,
                    pred,
                );
            }
        }
    }

    #[test]
    fn test_reverse_postorder() {
        let stmts = vec![Statement::If(IfStatement {
            condition: make_expr_literal_true(),
            then_block: empty_block(),
            else_ifs: &[],
            else_block: Some(empty_block()),
            span: Span::new(0, 50, 1, 1),
        })];

        let cfg = CfgBuilder::build(&stmts);
        let rpo = cfg.reverse_postorder();

        // ENTRY should be first in reverse postorder
        assert_eq!(rpo[0], BlockId::ENTRY);
        // All reachable blocks should appear
        assert!(!rpo.is_empty());
    }

    #[test]
    fn test_try_catch() {
        use luanext_parser::ast::statement::{CatchClause, CatchPattern, TryStatement};

        let interner = StringInterner::new();
        let arena = bumpalo::Bump::new();
        let catch_clauses = arena.alloc_slice_clone(&[CatchClause {
            pattern: CatchPattern::Untyped {
                variable: make_ident(&interner, "e"),
                span: Span::dummy(),
            },
            body: empty_block(),
            span: Span::new(20, 40, 2, 1),
        }]);

        let stmts = vec![Statement::Try(TryStatement {
            try_block: empty_block(),
            catch_clauses,
            finally_block: None,
            span: Span::new(0, 50, 1, 1),
        })];

        let cfg = CfgBuilder::build(&stmts);

        // Find TryCatch terminator
        let mut found_try_catch = false;
        for block in &cfg.blocks {
            if let Terminator::TryCatch { catch_targets, .. } = &block.terminator {
                found_try_catch = true;
                assert_eq!(catch_targets.len(), 1, "Should have one catch target");
            }
        }
        assert!(found_try_catch, "Should have a TryCatch terminator");
    }

    #[test]
    fn test_stmt_to_block_mapping() {
        let interner = StringInterner::new();
        let stmts = vec![
            make_var_decl(&interner, "a"),
            make_var_decl(&interner, "b"),
            make_var_decl(&interner, "c"),
        ];

        let cfg = CfgBuilder::build(&stmts);

        // All 3 statements should be mapped
        assert!(cfg.stmt_to_block.contains_key(&0));
        assert!(cfg.stmt_to_block.contains_key(&1));
        assert!(cfg.stmt_to_block.contains_key(&2));

        // All should map to the same block (linear code)
        let block_0 = cfg.stmt_to_block[&0];
        assert_eq!(cfg.stmt_to_block[&1], block_0);
        assert_eq!(cfg.stmt_to_block[&2], block_0);
    }

    #[test]
    fn test_nested_loops() {
        // while true do while true do end end
        let inner_while = Statement::While(WhileStatement {
            condition: make_expr_literal_true(),
            body: empty_block(),
            span: Span::new(20, 50, 2, 5),
        });
        let arena = bumpalo::Bump::new();
        let inner_stmts = arena.alloc_slice_clone(&[inner_while]);
        let outer_body = Block {
            statements: inner_stmts,
            span: Span::new(15, 55, 1, 16),
        };

        let stmts = vec![Statement::While(WhileStatement {
            condition: make_expr_literal_true(),
            body: outer_body,
            span: Span::new(0, 60, 1, 1),
        })];

        let cfg = CfgBuilder::build(&stmts);

        // Should have 2 loop headers (outer + inner)
        assert_eq!(
            cfg.loop_headers.len(),
            2,
            "Nested loops should create 2 loop headers"
        );
    }
}
