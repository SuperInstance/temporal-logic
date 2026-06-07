//! Tableau construction for LTL satisfiability checking.

use crate::Formula;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Unique identifier for a tableau node.
pub type NodeRef = u32;

/// A node in the tableau.
#[derive(Clone, Debug)]
pub struct TableauNode {
    /// Unique identifier.
    pub id: NodeRef,
    /// Formulas that must hold at this node.
    pub formulas: HashSet<String>,
    /// Parsed formula representations.
    pub sub_formulas: Vec<Formula>,
    /// Whether this is a leaf node.
    pub is_leaf: bool,
    /// Whether this node is marked (closed).
    pub marked: bool,
    /// Children of this node.
    pub children: Vec<NodeRef>,
}

impl fmt::Display for TableauNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "n{}:{{{}}}{}", self.id,
            self.formulas.iter().cloned().collect::<Vec<_>>().join(", "),
            if self.marked { " ✗" } else if self.is_leaf { " ✓" } else { "" }
        )
    }
}

/// An LTL tableau for satisfiability checking.
#[derive(Clone, Debug)]
pub struct Tableau {
    /// Nodes indexed by ID.
    nodes: HashMap<NodeRef, TableauNode>,
    /// Next available node ID.
    next_id: NodeRef,
    /// Root node reference.
    root: Option<NodeRef>,
}

impl Tableau {
    /// Create a new empty tableau.
    pub fn new() -> Self {
        Tableau {
            nodes: HashMap::new(),
            next_id: 0,
            root: None,
        }
    }

    /// Build a tableau for the given formula.
    pub fn build(formula: &Formula) -> Self {
        let mut tableau = Tableau::new();
        let root = tableau.create_node();
        tableau.root = Some(root);

        // Add the formula and its expansion
        let expanded = tableau.expand_formula(formula);
        for f in &expanded {
            if let Some(node) = tableau.nodes.get_mut(&root) {
                node.formulas.insert(format!("{}", f));
                node.sub_formulas.push(f.clone());
            }
        }

        // Expand the tableau
        tableau.expand_node(root);
        tableau
    }

    /// Create a new node.
    fn create_node(&mut self) -> NodeRef {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, TableauNode {
            id,
            formulas: HashSet::new(),
            sub_formulas: Vec::new(),
            is_leaf: false,
            marked: false,
            children: Vec::new(),
        });
        id
    }

    /// Expand a formula into a set of conjunctive subformulas.
    fn expand_formula(&self, formula: &Formula) -> Vec<Formula> {
        match formula {
            Formula::Binary(crate::BinOp::And, l, r) => {
                let mut result = self.expand_formula(l);
                result.extend(self.expand_formula(r));
                result
            }
            _ => vec![formula.clone()],
        }
    }

    /// Expand a node in the tableau.
    fn expand_node(&mut self, node_id: NodeRef) {
        let formulas: Vec<Formula> = self.nodes.get(&node_id)
            .map(|n| n.sub_formulas.clone())
            .unwrap_or_default();

        for formula in formulas {
            match &formula {
                Formula::Binary(crate::BinOp::Or, l, r) => {
                    // Beta rule: branch into two children
                    let left_id = self.create_node();
                    let right_id = self.create_node();

                    if let Some(left_node) = self.nodes.get_mut(&left_id) {
                        left_node.formulas.insert(format!("{}", l));
                        left_node.sub_formulas.push(*l.clone());
                    }
                    if let Some(right_node) = self.nodes.get_mut(&right_id) {
                        right_node.formulas.insert(format!("{}", r));
                        right_node.sub_formulas.push(*r.clone());
                    }

                    if let Some(node) = self.nodes.get_mut(&node_id) {
                        node.children.push(left_id);
                        node.children.push(right_id);
                    }
                }
                Formula::Binary(crate::BinOp::Until, l, r) => {
                    // p U q ≡ q ∨ (p ∧ X(p U q))
                    // Beta: either q holds now, or p holds and X(p U q)
                    let left_id = self.create_node();
                    let right_id = self.create_node();

                    if let Some(ln) = self.nodes.get_mut(&left_id) {
                        ln.formulas.insert(format!("{}", r));
                        ln.sub_formulas.push(*r.clone());
                    }
                    if let Some(rn) = self.nodes.get_mut(&right_id) {
                        rn.formulas.insert(format!("{}", l));
                        rn.sub_formulas.push(*l.clone());
                        rn.formulas.insert(format!("X({})", formula));
                        rn.sub_formulas.push(Formula::next(formula.clone()));
                    }

                    if let Some(node) = self.nodes.get_mut(&node_id) {
                        node.children.push(left_id);
                        node.children.push(right_id);
                    }
                }
                _ => {}
            }
        }

        // Recursively expand children
        let children: Vec<NodeRef> = self.nodes.get(&node_id)
            .map(|n| n.children.clone())
            .unwrap_or_default();

        for child in children {
            self.expand_node(child);
        }

        // Check if leaf
        let children_empty = self.nodes.get(&node_id).map(|n| n.children.is_empty()).unwrap_or(false);
        if children_empty {
            let is_contra = self.nodes.get(&node_id)
                .map(|n| self.has_contradiction(&n.formulas))
                .unwrap_or(false);
            if let Some(node) = self.nodes.get_mut(&node_id) {
                node.is_leaf = true;
                node.marked = is_contra;
            }
        }
    }

    /// Check if a set of formulas contains a contradiction.
    fn has_contradiction(&self, formulas: &HashSet<String>) -> bool {
        // Check for p and ¬p simultaneously
        for f in formulas {
            if f.starts_with('¬') || f.starts_with('!') {
                let atom = f.trim_start_matches('¬').trim_start_matches('!');
                if formulas.contains(atom) {
                    return true;
                }
            }
        }
        // Check for false
        if formulas.contains("false") {
            return true;
        }
        false
    }

    /// Check if the formula is satisfiable.
    pub fn is_satisfiable(&self) -> bool {
        match self.root {
            Some(root) => self.node_satisfiable(root),
            None => true,
        }
    }

    fn node_satisfiable(&self, node_id: NodeRef) -> bool {
        let node = match self.nodes.get(&node_id) {
            Some(n) => n,
            None => return false,
        };

        if node.marked {
            return false;
        }

        if node.is_leaf {
            return true;
        }

        // At least one child must be satisfiable
        node.children.iter().any(|&child| self.node_satisfiable(child))
    }

    /// Get the root node.
    pub fn root(&self) -> Option<&TableauNode> {
        self.root.and_then(|id| self.nodes.get(&id))
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: NodeRef) -> Option<&TableauNode> {
        self.nodes.get(&id)
    }

    /// Number of nodes.
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    /// Get all node IDs.
    pub fn node_ids(&self) -> Vec<NodeRef> {
        let mut ids: Vec<NodeRef> = self.nodes.keys().copied().collect();
        ids.sort();
        ids
    }
}

impl Default for Tableau {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tableau_atom() {
        let f = Formula::atom("p");
        let t = Tableau::build(&f);
        assert!(t.is_satisfiable());
    }

    #[test]
    fn test_tableau_conjunction() {
        let f = Formula::and(Formula::atom("p"), Formula::atom("q"));
        let t = Tableau::build(&f);
        assert!(t.is_satisfiable());
    }

    #[test]
    fn test_tableau_disjunction() {
        let f = Formula::or(Formula::atom("p"), Formula::atom("q"));
        let t = Tableau::build(&f);
        assert!(t.is_satisfiable());
    }

    #[test]
    fn test_tableau_contradiction() {
        let f = Formula::and(Formula::atom("p"), Formula::not(Formula::atom("p")));
        let t = Tableau::build(&f);
        // p AND ¬p is unsatisfiable
        assert!(!t.is_satisfiable());
    }

    #[test]
    fn test_tableau_eventually() {
        let f = Formula::finally(Formula::atom("p"));
        let t = Tableau::build(&f);
        assert!(t.is_satisfiable());
    }

    #[test]
    fn test_tableau_globally() {
        let f = Formula::globally(Formula::atom("p"));
        let t = Tableau::build(&f);
        // G(p) is satisfiable (always p)
        assert!(t.is_satisfiable());
    }

    #[test]
    fn test_tableau_num_nodes() {
        let f = Formula::or(Formula::atom("p"), Formula::atom("q"));
        let t = Tableau::build(&f);
        assert!(t.num_nodes() >= 3); // root + 2 children
    }

    #[test]
    fn test_tableau_root() {
        let f = Formula::atom("p");
        let t = Tableau::build(&f);
        assert!(t.root().is_some());
    }

    #[test]
    fn test_tableau_until() {
        let f = Formula::until(Formula::atom("p"), Formula::atom("q"));
        let t = Tableau::build(&f);
        assert!(t.is_satisfiable());
    }

    #[test]
    fn test_empty_tableau() {
        let t = Tableau::new();
        assert!(t.is_satisfiable());
        assert_eq!(t.num_nodes(), 0);
    }
}
