//! M2.4 — rule registry: query-driven traversal, shape only (docs/plan.md line 106).
//! Empty by construction in Phase 1; M5 (Oberon-X) is what actually registers rules.

use crate::diagnostic::Diagnostic;
use tree_sitter::Tree;

pub trait Rule {
    fn check(&self, tree: &Tree, text: &str) -> Vec<Diagnostic>;
}

pub struct RuleRegistry {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleRegistry {
    pub fn new() -> Self {
        RuleRegistry { rules: Vec::new() }
    }

    pub fn register(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    pub fn run(&self, tree: &Tree, text: &str) -> Vec<Diagnostic> {
        self.rules.iter().flat_map(|r| r.check(tree, text)).collect()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
