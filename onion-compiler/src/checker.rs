use crate::ast::*;
use crate::error::{OnionError, Span};
use std::collections::{HashMap, HashSet};

/// Information the checker remembers about every declared variable.
#[derive(Clone, Copy)]
struct VarInfo {
    /// `None` means "type unknown because an earlier error already happened".
    /// This stops one mistake from causing a cascade of follow-up errors.
    ty: Option<Type>,
    mutable: bool,
}

struct Checker {
    scopes: Vec<HashMap<String, VarInfo>>,
    errors: Vec<OnionError>,
    loop_depth: usize,
}

/// Type-checks and scope-checks the whole program.
/// Returns *every* error found, not just the first one.
pub fn check(program: &Program) -> Result<(), Vec<OnionError>> {
    let mut checker = Checker {
        scopes: Vec::new(),
        errors: Vec::new(),
        loop_depth: 0,
    };
    checker.check_program(program);

    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(checker.errors)
    }
}

fn mismatch(expected: Type, found: Type, span: Span) -> OnionError {
    let error = OnionError::new(
        format!("mismatched types: expected `{expected}`, found `{found}`"),
        span,
    );
    if expected.is_numeric() && found.is_numeric() {
        error.with_help(
            "Onion never converts between i32 and f64 implicitly (write `5.0` for a float literal)",
        )
    } else {
        error
    }
}

fn binary_error(op: BinOp, left: Type, right: Type, span: Span) -> OnionError {
    if left != right {
        let error = OnionError::new(
            format!("mismatched types: cannot apply `{op}` to `{left}` and `{right}`"),
            span,
        );
        if left.is_numeric() && right.is_numeric() {
            error.with_help("Onion never converts between i32 and f64 implicitly")
        } else {
            error
        }
    } else {
        OnionError::new(
            format!("operator `{op}` cannot be applied to `{left}` values"),
            span,
        )
    }
}

impl Checker {
    /// Records an error, skipping exact duplicates (same message and location).
    fn report(&mut self, error: OnionError) {
        let duplicate = self
            .errors
            .iter()
            .any(|e| e.message == error.message && e.span == error.span);
        if !duplicate {
            self.errors.push(error);
        }
    }

    fn declare(&mut self, name: &str, info: VarInfo) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), info);
        }
    }

    fn lookup(&self, name: &str) -> Option<VarInfo> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    // ---------- structure ----------

    fn check_program(&mut self, program: &Program) {
        let mut seen: HashSet<&str> = HashSet::new();

        for function in &program.functions {
            if !seen.insert(function.name.as_str()) {
                self.report(OnionError::new(
                    format!("function `{}` is defined more than once", function.name),
                    function.name_span.clone(),
                ));
                continue;
            }
            self.check_block(&function.body);
        }

        if !program.functions.iter().any(|f| f.name == "main") {
            self.report(OnionError::without_span(
                "`main` function not found; add `fn main() { ... }`",
            ));
        }
    }

    fn check_block(&mut self, block: &Block) {
        self.scopes.push(HashMap::new());
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
        self.scopes.pop();
    }

    // ---------- statements ----------

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let {
                name,
                mutable,
                ty,
                init,
                ..
            } => {
                // Check the initializer BEFORE declaring, so `let x = x + 1`
                // refers to the previous `x`.
                let init_ty = self.check_expr(init);

                let var_ty = match (*ty, init_ty) {
                    (Some(expected), Some(found)) if expected != found => {
                        self.report(mismatch(expected, found, init.span.clone()));
                        Some(expected)
                    }
                    (Some(expected), _) => Some(expected),
                    (None, inferred) => inferred,
                };

                self.declare(
                    name,
                    VarInfo {
                        ty: var_ty,
                        mutable: *mutable,
                    },
                );
            }

            Stmt::Assign {
                name,
                name_span,
                value,
            } => {
                let value_ty = self.check_expr(value);

                match self.lookup(name) {
                    None => self.report(OnionError::new(
                        format!("cannot find variable `{name}` in this scope"),
                        name_span.clone(),
                    )),
                    Some(info) => {
                        if !info.mutable {
                            self.report(
                                OnionError::new(
                                    format!("cannot assign twice to immutable variable `{name}`"),
                                    name_span.clone(),
                                )
                                .with_help(format!("declare it with `let mut {name}`")),
                            );
                        }
                        if let (Some(expected), Some(found)) = (info.ty, value_ty) {
                            if expected != found {
                                self.report(mismatch(expected, found, value.span.clone()));
                            }
                        }
                    }
                }
            }

            Stmt::Println { args, .. } => {
                for arg in args {
                    self.check_expr(arg);
                }
            }

            Stmt::If {
                cond,
                then_block,
                else_block,
            } => {
                if let Some(found) = self.check_expr(cond) {
                    if found != Type::Bool {
                        self.report(mismatch(Type::Bool, found, cond.span.clone()).with_help(
                            "conditions must be `bool`; Onion has no truthy or falsy values",
                        ));
                    }
                }
                self.check_block(then_block);
                if let Some(block) = else_block {
                    self.check_block(block);
                }
            }

            Stmt::Loop { body } => {
                self.loop_depth += 1;
                self.check_block(body);
                self.loop_depth -= 1;
            }

            Stmt::Break { span } => {
                if self.loop_depth == 0 {
                    self.report(OnionError::new(
                        "`break` used outside of a loop",
                        span.clone(),
                    ));
                }
            }

            Stmt::Continue { span } => {
                if self.loop_depth == 0 {
                    self.report(OnionError::new(
                        "`continue` used outside of a loop",
                        span.clone(),
                    ));
                }
            }
        }
    }

    // ---------- expressions ----------

    /// Returns the expression's type, or `None` if an error was already reported.
    fn check_expr(&mut self, expr: &Expr) -> Option<Type> {
        match &expr.kind {
            ExprKind::Int(value) => {
                if *value > i32::MAX as i64 {
                    self.report(
                        OnionError::new(
                            format!("integer literal `{value}` is out of range for `i32`"),
                            expr.span.clone(),
                        )
                        .with_help("i32 holds values up to 2147483647"),
                    );
                    None
                } else {
                    Some(Type::I32)
                }
            }
            ExprKind::Float(_) => Some(Type::F64),
            ExprKind::Bool(_) => Some(Type::Bool),
            ExprKind::Str(_) => Some(Type::Str),

            ExprKind::Var(name) => match self.lookup(name) {
                Some(info) => info.ty,
                None => {
                    self.report(OnionError::new(
                        format!("cannot find variable `{name}` in this scope"),
                        expr.span.clone(),
                    ));
                    None
                }
            },

            ExprKind::Unary { op, operand } => {
                let ty = self.check_expr(operand)?;
                match (*op, ty) {
                    (UnOp::Neg, Type::I32) | (UnOp::Neg, Type::F64) => Some(ty),
                    (UnOp::Not, Type::Bool) => Some(Type::Bool),
                    _ => {
                        self.report(OnionError::new(
                            format!("cannot apply unary `{op}` to `{ty}`"),
                            expr.span.clone(),
                        ));
                        None
                    }
                }
            }

            ExprKind::Binary { op, lhs, rhs } => {
                let left = self.check_expr(lhs);
                let right = self.check_expr(rhs);

                let (l, r) = match (left, right) {
                    (Some(l), Some(r)) => (l, r),
                    // An operand already failed: don't pile on more errors.
                    _ => {
                        return if op.yields_bool() {
                            Some(Type::Bool)
                        } else {
                            None
                        }
                    }
                };

                let result = match *op {
                    BinOp::Add if l == r && (l.is_numeric() || l == Type::Str) => Some(l),
                    BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
                        if l == r && l.is_numeric() =>
                    {
                        Some(l)
                    }
                    BinOp::Equal | BinOp::NotEqual if l == r => Some(Type::Bool),
                    BinOp::Less | BinOp::LessEq | BinOp::Greater | BinOp::GreaterEq
                        if l == r && l.is_numeric() =>
                    {
                        Some(Type::Bool)
                    }
                    BinOp::And | BinOp::Or if l == Type::Bool && r == Type::Bool => {
                        Some(Type::Bool)
                    }
                    _ => None,
                };

                if result.is_none() {
                    self.report(binary_error(*op, l, r, expr.span.clone()));
                }
                result
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;

    fn check_src(src: &str) -> Result<(), Vec<OnionError>> {
        let program = parse(lex(src).unwrap(), src.len()).unwrap();
        check(&program)
    }

    #[test]
    fn valid_program_passes() {
        let src = r#"
            fn main() {
                let mut n = 0
                let pi = 3.14
                loop { n += 1 if n > 3 { break } }
                println!("{} {}", n, pi * 2.0)
            }
        "#;
        assert!(check_src(src).is_ok());
    }

    #[test]
    fn immutable_assignment_is_error() {
        let errors = check_src("fn main() { let x = 1 x = 2 }").unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("immutable"));
    }

    #[test]
    fn type_mismatch_is_error() {
        assert!(check_src("fn main() { let x: i32 = true }").is_err());
        assert!(check_src("fn main() { let x = 1 + 2.0 }").is_err());
    }

    #[test]
    fn undefined_variable_is_error() {
        assert!(check_src("fn main() { println!(\"{}\", nope) }").is_err());
    }

    #[test]
    fn break_outside_loop_is_error() {
        assert!(check_src("fn main() { break }").is_err());
    }

    #[test]
    fn shadowing_and_inner_scopes_work() {
        let src = "fn main() { let x = 1 let x = x + 1 if x == 2 { let y = true } }";
        assert!(check_src(src).is_ok());
    }

    #[test]
    fn inner_variable_does_not_leak() {
        let src = "fn main() { if true { let y = 1 } println!(\"{}\", y) }";
        assert!(check_src(src).is_err());
    }
}
