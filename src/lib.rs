//! # temporal-logic
//!
//! Linear temporal logic (LTL) with formula parsing, tableau construction,
//! and Büchi automaton translation.
//!
//! ## Features
//!
//! - LTL formula representation with all standard operators
//! - Text parsing for LTL formulas
//! - Negation Normal Form (NNF) conversion
//! - Tableau construction for satisfiability checking
//! - Büchi automaton generation for model checking
//!
//! ## Example
//!
//! ```
//! use temporal_logic::Formula;
//!
//! // Parse an LTL formula
//! let f = Formula::parse("G(request -> F response)").unwrap();
//! println!("Parsed: {}", f);
//!
//! // Convert to NNF
//! let nnf = f.to_nnf();
//! println!("NNF: {}", nnf);
//! ```

mod formula;
mod parse;
mod tableau;
mod automaton;
mod check;

pub use formula::{Formula, BinOp, UnOp};
pub use parse::ParseError;
pub use tableau::{Tableau, TableauNode};
pub use automaton::{BuchiAutomaton, BuchiState};
pub use check::LtlChecker;
