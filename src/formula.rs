//! LTL formula representation.

use std::fmt;

/// Binary temporal/logical operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinOp {
    And,
    Or,
    Implies,
    Until,
    Release,
}

/// Unary temporal operators.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UnOp {
    Not,
    Next,
    Finally,
    Globally,
}

/// An LTL formula.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Formula {
    /// Boolean constant true.
    True,
    /// Boolean constant false.
    False,
    /// Atomic proposition.
    Atom(String),
    /// Unary operator applied to a subformula.
    Unary(UnOp, Box<Formula>),
    /// Binary operator applied to two subformulas.
    Binary(BinOp, Box<Formula>, Box<Formula>),
}

impl Formula {
    /// Atomic proposition.
    pub fn atom(name: &str) -> Self {
        Formula::Atom(name.to_string())
    }

    /// Negation.
    pub fn not(f: Formula) -> Self {
        Formula::Unary(UnOp::Not, Box::new(f))
    }

    /// Next (X).
    pub fn next(f: Formula) -> Self {
        Formula::Unary(UnOp::Next, Box::new(f))
    }

    /// Eventually (F).
    pub fn finally(f: Formula) -> Self {
        Formula::Unary(UnOp::Finally, Box::new(f))
    }

    /// Always/Globally (G).
    pub fn globally(f: Formula) -> Self {
        Formula::Unary(UnOp::Globally, Box::new(f))
    }

    /// Conjunction.
    pub fn and(left: Formula, right: Formula) -> Self {
        Formula::Binary(BinOp::And, Box::new(left), Box::new(right))
    }

    /// Disjunction.
    pub fn or(left: Formula, right: Formula) -> Self {
        Formula::Binary(BinOp::Or, Box::new(left), Box::new(right))
    }

    /// Implication.
    pub fn implies(left: Formula, right: Formula) -> Self {
        Formula::Binary(BinOp::Implies, Box::new(left), Box::new(right))
    }

    /// Until (U).
    pub fn until(left: Formula, right: Formula) -> Self {
        Formula::Binary(BinOp::Until, Box::new(left), Box::new(right))
    }

    /// Release (R).
    pub fn release(left: Formula, right: Formula) -> Self {
        Formula::Binary(BinOp::Release, Box::new(left), Box::new(right))
    }

    /// Parse a formula from a string.
    pub fn parse(input: &str) -> Result<Formula, crate::ParseError> {
        crate::parse::parse_formula(input)
    }

    /// Convert to negation normal form (NNF).
    /// Pushes negations inward and eliminates implications.
    pub fn to_nnf(&self) -> Formula {
        match self {
            Formula::True | Formula::False | Formula::Atom(_) => self.clone(),
            Formula::Unary(UnOp::Not, f) => f.negate_to_nnf(),
            Formula::Unary(op, f) => Formula::Unary(*op, Box::new(f.to_nnf())),
            Formula::Binary(BinOp::Implies, l, r) => {
                // p -> q  ≡  ¬p ∨ q
                Formula::or(Formula::not(*l.clone()).to_nnf(), r.to_nnf())
            }
            Formula::Binary(BinOp::Until, l, r) => {
                Formula::until(l.to_nnf(), r.to_nnf())
            }
            Formula::Binary(BinOp::Release, l, r) => {
                Formula::release(l.to_nnf(), r.to_nnf())
            }
            Formula::Binary(BinOp::And, l, r) => {
                Formula::and(l.to_nnf(), r.to_nnf())
            }
            Formula::Binary(BinOp::Or, l, r) => {
                Formula::or(l.to_nnf(), r.to_nnf())
            }
        }
    }

    /// Helper: negate a formula and push to NNF.
    fn negate_to_nnf(&self) -> Formula {
        match self {
            Formula::True => Formula::False,
            Formula::False => Formula::True,
            Formula::Atom(s) => Formula::not(Formula::Atom(s.clone())),
            Formula::Unary(UnOp::Not, f) => f.to_nnf(), // double negation
            Formula::Unary(UnOp::Finally, f) => {
                // ¬F p ≡ G ¬p
                Formula::globally(f.negate_to_nnf())
            }
            Formula::Unary(UnOp::Globally, f) => {
                // ¬G p ≡ F ¬p
                Formula::finally(f.negate_to_nnf())
            }
            Formula::Unary(UnOp::Next, f) => {
                // ¬X p ≡ X ¬p
                Formula::next(f.negate_to_nnf())
            }
            Formula::Binary(BinOp::And, l, r) => {
                // De Morgan
                Formula::or(l.negate_to_nnf(), r.negate_to_nnf())
            }
            Formula::Binary(BinOp::Or, l, r) => {
                Formula::and(l.negate_to_nnf(), r.negate_to_nnf())
            }
            Formula::Binary(BinOp::Implies, l, r) => {
                // ¬(p → q) ≡ p ∧ ¬q
                Formula::and(l.to_nnf(), r.negate_to_nnf())
            }
            Formula::Binary(BinOp::Until, l, r) => {
                // ¬(p U q) ≡ (¬q) R (¬p)
                Formula::release(r.negate_to_nnf(), l.negate_to_nnf())
            }
            Formula::Binary(BinOp::Release, l, r) => {
                // ¬(p R q) ≡ (¬q) U (¬p)
                Formula::until(r.negate_to_nnf(), l.negate_to_nnf())
            }
        }
    }

    /// Collect all atomic propositions in the formula.
    pub fn atoms(&self) -> Vec<String> {
        let mut atoms = Vec::new();
        self.collect_atoms(&mut atoms);
        atoms.sort();
        atoms.dedup();
        atoms
    }

    fn collect_atoms(&self, atoms: &mut Vec<String>) {
        match self {
            Formula::Atom(s) => {
                if !atoms.contains(s) {
                    atoms.push(s.clone());
                }
            }
            Formula::True | Formula::False => {}
            Formula::Unary(_, f) => f.collect_atoms(atoms),
            Formula::Binary(_, l, r) => {
                l.collect_atoms(atoms);
                r.collect_atoms(atoms);
            }
        }
    }

    /// Whether the formula is in NNF (no implications, negations only on atoms).
    pub fn is_nnf(&self) -> bool {
        match self {
            Formula::True | Formula::False | Formula::Atom(_) => true,
            Formula::Unary(UnOp::Not, f) => matches!(**f, Formula::Atom(_)),
            Formula::Unary(_, f) => f.is_nnf(),
            Formula::Binary(BinOp::Implies, _, _) => false,
            Formula::Binary(_, l, r) => l.is_nnf() && r.is_nnf(),
        }
    }

    /// Substitute an atom with another formula.
    pub fn substitute(&self, atom: &str, replacement: &Formula) -> Formula {
        match self {
            Formula::Atom(s) if s == atom => replacement.clone(),
            Formula::True | Formula::False | Formula::Atom(_) => self.clone(),
            Formula::Unary(op, f) => Formula::Unary(*op, Box::new(f.substitute(atom, replacement))),
            Formula::Binary(op, l, r) => Formula::Binary(
                *op,
                Box::new(l.substitute(atom, replacement)),
                Box::new(r.substitute(atom, replacement)),
            ),
        }
    }

    /// Get the subformulas.
    pub fn subformulas(&self) -> Vec<Formula> {
        match self {
            Formula::True | Formula::False | Formula::Atom(_) => vec![self.clone()],
            Formula::Unary(_, f) => {
                let mut subs = vec![self.clone()];
                subs.extend(f.subformulas());
                subs
            }
            Formula::Binary(_, l, r) => {
                let mut subs = vec![self.clone()];
                subs.extend(l.subformulas());
                subs.extend(r.subformulas());
                subs
            }
        }
    }
}

impl fmt::Display for Formula {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Formula::True => write!(f, "true"),
            Formula::False => write!(f, "false"),
            Formula::Atom(s) => write!(f, "{}", s),
            Formula::Unary(UnOp::Not, inner) => write!(f, "¬{}", inner),
            Formula::Unary(UnOp::Next, inner) => write!(f, "X({})", inner),
            Formula::Unary(UnOp::Finally, inner) => write!(f, "F({})", inner),
            Formula::Unary(UnOp::Globally, inner) => write!(f, "G({})", inner),
            Formula::Binary(BinOp::And, l, r) => write!(f, "({} ∧ {})", l, r),
            Formula::Binary(BinOp::Or, l, r) => write!(f, "({} ∨ {})", l, r),
            Formula::Binary(BinOp::Implies, l, r) => write!(f, "({} → {})", l, r),
            Formula::Binary(BinOp::Until, l, r) => write!(f, "({} U {})", l, r),
            Formula::Binary(BinOp::Release, l, r) => write!(f, "({} R {})", l, r),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom() {
        let f = Formula::atom("p");
        assert_eq!(format!("{}", f), "p");
        assert_eq!(f.atoms(), vec!["p"]);
    }

    #[test]
    fn test_boolean_constants() {
        assert_eq!(format!("{}", Formula::True), "true");
        assert_eq!(format!("{}", Formula::False), "false");
    }

    #[test]
    fn test_unary_operators() {
        let p = Formula::atom("p");
        assert_eq!(format!("{}", Formula::not(p.clone())), "¬p");
        assert_eq!(format!("{}", Formula::next(p.clone())), "X(p)");
        assert_eq!(format!("{}", Formula::finally(p.clone())), "F(p)");
        assert_eq!(format!("{}", Formula::globally(p)), "G(p)");
    }

    #[test]
    fn test_binary_operators() {
        let p = Formula::atom("p");
        let q = Formula::atom("q");
        assert_eq!(format!("{}", Formula::and(p.clone(), q.clone())), "(p ∧ q)");
        assert_eq!(format!("{}", Formula::or(p.clone(), q.clone())), "(p ∨ q)");
        assert_eq!(format!("{}", Formula::implies(p.clone(), q.clone())), "(p → q)");
        assert_eq!(format!("{}", Formula::until(p.clone(), q.clone())), "(p U q)");
        assert_eq!(format!("{}", Formula::release(p, q)), "(p R q)");
    }

    #[test]
    fn test_nnf_simple() {
        let f = Formula::not(Formula::atom("p"));
        let nnf = f.to_nnf();
        assert!(nnf.is_nnf());
    }

    #[test]
    fn test_nnf_double_negation() {
        let f = Formula::not(Formula::not(Formula::atom("p")));
        let nnf = f.to_nnf();
        assert_eq!(nnf, Formula::atom("p"));
    }

    #[test]
    fn test_nnf_implication() {
        let f = Formula::implies(Formula::atom("p"), Formula::atom("q"));
        let nnf = f.to_nnf();
        assert!(nnf.is_nnf());
        // p → q ≡ ¬p ∨ q
        assert_eq!(nnf, Formula::or(Formula::not(Formula::atom("p")), Formula::atom("q")));
    }

    #[test]
    fn test_nnf_negated_globally() {
        let f = Formula::not(Formula::globally(Formula::atom("p")));
        let nnf = f.to_nnf();
        assert_eq!(nnf, Formula::finally(Formula::not(Formula::atom("p"))));
    }

    #[test]
    fn test_nnf_negated_finally() {
        let f = Formula::not(Formula::finally(Formula::atom("p")));
        let nnf = f.to_nnf();
        assert_eq!(nnf, Formula::globally(Formula::not(Formula::atom("p"))));
    }

    #[test]
    fn test_collect_atoms() {
        let f = Formula::and(
            Formula::atom("p"),
            Formula::or(Formula::atom("q"), Formula::atom("p")),
        );
        let atoms = f.atoms();
        assert_eq!(atoms, vec!["p", "q"]);
    }

    #[test]
    fn test_substitute() {
        let f = Formula::and(Formula::atom("p"), Formula::atom("q"));
        let sub = f.substitute("p", &Formula::atom("r"));
        assert_eq!(sub, Formula::and(Formula::atom("r"), Formula::atom("q")));
    }

    #[test]
    fn test_subformulas() {
        let f = Formula::and(Formula::atom("p"), Formula::atom("q"));
        let subs = f.subformulas();
        assert_eq!(subs.len(), 3);
    }
}
