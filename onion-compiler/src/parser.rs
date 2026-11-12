use crate::ast::*;
use crate::error::{OnionError, Span};
use crate::lexer::Token;

/// PHASE 2 grammar (semicolons are optional everywhere):
///
/// ```text
/// program    := function*
/// function   := "fn" IDENT "(" ")" block
/// block      := "{" stmt* "}"
/// stmt       := let | if | loop | "break" | "continue" | println | assign
/// let        := "let" "mut"? IDENT (":" TYPE)? "=" expr ";"?
/// assign     := IDENT ("=" | "+=" | "-=" | "*=" | "/=") expr ";"?
/// println    := "println" "!" "(" STRING ("," expr)* ","? ")" ";"?
/// if         := "if" expr block ("else" (if | block))?
/// loop       := "loop" block
///
/// expr       := precedence climbing over:
///               ||  <  &&  <  == !=  <  < <= > >=  <  + -  <  * / %  <  unary (- !)
/// primary    := INT | FLOAT | STRING | "true" | "false" | IDENT | "(" expr ")"
/// ```
pub fn parse(tokens: Vec<(Token, Span)>, source_len: usize) -> Result<Program, OnionError> {
    Parser::new(tokens, source_len).parse_program()
}

struct Parser {
    tokens: Vec<(Token, Span)>,
    pos: usize,
    /// "End of file" span used for error messages when tokens run out.
    eof_span: Span,
}

/// Maps a token to a binary operator and its precedence (higher binds tighter).
fn binary_op(token: &Token) -> Option<(BinOp, u8)> {
    Some(match token {
        Token::OrOr => (BinOp::Or, 1),
        Token::AndAnd => (BinOp::And, 2),
        Token::EqEq => (BinOp::Equal, 3),
        Token::NotEq => (BinOp::NotEqual, 3),
        Token::Lt => (BinOp::Less, 4),
        Token::LtEq => (BinOp::LessEq, 4),
        Token::Gt => (BinOp::Greater, 4),
        Token::GtEq => (BinOp::GreaterEq, 4),
        Token::Plus => (BinOp::Add, 5),
        Token::Minus => (BinOp::Sub, 5),
        Token::Star => (BinOp::Mul, 6),
        Token::Slash => (BinOp::Div, 6),
        Token::Percent => (BinOp::Rem, 6),
        _ => return None,
    })
}

impl Parser {
    fn new(tokens: Vec<(Token, Span)>, source_len: usize) -> Self {
        let eof_start = source_len.saturating_sub(1);
        Self {
            tokens,
            pos: 0,
            eof_span: eof_start..source_len.max(eof_start),
        }
    }

    // ---------- helpers ----------

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    fn current_span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map(|(_, s)| s.clone())
            .unwrap_or_else(|| self.eof_span.clone())
    }

    fn advance(&mut self) -> Option<(Token, Span)> {
        let item = self.tokens.get(self.pos).cloned();
        if item.is_some() {
            self.pos += 1;
        }
        item
    }

    /// Builds an "expected X, but found Y" error for the current token.
    fn unexpected(&self, expected: &str) -> OnionError {
        match self.tokens.get(self.pos) {
            Some((tok, span)) => OnionError::new(
                format!("expected {expected}, but found `{tok}`"),
                span.clone(),
            ),
            None => OnionError::new(
                format!("expected {expected}, but reached end of file"),
                self.eof_span.clone(),
            ),
        }
    }

    /// Consumes exactly the expected token, or returns a clear error.
    fn expect(&mut self, expected: Token, description: &str) -> Result<Span, OnionError> {
        match self.tokens.get(self.pos) {
            Some((tok, span)) if *tok == expected => {
                let span = span.clone();
                self.pos += 1;
                Ok(span)
            }
            _ => Err(self.unexpected(description)),
        }
    }

    fn expect_ident(&mut self, description: &str) -> Result<(String, Span), OnionError> {
        match self.tokens.get(self.pos) {
            Some((Token::Ident(name), span)) => {
                let result = (name.clone(), span.clone());
                self.pos += 1;
                Ok(result)
            }
            _ => Err(self.unexpected(description)),
        }
    }

    /// Optional semicolon (Human-Centric Syntax).
    fn skip_semicolon(&mut self) {
        if matches!(self.peek(), Some(Token::Semicolon)) {
            self.pos += 1;
        }
    }

    // ---------- program structure ----------

    fn parse_program(&mut self) -> Result<Program, OnionError> {
        let mut functions = Vec::new();
        while self.peek().is_some() {
            functions.push(self.parse_function()?);
        }
        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<Function, OnionError> {
        self.expect(Token::Fn, "keyword `fn`")?;
        let (name, name_span) = self.expect_ident("a function name")?;
        self.expect(Token::LParen, "`(`")?;
        self.expect(Token::RParen, "`)`")?;
        let body = self.parse_block()?;
        Ok(Function {
            name,
            name_span,
            body,
        })
    }

    fn parse_block(&mut self) -> Result<Block, OnionError> {
        self.expect(Token::LBrace, "`{`")?;

        let mut stmts = Vec::new();
        while !matches!(self.peek(), Some(Token::RBrace) | None) {
            stmts.push(self.parse_stmt()?);
        }

        self.expect(Token::RBrace, "`}` to close the block")?;
        Ok(Block { stmts })
    }

    // ---------- statements ----------

    fn parse_stmt(&mut self) -> Result<Stmt, OnionError> {
        let tok = self.peek().cloned();
        match tok {
            Some(Token::Let) => self.parse_let(),
            Some(Token::If) => self.parse_if(),
            Some(Token::Loop) => self.parse_loop(),
            Some(Token::Break) => {
                let span = self.current_span();
                self.pos += 1;
                self.skip_semicolon();
                Ok(Stmt::Break { span })
            }
            Some(Token::Continue) => {
                let span = self.current_span();
                self.pos += 1;
                self.skip_semicolon();
                Ok(Stmt::Continue { span })
            }
            Some(Token::Ident(_)) => self.parse_ident_stmt(),
            Some(Token::Star) => self.parse_deref_assign(),
            _ => Err(self.unexpected("a statement")),
        }
    }

    fn parse_deref_assign(&mut self) -> Result<Stmt, OnionError> {
        let start = self.expect(Token::Star, "`*`")?;
        let (name, span) = self.expect_ident("reference variable after `*`")?;
        self.expect(Token::Eq, "`=` after dereference target")?;
        let value = self.parse_expr()?;
        self.skip_semicolon();
        Ok(Stmt::DerefAssign {
            target: Expr {
                kind: ExprKind::Var(name),
                span: start.start..span.end,
            },
            value,
        })
    }

    fn parse_let(&mut self) -> Result<Stmt, OnionError> {
        self.pos += 1; // `let`

        let mutable = if matches!(self.peek(), Some(Token::Mut)) {
            self.pos += 1;
            true
        } else {
            false
        };

        let (name, name_span) = self.expect_ident("a variable name")?;

        let ty = if matches!(self.peek(), Some(Token::Colon)) {
            self.pos += 1;
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(Token::Eq, "`=` followed by an initial value")?;
        let init = self.parse_expr()?;
        self.skip_semicolon();

        Ok(Stmt::Let {
            name,
            name_span,
            mutable,
            ty,
            init,
        })
    }

    fn parse_type(&mut self) -> Result<Type, OnionError> {
        let (name, span) = self.expect_ident("a type name")?;
        Type::from_name(&name).ok_or_else(|| {
            OnionError::new(format!("unknown type `{name}`"), span)
                .with_help("supported types are: i32, f64, bool, String")
        })
    }

    /// Statements that start with an identifier: `println!(...)` or an assignment.
    fn parse_ident_stmt(&mut self) -> Result<Stmt, OnionError> {
        let is_println = matches!(
            self.tokens.get(self.pos),
            Some((Token::Ident(n), _)) if n == "println"
        ) && matches!(self.tokens.get(self.pos + 1), Some((Token::Bang, _)));

        if is_println {
            return self.parse_println();
        }

        let (name, name_span) = self.expect_ident("a statement")?;

        // Which assignment operator follows? `None` inside means plain `=`.
        let compound = match self.peek().cloned() {
            Some(Token::Eq) => None,
            Some(Token::PlusEq) => Some(BinOp::Add),
            Some(Token::MinusEq) => Some(BinOp::Sub),
            Some(Token::StarEq) => Some(BinOp::Mul),
            Some(Token::SlashEq) => Some(BinOp::Div),
            _ => return Err(self.unexpected(&format!("`=` after `{name}`"))),
        };
        self.pos += 1;

        let rhs = self.parse_expr()?;
        self.skip_semicolon();

        // Desugar `x += e` into `x = x + e`.
        let value = match compound {
            None => rhs,
            Some(op) => {
                let span = name_span.start..rhs.span.end;
                let lhs = Expr {
                    kind: ExprKind::Var(name.clone()),
                    span: name_span.clone(),
                };
                Expr {
                    kind: ExprKind::Binary {
                        op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    },
                    span,
                }
            }
        };

        Ok(Stmt::Assign {
            name,
            name_span,
            value,
        })
    }

    fn parse_println(&mut self) -> Result<Stmt, OnionError> {
        self.pos += 2; // `println` `!`
        self.expect(Token::LParen, "`(`")?;

        let (format, format_span) = match self.advance() {
            Some((Token::StringLit(s), span)) => (s, span),
            Some((tok, span)) => {
                return Err(OnionError::new(
                    format!("expected a format string literal, but found `{tok}`"),
                    span,
                ))
            }
            None => {
                return Err(OnionError::new(
                    "expected a format string literal, but reached end of file",
                    self.eof_span.clone(),
                ))
            }
        };

        let mut args = Vec::new();
        while matches!(self.peek(), Some(Token::Comma)) {
            self.pos += 1;
            if matches!(self.peek(), Some(Token::RParen)) {
                break; // trailing comma
            }
            args.push(self.parse_expr()?);
        }

        self.expect(Token::RParen, "`)`")?;

        let parts = parse_format(&format, &format_span)?;
        let placeholders = parts
            .iter()
            .filter(|p| matches!(p, FormatPart::Placeholder))
            .count();

        if placeholders != args.len() {
            return Err(OnionError::new(
                format!(
                    "format string has {placeholders} placeholder(s) but {} argument(s) were given",
                    args.len()
                ),
                format_span,
            )
            .with_help("use one `{}` for each argument (write `{{` or `}}` for literal braces)"));
        }

        self.skip_semicolon();
        Ok(Stmt::Println { parts, args })
    }

    fn parse_if(&mut self) -> Result<Stmt, OnionError> {
        self.pos += 1; // `if`
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;

        let else_block = if matches!(self.peek(), Some(Token::Else)) {
            self.pos += 1;
            if matches!(self.peek(), Some(Token::If)) {
                // `else if` is an `else` block containing a single nested `if`.
                let nested = self.parse_if()?;
                Some(Block {
                    stmts: vec![nested],
                })
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        Ok(Stmt::If {
            cond,
            then_block,
            else_block,
        })
    }

    fn parse_loop(&mut self) -> Result<Stmt, OnionError> {
        self.pos += 1; // `loop`
        let body = self.parse_block()?;
        Ok(Stmt::Loop { body })
    }

    // ---------- expressions ----------

    fn parse_expr(&mut self) -> Result<Expr, OnionError> {
        self.parse_binary(1)
    }

    /// Precedence climbing. All binary operators are left-associative.
    fn parse_binary(&mut self, min_prec: u8) -> Result<Expr, OnionError> {
        let mut lhs = self.parse_unary()?;

        while let Some((op, prec)) = self.peek().and_then(binary_op) {
            // Semicolons are optional in Onion. In `let r = &mut x *r = 25`,
            // `*r =` starts a new dereference-assignment statement, not a
            // multiplication expression. An assignment sign cannot follow a
            // legal multiplication operand, so this lookahead is unambiguous.
            if matches!(self.peek(), Some(Token::Star))
                && matches!(self.tokens.get(self.pos + 1), Some((Token::Ident(_), _)))
                && matches!(self.tokens.get(self.pos + 2), Some((Token::Eq, _)))
            {
                break;
            }
            if prec < min_prec {
                break;
            }
            self.pos += 1;

            let rhs = self.parse_binary(prec + 1)?;
            let span = lhs.span.start..rhs.span.end;
            lhs = Expr {
                kind: ExprKind::Binary {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                span,
            };
        }

        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, OnionError> {
        let op = match self.peek() {
            Some(Token::Minus) => UnOp::Neg,
            Some(Token::Bang) => UnOp::Not,
            Some(Token::Star) => UnOp::Deref,
            Some(Token::Ampersand) => {
                if matches!(self.tokens.get(self.pos + 1), Some((Token::Mut, _))) {
                    UnOp::BorrowMut
                } else {
                    UnOp::Borrow
                }
            }
            _ => return self.parse_primary(),
        };

        let start = self.current_span();
        self.pos += 1;
        if op == UnOp::BorrowMut {
            self.pos += 1; // consume `mut` after `&`
        }
        let operand = self.parse_unary()?;
        let span = start.start..operand.span.end;

        Ok(Expr {
            kind: ExprKind::Unary {
                op,
                operand: Box::new(operand),
            },
            span,
        })
    }

    fn parse_primary(&mut self) -> Result<Expr, OnionError> {
        match self.advance() {
            Some((Token::Int(v), span)) => Ok(Expr {
                kind: ExprKind::Int(v),
                span,
            }),
            Some((Token::Float(v), span)) => Ok(Expr {
                kind: ExprKind::Float(v),
                span,
            }),
            Some((Token::True, span)) => Ok(Expr {
                kind: ExprKind::Bool(true),
                span,
            }),
            Some((Token::False, span)) => Ok(Expr {
                kind: ExprKind::Bool(false),
                span,
            }),
            Some((Token::StringLit(s), span)) => Ok(Expr {
                kind: ExprKind::Str(s),
                span,
            }),
            Some((Token::Ident(name), span)) => Ok(Expr {
                kind: ExprKind::Var(name),
                span,
            }),
            Some((Token::LParen, start)) => {
                let inner = self.parse_expr()?;
                let end = self.expect(Token::RParen, "`)` to close the group")?;
                Ok(Expr {
                    kind: inner.kind,
                    span: start.start..end.end,
                })
            }
            Some((tok, span)) => Err(OnionError::new(
                format!("expected an expression, but found `{tok}`"),
                span,
            )),
            None => Err(OnionError::new(
                "expected an expression, but reached end of file",
                self.eof_span.clone(),
            )),
        }
    }
}

/// Splits a `println!` format string into literal text and `{}` placeholders.
fn parse_format(format: &str, span: &Span) -> Result<Vec<FormatPart>, OnionError> {
    let mut parts = Vec::new();
    let mut lit = String::new();
    let mut chars = format.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '{' => match chars.peek() {
                Some('{') => {
                    chars.next();
                    lit.push('{');
                }
                Some('}') => {
                    chars.next();
                    if !lit.is_empty() {
                        parts.push(FormatPart::Lit(std::mem::take(&mut lit)));
                    }
                    parts.push(FormatPart::Placeholder);
                }
                _ => {
                    return Err(OnionError::new(
                        "invalid format string: `{` must be followed by `}`",
                        span.clone(),
                    )
                    .with_help("write `{{` for a literal `{`"));
                }
            },
            '}' => {
                if chars.peek() == Some(&'}') {
                    chars.next();
                    lit.push('}');
                } else {
                    return Err(OnionError::new(
                        "invalid format string: unmatched `}`",
                        span.clone(),
                    )
                    .with_help("write `}}` for a literal `}`"));
                }
            }
            other => lit.push(other),
        }
    }

    if !lit.is_empty() {
        parts.push(FormatPart::Lit(lit));
    }
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    fn parse_src(src: &str) -> Result<Program, OnionError> {
        parse(lex(src)?, src.len())
    }

    #[test]
    fn parse_hello() {
        let p = parse_src(r#"fn main() { println!("hi") }"#).unwrap();
        assert_eq!(p.functions.len(), 1);
        assert_eq!(p.functions[0].name, "main");
        assert_eq!(p.functions[0].body.stmts.len(), 1);
    }

    #[test]
    fn missing_brace_is_error() {
        assert!(parse_src(r#"fn main() { println!("hi")"#).is_err());
    }

    #[test]
    fn parse_let_if_loop_without_semicolons() {
        let src = "fn main() { let mut i = 0 loop { i += 1 if i > 3 { break } } }";
        let p = parse_src(src).unwrap();
        assert_eq!(p.functions[0].body.stmts.len(), 2);
    }

    #[test]
    fn precedence_multiplication_binds_tighter() {
        let p = parse_src("fn main() { let x = 1 + 2 * 3 }").unwrap();
        match &p.functions[0].body.stmts[0] {
            Stmt::Let { init, .. } => match &init.kind {
                ExprKind::Binary { op, rhs, .. } => {
                    assert_eq!(*op, BinOp::Add);
                    assert!(matches!(rhs.kind, ExprKind::Binary { op: BinOp::Mul, .. }));
                }
                _ => panic!("expected a binary expression"),
            },
            _ => panic!("expected a let statement"),
        }
    }

    #[test]
    fn format_placeholder_count_must_match() {
        assert!(parse_src(r#"fn main() { println!("{} {}", 1) }"#).is_err());
        assert!(parse_src(r#"fn main() { println!("{{}}") }"#).is_ok());
    }

    #[test]
    fn unknown_type_is_error() {
        assert!(parse_src("fn main() { let x: u8 = 1 }").is_err());
    }

    #[test]
    fn parse_deref_assignment_after_unterminated_let() {
        let p = parse_src("fn main() { let mut x = 10 let r = &mut x *r = 25 }").unwrap();
        assert_eq!(p.functions[0].body.stmts.len(), 3);
        assert!(matches!(
            p.functions[0].body.stmts[2],
            Stmt::DerefAssign { .. }
        ));
    }

    #[test]
    fn multiplication_remains_multiplication() {
        let p = parse_src("fn main() { let a = 2 let b = 3 let c = a * b }").unwrap();
        match &p.functions[0].body.stmts[2] {
            Stmt::Let { init, .. } => {
                assert!(matches!(init.kind, ExprKind::Binary { op: BinOp::Mul, .. }));
            }
            _ => panic!("expected let"),
        }
    }

    #[test]
    fn parse_mutable_deref_assignment() {
        let p = parse_src("fn main() { let mut score = 10 let r = &mut score *r = 25 }").unwrap();
        assert!(matches!(
            p.functions[0].body.stmts[2],
            Stmt::DerefAssign { .. }
        ));
    }

    #[test]
    fn parse_immutable_reference_syntax() {
        let p = parse_src(r#"fn main() { let message = "hello" let view = &message }"#).unwrap();
        match &p.functions[0].body.stmts[1] {
            Stmt::Let { init, .. } => assert!(matches!(
                &init.kind,
                ExprKind::Unary { op: UnOp::Borrow, operand }
                    if matches!(&operand.kind, ExprKind::Var(n) if n == "message")
            )),
            _ => panic!("expected let"),
        }
    }

    #[test]
    fn parse_mutable_reference_syntax() {
        let p = parse_src("fn main() { let mut score = 1 let view = &mut score }").unwrap();
        match &p.functions[0].body.stmts[1] {
            Stmt::Let { init, .. } => assert!(matches!(
                &init.kind,
                ExprKind::Unary {
                    op: UnOp::BorrowMut,
                    ..
                }
            )),
            _ => panic!("expected let"),
        }
    }

    #[test]
    fn parse_deref_expression_syntax() {
        let p = parse_src("fn main() { let value = 1 let view = *value }").unwrap();
        match &p.functions[0].body.stmts[1] {
            Stmt::Let { init, .. } => assert!(matches!(
                &init.kind,
                ExprKind::Unary {
                    op: UnOp::Deref,
                    ..
                }
            )),
            _ => panic!("expected let"),
        }
    }
}
