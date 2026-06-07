//! LTL formula parser.

use crate::formula::Formula;

/// Parse error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.message)
    }
}

impl std::error::Error for ParseError {}

/// Token types for the lexer.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    True,
    False,
    Atom(String),
    Not,
    And,
    Or,
    Implies,
    Until,
    Release,
    Next,
    Finally,
    Globally,
    LParen,
    RParen,
}

struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer { input, pos: 0 }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return None;
        }

        let remaining = &self.input[self.pos..];

        // Single character tokens
        match remaining.as_bytes()[0] {
            b'(' => { self.pos += 1; return Some(Token::LParen); }
            b')' => { self.pos += 1; return Some(Token::RParen); }
            b'!' | b'~' => { self.pos += 1; return Some(Token::Not); }
            b'&' => {
                if remaining.starts_with("&&") || remaining.starts_with("&") {
                    let advance = if remaining.starts_with("&&") { 2 } else { 1 };
                    self.pos += advance;
                    return Some(Token::And);
                }
            }
            b'|' => {
                if remaining.starts_with("||") || remaining.starts_with("|") {
                    let advance = if remaining.starts_with("||") { 2 } else { 1 };
                    self.pos += advance;
                    return Some(Token::Or);
                }
            }
            b'-' if remaining.starts_with("->") => {
                self.pos += 2;
                return Some(Token::Implies);
            }
            _ => {}
        }

        // Keywords and identifiers
        let end = remaining.find(|c: char| c.is_whitespace() || c == '(' || c == ')' || c == '&' || c == '|' || c == '-' || c == '!' || c == '~')
            .unwrap_or(remaining.len());
        let word = &remaining[..end];

        let token = match word {
            "true" | "TRUE" | "True" | "1" => Token::True,
            "false" | "FALSE" | "False" | "0" => Token::False,
            "!" | "NOT" | "not" | "neg" => Token::Not,
            "&&" | "AND" | "and" => Token::And,
            "||" | "OR" | "or" => Token::Or,
            "->" | "IMPLIES" | "implies" | "=>" => Token::Implies,
            "U" | "UNTIL" | "until" => Token::Until,
            "R" | "RELEASE" | "release" => Token::Release,
            "X" | "NEXT" | "next" => Token::Next,
            "F" | "FINALLY" | "finally" | "<>" => Token::Finally,
            "G" | "GLOBALLY" | "globally" | "[]" => Token::Globally,
            s => Token::Atom(s.to_string()),
        };

        self.pos += end;
        Some(token)
    }

    fn peek_token(&mut self) -> Option<Token> {
        let saved_pos = self.pos;
        let tok = self.next_token();
        self.pos = saved_pos;
        tok
    }
}

/// Parse an LTL formula from a string.
pub fn parse_formula(input: &str) -> Result<Formula, ParseError> {
    let mut lexer = Lexer::new(input);
    let formula = parse_implies(&mut lexer)?;

    // Check for trailing tokens
    lexer.skip_whitespace();
    if lexer.pos < input.len() {
        return Err(ParseError {
            message: format!("Unexpected trailing input at position {}", lexer.pos),
        });
    }

    Ok(formula)
}

fn parse_implies(lexer: &mut Lexer) -> Result<Formula, ParseError> {
    let left = parse_until(lexer)?;
    let token = lexer.peek_token();
    match token {
        Some(Token::Implies) => {
            lexer.next_token();
            let right = parse_implies(lexer)?;
            Ok(Formula::implies(left, right))
        }
        _ => Ok(left),
    }
}

fn parse_until(lexer: &mut Lexer) -> Result<Formula, ParseError> {
    let left = parse_or(lexer)?;
    let token = lexer.peek_token();
    match token {
        Some(Token::Until) => {
            lexer.next_token();
            let right = parse_until(lexer)?;
            Ok(Formula::until(left, right))
        }
        Some(Token::Release) => {
            lexer.next_token();
            let right = parse_until(lexer)?;
            Ok(Formula::release(left, right))
        }
        _ => Ok(left),
    }
}

fn parse_or(lexer: &mut Lexer) -> Result<Formula, ParseError> {
    let mut left = parse_and(lexer)?;
    loop {
        let token = lexer.peek_token();
        match token {
            Some(Token::Or) => {
                lexer.next_token();
                let right = parse_and(lexer)?;
                left = Formula::or(left, right);
            }
            _ => break,
        }
    }
    Ok(left)
}

fn parse_and(lexer: &mut Lexer) -> Result<Formula, ParseError> {
    let mut left = parse_unary(lexer)?;
    loop {
        let token = lexer.peek_token();
        match token {
            Some(Token::And) => {
                lexer.next_token();
                let right = parse_unary(lexer)?;
                left = Formula::and(left, right);
            }
            _ => break,
        }
    }
    Ok(left)
}

fn parse_unary(lexer: &mut Lexer) -> Result<Formula, ParseError> {
    let token = lexer.peek_token();
    match token {
        Some(Token::Not) => {
            lexer.next_token();
            let f = parse_unary(lexer)?;
            Ok(Formula::not(f))
        }
        Some(Token::Next) => {
            lexer.next_token();
            let f = parse_unary(lexer)?;
            Ok(Formula::next(f))
        }
        Some(Token::Finally) => {
            lexer.next_token();
            let f = parse_unary(lexer)?;
            Ok(Formula::finally(f))
        }
        Some(Token::Globally) => {
            lexer.next_token();
            let f = parse_unary(lexer)?;
            Ok(Formula::globally(f))
        }
        _ => parse_primary(lexer),
    }
}

fn parse_primary(lexer: &mut Lexer) -> Result<Formula, ParseError> {
    let token = lexer.next_token();
    match token {
        Some(Token::True) => Ok(Formula::True),
        Some(Token::False) => Ok(Formula::False),
        Some(Token::Atom(name)) => Ok(Formula::atom(&name)),
        Some(Token::LParen) => {
            let f = parse_implies(lexer)?;
            let next = lexer.next_token();
            match next {
                Some(Token::RParen) => Ok(f),
                _ => Err(ParseError {
                    message: "Expected ')'".to_string(),
                }),
            }
        }
        Some(t) => Err(ParseError {
            message: format!("Unexpected token: {:?}", t),
        }),
        None => Err(ParseError {
            message: "Unexpected end of input".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_atom() {
        let f = parse_formula("p").unwrap();
        assert_eq!(f, Formula::atom("p"));
    }

    #[test]
    fn test_parse_true_false() {
        assert_eq!(parse_formula("true").unwrap(), Formula::True);
        assert_eq!(parse_formula("false").unwrap(), Formula::False);
    }

    #[test]
    fn test_parse_not() {
        let f = parse_formula("!p").unwrap();
        assert_eq!(f, Formula::not(Formula::atom("p")));
    }

    #[test]
    fn test_parse_and() {
        let f = parse_formula("p AND q").unwrap();
        assert_eq!(f, Formula::and(Formula::atom("p"), Formula::atom("q")));
    }

    #[test]
    fn test_parse_or() {
        let f = parse_formula("p OR q").unwrap();
        assert_eq!(f, Formula::or(Formula::atom("p"), Formula::atom("q")));
    }

    #[test]
    fn test_parse_implies() {
        let f = parse_formula("p -> q").unwrap();
        assert_eq!(f, Formula::implies(Formula::atom("p"), Formula::atom("q")));
    }

    #[test]
    fn test_parse_globally_eventually() {
        let f = parse_formula("G(F(p))").unwrap();
        assert_eq!(f, Formula::globally(Formula::finally(Formula::atom("p"))));
    }

    #[test]
    fn test_parse_until() {
        let f = parse_formula("p U q").unwrap();
        assert_eq!(f, Formula::until(Formula::atom("p"), Formula::atom("q")));
    }

    #[test]
    fn test_parse_complex() {
        let f = parse_formula("G(request -> F response)").unwrap();
        assert_eq!(
            f,
            Formula::globally(Formula::implies(
                Formula::atom("request"),
                Formula::finally(Formula::atom("response")),
            ))
        );
    }

    #[test]
    fn test_parse_error() {
        assert!(parse_formula("(").is_err());
        assert!(parse_formula("p AND").is_err());
    }

    #[test]
    fn test_parse_parenthesized() {
        let f = parse_formula("(p OR q) AND r").unwrap();
        assert_eq!(
            f,
            Formula::and(
                Formula::or(Formula::atom("p"), Formula::atom("q")),
                Formula::atom("r"),
            )
        );
    }

    #[test]
    fn test_parse_next() {
        let f = parse_formula("X(p)").unwrap();
        assert_eq!(f, Formula::next(Formula::atom("p")));
    }

    #[test]
    fn test_parse_release() {
        let f = parse_formula("p R q").unwrap();
        assert_eq!(f, Formula::release(Formula::atom("p"), Formula::atom("q")));
    }

    #[test]
    fn test_parse_nested() {
        let f = parse_formula("G(p -> X(q))").unwrap();
        assert_eq!(
            f,
            Formula::globally(Formula::implies(
                Formula::atom("p"),
                Formula::next(Formula::atom("q")),
            ))
        );
    }
}
