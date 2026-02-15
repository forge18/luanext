//! Dominator tree computation from a Control Flow Graph.
//!
//! Required for: advanced loop optimizations, SSA construction (phi-function placement).
//!
//! Uses the Cooper-Harvey-Kennedy iterative algorithm, which is simple and efficient
//! for the moderate-size CFGs typical of single functions.

use super::cfg::{BlockId, ControlFlowGraph};
use rustc_hash::FxHashMap;

/// Dominator tree computed from a CFG.
///
/// Block A **dominates** block B if every path from the entry block to B must
/// pass through A. The **immediate dominator** (idom) of B is the closest
/// strict dominator.
///
/// The **dominance frontier** of A is the set of blocks B where A dominates
/// a predecessor of B but does not strictly dominate B itself. This is needed
/// for SSA phi-function placement.
#[derive(Debug)]
pub struct DominatorTree {
    /// Immediate dominator for each block. Entry block maps to itself.
    pub idom: FxHashMap<BlockId, BlockId>,
    /// Children in the dominator tree: block -> blocks it immediately dominates.
    pub children: FxHashMap<BlockId, Vec<BlockId>>,
    /// Dominance frontier: block -> set of blocks at its dominance frontier.
    pub frontiers: FxHashMap<BlockId, Vec<BlockId>>,
}

impl DominatorTree {
    /// Compute the dominator tree from a CFG using the Cooper-Harvey-Kennedy algorithm.
    pub fn build(cfg: &ControlFlowGraph) -> Self {
        let rpo = cfg.reverse_postorder();
        if rpo.is_empty() {
            return DominatorTree {
                idom: FxHashMap::default(),
                children: FxHashMap::default(),
                frontiers: FxHashMap::default(),
            };
        }

        // Map BlockId -> reverse postorder index for intersection
        let mut rpo_index: FxHashMap<BlockId, usize> = FxHashMap::default();
        for (i, &block) in rpo.iter().enumerate() {
            rpo_index.insert(block, i);
        }

        // Initialize: idom[entry] = entry, all others undefined
        let mut idom: FxHashMap<BlockId, BlockId> = FxHashMap::default();
        idom.insert(BlockId::ENTRY, BlockId::ENTRY);

        // Iterate until stable
        let mut changed = true;
        while changed {
            changed = false;
            // Skip ENTRY (first in RPO)
            for &block in rpo.iter().skip(1) {
                // Find the first processed predecessor
                let preds = cfg.preds(block);
                let mut new_idom: Option<BlockId> = None;

                for &pred in preds {
                    if idom.contains_key(&pred) {
                        new_idom = Some(match new_idom {
                            None => pred,
                            Some(current) => Self::intersect(current, pred, &idom, &rpo_index),
                        });
                    }
                }

                if let Some(new_idom) = new_idom {
                    if idom.get(&block) != Some(&new_idom) {
                        idom.insert(block, new_idom);
                        changed = true;
                    }
                }
            }
        }

        // Build children map from idom
        let mut children: FxHashMap<BlockId, Vec<BlockId>> = FxHashMap::default();
        for block in &rpo {
            children.insert(*block, Vec::new());
        }
        for (&block, &dom) in &idom {
            if block != dom {
                children.entry(dom).or_default().push(block);
            }
        }

        // Compute dominance frontiers
        let frontiers = Self::compute_frontiers(cfg, &idom);

        DominatorTree {
            idom,
            children,
            frontiers,
        }
    }

    /// Returns true if block `a` dominates block `b`.
    pub fn dominates(&self, a: BlockId, b: BlockId) -> bool {
        if a == b {
            return true;
        }
        let mut current = b;
        loop {
            match self.idom.get(&current) {
                Some(&dom) if dom == current => return false, // Reached entry without finding a
                Some(&dom) if dom == a => return true,
                Some(&dom) => current = dom,
                None => return false,
            }
        }
    }

    /// Returns the immediate dominator of a block, if it has one.
    pub fn immediate_dominator(&self, block: BlockId) -> Option<BlockId> {
        self.idom.get(&block).copied().and_then(|dom| {
            if dom == block {
                None // Entry block has no real idom
            } else {
                Some(dom)
            }
        })
    }

    /// Returns the dominance frontier of a block.
    pub fn frontier(&self, block: BlockId) -> &[BlockId] {
        self.frontiers
            .get(&block)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Intersect two blocks in the dominator tree to find their nearest common dominator.
    fn intersect(
        mut b1: BlockId,
        mut b2: BlockId,
        idom: &FxHashMap<BlockId, BlockId>,
        rpo_index: &FxHashMap<BlockId, usize>,
    ) -> BlockId {
        while b1 != b2 {
            let idx1 = rpo_index.get(&b1).copied().unwrap_or(usize::MAX);
            let idx2 = rpo_index.get(&b2).copied().unwrap_or(usize::MAX);
            if idx1 > idx2 {
                b1 = match idom.get(&b1) {
                    Some(&dom) => dom,
                    None => return b2,
                };
            } else {
                b2 = match idom.get(&b2) {
                    Some(&dom) => dom,
                    None => return b1,
                };
            }
        }
        b1
    }

    /// Compute dominance frontiers using the standard algorithm.
    ///
    /// For each block b with 2+ predecessors: walk from each predecessor up
    /// through the dominator tree until reaching idom(b). Each block visited
    /// (except idom(b)) has b in its dominance frontier.
    fn compute_frontiers(
        cfg: &ControlFlowGraph,
        idom: &FxHashMap<BlockId, BlockId>,
    ) -> FxHashMap<BlockId, Vec<BlockId>> {
        let mut frontiers: FxHashMap<BlockId, Vec<BlockId>> = FxHashMap::default();

        for block in &cfg.blocks {
            let preds = cfg.preds(block.id);
            if preds.len() >= 2 {
                let block_idom = idom.get(&block.id).copied();
                for &pred in preds {
                    let mut runner = pred;
                    while Some(runner) != block_idom {
                        let frontier = frontiers.entry(runner).or_default();
                        if !frontier.contains(&block.id) {
                            frontier.push(block.id);
                        }
                        match idom.get(&runner) {
                            Some(&dom) if dom != runner => runner = dom,
                            _ => break,
                        }
                    }
                }
            }
        }

        frontiers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::analysis::cfg::{CfgBuilder, Terminator};
    use luanext_parser::ast::expression::{Expression, ExpressionKind, Literal};
    use luanext_parser::ast::statement::{Block, IfStatement, Statement, WhileStatement};
    use luanext_parser::span::Span;

    fn make_expr_true() -> Expression<'static> {
        Expression {
            kind: ExpressionKind::Literal(Literal::Boolean(true)),
            span: Span::dummy(),
            annotated_type: None,
            receiver_class: None,
        }
    }

    fn empty_block() -> Block<'static> {
        Block {
            statements: &[],
            span: Span::dummy(),
        }
    }

    #[test]
    fn test_linear_cfg_domination() {
        // Linear: ENTRY -> A -> EXIT
        // ENTRY dominates everything
        let stmts: Vec<Statement<'_>> = vec![];
        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);

        // ENTRY dominates all blocks
        for block in &cfg.blocks {
            if block.id != BlockId::ENTRY {
                assert!(
                    dom_tree.dominates(BlockId::ENTRY, block.id),
                    "ENTRY should dominate {}",
                    block.id
                );
            }
        }
    }

    #[test]
    fn test_diamond_dominance() {
        // if/else diamond: ENTRY -> code -> {then, else} -> join -> EXIT
        let stmts = vec![Statement::If(IfStatement {
            condition: make_expr_true(),
            then_block: empty_block(),
            else_ifs: &[],
            else_block: Some(empty_block()),
            span: Span::new(0, 50, 1, 1),
        })];

        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);

        // Find the branch block (has Branch terminator)
        let branch_block = cfg
            .blocks
            .iter()
            .find(|b| matches!(b.terminator, Terminator::Branch { .. }))
            .unwrap();

        // Branch block should dominate its targets
        if let Terminator::Branch {
            true_target,
            false_target,
            ..
        } = &branch_block.terminator
        {
            assert!(dom_tree.dominates(branch_block.id, *true_target));
            assert!(dom_tree.dominates(branch_block.id, *false_target));
        }
    }

    #[test]
    fn test_loop_header_dominance() {
        // while loop: ENTRY -> code -> header -> {body, exit}
        // header dominates body
        let stmts = vec![Statement::While(WhileStatement {
            condition: make_expr_true(),
            body: empty_block(),
            span: Span::new(0, 30, 1, 1),
        })];

        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);

        // Find the loop header (target of LoopBack)
        for &header_id in &cfg.loop_headers {
            // Header should dominate its body (true_target of Branch)
            if let Some(header) = cfg.block(header_id) {
                if let Terminator::Branch { true_target, .. } = &header.terminator {
                    assert!(
                        dom_tree.dominates(header_id, *true_target),
                        "Loop header should dominate its body"
                    );
                }
            }
        }
    }

    #[test]
    fn test_dominance_frontier_diamond() {
        // In a diamond: the join block should be in the dominance frontier
        // of both the then and else blocks.
        let stmts = vec![Statement::If(IfStatement {
            condition: make_expr_true(),
            then_block: empty_block(),
            else_ifs: &[],
            else_block: Some(empty_block()),
            span: Span::new(0, 50, 1, 1),
        })];

        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);

        // Find the then and else blocks
        if let Some(branch) = cfg
            .blocks
            .iter()
            .find(|b| matches!(b.terminator, Terminator::Branch { .. }))
        {
            if let Terminator::Branch {
                true_target,
                false_target,
                ..
            } = &branch.terminator
            {
                // Both branches should have a non-empty dominance frontier (the join block)
                let then_frontier = dom_tree.frontier(*true_target);
                let else_frontier = dom_tree.frontier(*false_target);

                // At least one of them should have a frontier (the join block)
                assert!(
                    !then_frontier.is_empty() || !else_frontier.is_empty(),
                    "Diamond pattern should produce non-empty dominance frontiers"
                );
            }
        }
    }

    #[test]
    fn test_immediate_dominator() {
        let stmts: Vec<Statement<'_>> = vec![];
        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);

        // ENTRY has no immediate dominator
        assert_eq!(dom_tree.immediate_dominator(BlockId::ENTRY), None);

        // All other blocks should have some idom
        for block in &cfg.blocks {
            if block.id != BlockId::ENTRY {
                // Only reachable blocks have idom
                if dom_tree.idom.contains_key(&block.id) {
                    let idom = dom_tree.immediate_dominator(block.id);
                    // idom should either be None (for entry) or Some(parent)
                    if block.id != BlockId::ENTRY {
                        assert!(
                            idom.is_some(),
                            "Non-entry block {} should have an immediate dominator",
                            block.id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_entry_dominates_all_reachable() {
        // More complex CFG: if/else with while loop
        let stmts = vec![
            Statement::If(IfStatement {
                condition: make_expr_true(),
                then_block: empty_block(),
                else_ifs: &[],
                else_block: Some(empty_block()),
                span: Span::new(0, 50, 1, 1),
            }),
            Statement::While(WhileStatement {
                condition: make_expr_true(),
                body: empty_block(),
                span: Span::new(50, 80, 3, 1),
            }),
        ];

        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);

        // ENTRY dominates all reachable blocks
        let rpo = cfg.reverse_postorder();
        for &block in &rpo {
            assert!(
                dom_tree.dominates(BlockId::ENTRY, block),
                "ENTRY should dominate {} (reachable via RPO)",
                block
            );
        }
    }

    #[test]
    fn test_dominance_frontier_loop() {
        // While loop: the header is in the dominance frontier of the body
        let stmts = vec![Statement::While(WhileStatement {
            condition: make_expr_true(),
            body: empty_block(),
            span: Span::new(0, 30, 1, 1),
        })];

        let cfg = CfgBuilder::build(&stmts);
        let dom_tree = DominatorTree::build(&cfg);

        // Loop header should have a non-empty dominance frontier
        // (because both the pre-header and the body reach the header)
        for &header_id in &cfg.loop_headers {
            if let Some(header) = cfg.block(header_id) {
                if let Terminator::Branch { true_target, .. } = &header.terminator {
                    let body_frontier = dom_tree.frontier(*true_target);
                    // Body's dominance frontier should include the header
                    assert!(
                        body_frontier.contains(&header_id),
                        "Loop body's dominance frontier should include the header"
                    );
                }
            }
        }
    }
}
