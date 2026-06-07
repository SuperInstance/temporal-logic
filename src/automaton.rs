//! Büchi automaton construction from LTL formulas.

use crate::Formula;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Unique identifier for a Büchi automaton state.
pub type BuchiState = u32;

/// A transition label (set of propositions).
pub type Label = HashSet<String>;

/// A generalized Büchi automaton.
#[derive(Clone, Debug)]
pub struct BuchiAutomaton {
    /// States.
    states: HashSet<BuchiState>,
    /// Initial state.
    initial: Option<BuchiState>,
    /// Accepting states.
    accepting: HashSet<BuchiState>,
    /// Transitions: (source, label, target).
    transitions: Vec<(BuchiState, Label, BuchiState)>,
    /// State labels (for debugging).
    state_labels: HashMap<BuchiState, String>,
    /// Next state ID.
    next_id: BuchiState,
}

impl BuchiAutomaton {
    /// Create a new empty Büchi automaton.
    pub fn new() -> Self {
        BuchiAutomaton {
            states: HashSet::new(),
            initial: None,
            accepting: HashSet::new(),
            transitions: Vec::new(),
            state_labels: HashMap::new(),
            next_id: 0,
        }
    }

    /// Create a new state.
    pub fn add_state(&mut self) -> BuchiState {
        let id = self.next_id;
        self.next_id += 1;
        self.states.insert(id);
        id
    }

    /// Create a labeled state.
    pub fn add_labeled_state(&mut self, label: &str) -> BuchiState {
        let id = self.add_state();
        self.state_labels.insert(id, label.to_string());
        id
    }

    /// Set the initial state.
    pub fn set_initial(&mut self, state: BuchiState) {
        self.initial = Some(state);
    }

    /// Mark a state as accepting.
    pub fn set_accepting(&mut self, state: BuchiState) {
        self.accepting.insert(state);
    }

    /// Add a transition.
    pub fn add_transition(&mut self, from: BuchiState, label: Label, to: BuchiState) {
        self.transitions.push((from, label, to));
    }

    /// Get the initial state.
    pub fn initial(&self) -> Option<BuchiState> {
        self.initial
    }

    /// Get all states.
    pub fn states(&self) -> &HashSet<BuchiState> {
        &self.states
    }

    /// Get accepting states.
    pub fn accepting(&self) -> &HashSet<BuchiState> {
        &self.accepting
    }

    /// Get transitions from a state.
    pub fn transitions_from(&self, state: BuchiState) -> Vec<(&Label, BuchiState)> {
        self.transitions
            .iter()
            .filter(|(s, _, _)| *s == state)
            .map(|(_, l, t)| (l, *t))
            .collect()
    }

    /// Get all transitions.
    pub fn transitions(&self) -> &[(BuchiState, Label, BuchiState)] {
        &self.transitions
    }

    /// Number of states.
    pub fn num_states(&self) -> usize {
        self.states.len()
    }

    /// Number of transitions.
    pub fn num_transitions(&self) -> usize {
        self.transitions.len()
    }

    /// Whether a state is accepting.
    pub fn is_accepting(&self, state: BuchiState) -> bool {
        self.accepting.contains(&state)
    }

    /// Construct a Büchi automaton from an LTL formula.
    /// Uses a simple on-the-fly construction.
    pub fn from_ltl(formula: &Formula) -> Self {
        let mut ba = BuchiAutomaton::new();
        let nnf = formula.to_nnf();

        // Create initial and accepting states
        let s0 = ba.add_labeled_state("init");
        let s_acc = ba.add_labeled_state("accept");

        ba.set_initial(s0);
        ba.set_accepting(s_acc);

        // Build transitions based on the formula structure
        build_transitions(&mut ba, s0, s_acc, &nnf);

        ba
    }

    /// Check if a word (sequence of label sets) is accepted.
    /// A word is accepted if there's a run that visits accepting states infinitely often.
    pub fn accepts(&self, word: &[Label]) -> bool {
        if let Some(init) = self.initial {
            self.check_acceptance(init, word, 0, &mut HashSet::new(), 0)
        } else {
            false
        }
    }

    fn check_acceptance(
        &self,
        current: BuchiState,
        word: &[Label],
        pos: usize,
        visited: &mut HashSet<(BuchiState, usize, bool)>,
        accept_count: usize,
    ) -> bool {
        // State: (current_state, position, whether we've seen an accepting state)
        let have_accepted = self.is_accepting(current) || accept_count > 0;
        let key = (current, pos, have_accepted);

        if visited.contains(&key) {
            return false;
        }
        visited.insert(key);

        // If we've gone through the whole word, check if we're in an accepting loop
        if pos >= word.len() {
            return have_accepted;
        }

        let label = &word[pos];
        for (trans_label, next) in self.transitions_from(current) {
            if label_superset(label, trans_label) {
                let new_accept = if self.is_accepting(next) { accept_count + 1 } else { accept_count };
                if self.check_acceptance(next, word, pos + 1, visited, new_accept) {
                    return true;
                }
            }
        }

        false
    }

    /// Remove unreachable states.
    pub fn minimize(&mut self) {
        if let Some(init) = self.initial {
            let mut reachable = HashSet::new();
            let mut stack = vec![init];
            while let Some(s) = stack.pop() {
                if reachable.insert(s) {
                    for (_, next) in self.transitions_from(s) {
                        if !reachable.contains(&next) {
                            stack.push(next);
                        }
                    }
                }
            }
            self.states.retain(|s| reachable.contains(s));
            self.accepting.retain(|s| reachable.contains(s));
            self.transitions.retain(|(s, _, t)| reachable.contains(s) && reachable.contains(t));
        }
    }
}

/// Check if `word_label` is a superset of `trans_label`.
fn label_superset(word_label: &Label, trans_label: &Label) -> bool {
    trans_label.iter().all(|p| word_label.contains(p))
}

/// Build transitions for a formula.
fn build_transitions(ba: &mut BuchiAutomaton, from: BuchiState, accept: BuchiState, formula: &Formula) {
    match formula {
        Formula::True => {
            ba.add_transition(from, HashSet::new(), accept);
        }
        Formula::False => {
            // No transitions
        }
        Formula::Atom(s) => {
            let mut label = HashSet::new();
            label.insert(s.clone());
            ba.add_transition(from, label, accept);
        }
        Formula::Unary(crate::UnOp::Not, f) => {
            if let Formula::Atom(ref s) = **f {
                // ¬p: transition with label that doesn't contain p
                // We represent this as a special label
                let mut label = HashSet::new();
                label.insert(format!("¬{}", s));
                ba.add_transition(from, label, accept);
            }
        }
        Formula::Unary(crate::UnOp::Next, f) => {
            let mid = ba.add_labeled_state("X");
            ba.add_transition(from, HashSet::new(), mid);
            build_transitions(ba, mid, accept, f);
        }
        Formula::Unary(crate::UnOp::Finally, f) => {
            // F p ≡ true U p
            let mid = ba.add_labeled_state("F");
            ba.add_transition(from, HashSet::new(), mid);
            // Either p holds now
            build_transitions(ba, mid, accept, f);
            // Or skip
            ba.add_transition(mid, HashSet::new(), mid);
        }
        Formula::Unary(crate::UnOp::Globally, f) => {
            // G p: loop on states where p holds
            let loop_state = ba.add_labeled_state("G");
            ba.set_accepting(loop_state);
            build_transitions(ba, from, loop_state, f);
            build_transitions(ba, loop_state, loop_state, f);
        }
        Formula::Binary(crate::BinOp::And, l, r) => {
            let mid = ba.add_labeled_state("AND");
            build_transitions(ba, from, mid, l);
            build_transitions(ba, mid, accept, r);
        }
        Formula::Binary(crate::BinOp::Or, l, r) => {
            build_transitions(ba, from, accept, l);
            build_transitions(ba, from, accept, r);
        }
        Formula::Binary(crate::BinOp::Until, l, r) => {
            let mid = ba.add_labeled_state("U");
            ba.add_transition(from, HashSet::new(), mid);
            // Either r holds now
            build_transitions(ba, mid, accept, r);
            // Or l holds and we loop
            let loop_state = ba.add_labeled_state("U-loop");
            build_transitions(ba, mid, loop_state, l);
            ba.add_transition(loop_state, HashSet::new(), mid);
        }
        Formula::Binary(crate::BinOp::Release, l, r) => {
            // p R q: q must hold until and including when p holds
            let mid = ba.add_labeled_state("R");
            ba.add_transition(from, HashSet::new(), mid);
            ba.set_accepting(mid);
            // q holds and either p holds (done) or continue
            build_transitions(ba, mid, accept, l);
            build_transitions(ba, mid, mid, r);
        }
        _ => {}
    }
}

impl fmt::Display for BuchiAutomaton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Büchi Automaton ({} states, {} transitions):", self.num_states(), self.num_transitions())?;
        if let Some(init) = self.initial {
            writeln!(f, "  Initial: q{}", init)?;
        }
        write!(f, "  Accepting: ")?;
        let acc: Vec<String> = self.accepting.iter().map(|s| format!("q{}", s)).collect();
        writeln!(f, "{}", acc.join(", "))?;
        for (from, label, to) in &self.transitions {
            let label_str = if label.is_empty() {
                "ε".to_string()
            } else {
                label.iter().cloned().collect::<Vec<_>>().join("∧")
            };
            writeln!(f, "  q{} --[{}]--> q{}", from, label_str, to)?;
        }
        Ok(())
    }
}

impl Default for BuchiAutomaton {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_automaton() {
        let mut ba = BuchiAutomaton::new();
        let s0 = ba.add_state();
        let s1 = ba.add_state();
        ba.set_initial(s0);
        ba.set_accepting(s1);
        let mut label = HashSet::new();
        label.insert("p".to_string());
        ba.add_transition(s0, label, s1);
        assert_eq!(ba.num_states(), 2);
        assert_eq!(ba.num_transitions(), 1);
    }

    #[test]
    fn test_from_ltl_atom() {
        let f = Formula::atom("p");
        let ba = BuchiAutomaton::from_ltl(&f);
        assert!(ba.num_states() >= 2);
        assert!(ba.initial().is_some());
    }

    #[test]
    fn test_from_ltl_globally() {
        let f = Formula::globally(Formula::atom("p"));
        let ba = BuchiAutomaton::from_ltl(&f);
        assert!(ba.num_states() >= 2);
        assert!(!ba.accepting().is_empty());
    }

    #[test]
    fn test_from_ltl_finally() {
        let f = Formula::finally(Formula::atom("p"));
        let ba = BuchiAutomaton::from_ltl(&f);
        assert!(ba.num_states() >= 2);
    }

    #[test]
    fn test_from_ltl_until() {
        let f = Formula::until(Formula::atom("p"), Formula::atom("q"));
        let ba = BuchiAutomaton::from_ltl(&f);
        assert!(ba.num_states() >= 2);
    }

    #[test]
    fn test_transitions_from() {
        let mut ba = BuchiAutomaton::new();
        let s0 = ba.add_state();
        let s1 = ba.add_state();
        let mut label = HashSet::new();
        label.insert("p".to_string());
        ba.add_transition(s0, label.clone(), s1);
        let trans = ba.transitions_from(s0);
        assert_eq!(trans.len(), 1);
    }

    #[test]
    fn test_minimize() {
        let mut ba = BuchiAutomaton::new();
        let s0 = ba.add_state();
        let s1 = ba.add_state();
        let s2 = ba.add_state(); // unreachable
        ba.set_initial(s0);
        ba.add_transition(s0, HashSet::new(), s1);
        ba.minimize();
        assert_eq!(ba.num_states(), 2);
    }

    #[test]
    fn test_display() {
        let f = Formula::atom("p");
        let ba = BuchiAutomaton::from_ltl(&f);
        let s = format!("{}", ba);
        assert!(s.contains("Büchi"));
    }

    #[test]
    fn test_accepts_simple() {
        let mut ba = BuchiAutomaton::new();
        let s0 = ba.add_state();
        let s1 = ba.add_state();
        ba.set_initial(s0);
        ba.set_accepting(s1);
        let mut label = HashSet::new();
        label.insert("p".to_string());
        ba.add_transition(s0, label, s1);

        let word = vec![{
            let mut l = HashSet::new();
            l.insert("p".to_string());
            l
        }];
        assert!(ba.accepts(&word));
    }

    #[test]
    fn test_from_ltl_or() {
        let f = Formula::or(Formula::atom("p"), Formula::atom("q"));
        let ba = BuchiAutomaton::from_ltl(&f);
        assert!(ba.num_states() >= 2);
    }

    #[test]
    fn test_from_ltl_and() {
        let f = Formula::and(Formula::atom("p"), Formula::atom("q"));
        let ba = BuchiAutomaton::from_ltl(&f);
        assert!(ba.num_states() >= 2);
    }
}
