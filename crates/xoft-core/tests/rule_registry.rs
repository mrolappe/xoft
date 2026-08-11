//! M2.4 — rule registry: empty by construction in Phase 1 (docs/plan.md: "query-driven
//! traversal, empty in Phase 1 | shape only; filled in M5"). No real rule exists yet, so
//! these tests only prove the wiring: an empty registry runs zero rules, and a trivial
//! registered rule actually gets invoked and its diagnostics collected.

use xoft_core::diagnostic::Diagnostic;
use xoft_core::rule::{Rule, RuleRegistry};

fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&xoft_core::grammar::language()).unwrap();
    parser.parse(source, None).unwrap()
}

#[test]
fn empty_registry_runs_zero_rules() {
    let tree = parse("MODULE M;\nEND M.\n");
    let registry = RuleRegistry::new();
    assert!(registry.run(&tree, "MODULE M;\nEND M.\n").is_empty());
}

struct AlwaysFlagsRoot;

impl Rule for AlwaysFlagsRoot {
    fn check(&self, tree: &tree_sitter::Tree, _text: &str) -> Vec<Diagnostic> {
        let root = tree.root_node();
        vec![Diagnostic {
            start_byte: root.start_byte(),
            end_byte: root.end_byte(),
            message: "always flagged".to_string(),
        }]
    }
}

#[test]
fn a_registered_rule_is_actually_run() {
    let source = "MODULE M;\nEND M.\n";
    let tree = parse(source);
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(AlwaysFlagsRoot));
    let ds = registry.run(&tree, source);
    assert_eq!(ds.len(), 1);
    assert_eq!(ds[0].message, "always flagged");
}
