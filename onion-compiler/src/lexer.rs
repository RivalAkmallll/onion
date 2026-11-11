use crate::error::{OnionError, Span};
use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, Clone, PartialEq)]
// Whitespace and line comments (`// ...`) are ignored.
#[logos(skip r"([ \t\r\n\f]+|//[^\n]*)")]
pub enum Token {
    // ----- keywords -----
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("loop")]
    Loop,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("true")]
    True,
    #[token("false")]
    False,

    // ----- punctuation -----
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,

    // ----- operators -----
    #[token("!")]
    Bang,
    #[token("=")]
    Eq,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<")]
    Lt,
    #[token("<=")]
    LtEq,
    #[token(">")]
    Gt,
    #[token(">=")]
    GtEq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,

    // ----- literals & identifiers -----
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    Float(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Int(i64),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| unescape(lex.slice()))]
    StringLit(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text: &str = match self {
            Token::Fn => "fn",
            Token::Let => "let",
            Token::Mut => "mut",
            Token::If => "if",
            Token::Else => "else",
            Token::Loop => "loop",
            Token::Break => "break",
            Token::Continue => "continue",
            Token::True => "true",
            Token::False => "false",
            Token::LParen => "(",
            Token::RParen => ")",
            Token::LBrace => "{",
            Token::RBrace => "}",
            Token::Semicolon => ";",
            Token::Colon => ":",
            Token::Comma => ",",
            Token::Bang => "!",
            Token::Eq => "=",
            Token::EqEq => "==",
            Token::NotEq => "!=",
            Token::Lt => "<",
            Token::LtEq => "<=",
            Token::Gt => ">",
            Token::GtEq => ">=",
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Star => "*",
            Token::Slash => "/",
            Token::Percent => "%",
            Token::AndAnd => "&&",
            Token::OrOr => "||",
            Token::PlusEq => "+=",
            Token::MinusEq => "-=",
            Token::StarEq => "*=",
            Token::SlashEq => "/=",
            Token::Int(v) => return write!(f, "{v}"),
            Token::Float(v) => return write!(f, "{v:?}"),
            Token::Ident(s) => return write!(f, "{s}"),
            Token::StringLit(s) => return write!(f, "\"{s}\""),
        };
        f.write_str(text)
    }
}

/// Strips the surrounding quotes and processes escape sequences (\n, \t, \", \\).
fn unescape(raw: &str) -> String {
    let inner = &raw[1..raw.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('0') => out.push('\0'),
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Turns source code into a list of tokens along with their positions (spans).
pub fn lex(source: &str) -> Result<Vec<(Token, Span)>, OnionError> {
    let mut tokens = Vec::new();

    for (result, span) in Token::lexer(source).spanned() {
        match result {
            Ok(token) => tokens.push((token, span)),
            Err(()) => {
                let slice = &source[span.clone()];
                let message = if slice.starts_with('"') {
                    "string literal is not terminated with a closing `\"`".to_string()
                } else if slice.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    format!("integer literal `{slice}` is too large")
                } else {
                    format!("unrecognized character `{slice}`")
                };
                return Err(OnionError::new(message, span));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<Token> {
        lex(src).unwrap().into_iter().map(|(t, _)| t).collect()
    }

    #[test]
    fn lex_println() {
        assert_eq!(
            kinds(r#"println!("hi");"#),
            vec![
                Token::Ident("println".into()),
                Token::Bang,
                Token::LParen,
                Token::StringLit("hi".into()),
                Token::RParen,
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn lex_let_with_numbers() {
        assert_eq!(
            kinds("let mut x: i32 = 5 + 3.5"),
            vec![
                Token::Let,
                Token::Mut,
                Token::Ident("x".into()),
                Token::Colon,
                Token::Ident("i32".into()),
                Token::Eq,
                Token::Int(5),
                Token::Plus,
                Token::Float(3.5),
            ]
        );
    }

    #[test]
    fn lex_operators_prefer_longest_match() {
        assert_eq!(
            kinds("== != <= >= += && ||"),
            vec![
                Token::EqEq,
                Token::NotEq,
                Token::LtEq,
                Token::GtEq,
                Token::PlusEq,
                Token::AndAnd,
                Token::OrOr,
            ]
        );
    }

    #[test]
    fn lex_unknown_char() {
        assert!(lex("fn main() { @ }").is_err());
    }
}