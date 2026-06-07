# temporal-logic

Linear temporal logic (LTL) with formula parsing, tableau construction, and Büchi automaton translation.

## Features

- **Formula representation**: All LTL operators (G, F, X, U, R, ∧, ∨, →, ¬)
- **Text parsing**: Parse LTL formulas from strings with flexible syntax
- **Negation Normal Form**: Automatic NNF conversion for tableau and automata
- **Tableau construction**: Build tableaux for satisfiability checking
- **Büchi automata**: Translate LTL formulas to Büchi automata with minimization
- **Trace checking**: Verify LTL formulas against execution traces

## Installation

```toml
[dependencies]
temporal-logic = "0.1.0"
```

## Usage

```rust
use temporal_logic::Formula;

// Parse an LTL formula
let f = Formula::parse("G(request -> F response)").unwrap();

// Convert to NNF
let nnf = f.to_nnf();
println!("NNF: {}", nnf);

// Collect atomic propositions
println!("Atoms: {:?}", f.atoms());
```

## Syntax

| Input | Meaning |
|-------|---------|
| `G(p)` | Globally / Always |
| `F(p)` | Finally / Eventually |
| `X(p)` | Next |
| `p U q` | Until |
| `p R q` | Release |
| `p -> q` | Implies |
| `p AND q` | Conjunction |
| `p OR q` | Disjunction |
| `!p` | Negation |

## Architecture

| Module | Description |
|--------|-------------|
| `formula` | LTL formula representation with NNF conversion |
| `parse` | Recursive descent parser for LTL formulas |
| `tableau` | Tableau construction for satisfiability |
| `automaton` | Büchi automaton generation from LTL |
| `check` | Trace-based LTL model checking |

## License

MIT OR Apache-2.0
