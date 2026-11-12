use crate::ast::{BinOp, Block, Expr, ExprKind, Program, Stmt};
use crate::error::OnionError;

/// Performs additional static security checks.
///
/// This analyzer is intentionally conservative:
/// it only reports issues it can establish from
/// the available syntax tree.
///
/// Future rules can be added as Onion grows.
pub fn scan(program: &Program) -> Vec<OnionError> {
    let mut scanner = SecurityScanner {
        findings: Vec::new(),
    };

    for function in &program.functions {
        scanner.scan_block(&function.body);
    }

    scanner.findings
}

struct SecurityScanner {
    findings: Vec<OnionError>,
}

impl SecurityScanner {
    fn scan_block(&mut self, block: &Block) {
        for statement in &block.stmts {
            self.scan_statement(statement);
        }
    }

    fn scan_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Let { init, .. } => {
                self.scan_expr(init);
            }

            Stmt::Assign { value, .. } | Stmt::DerefAssign { value, .. } => {
                self.scan_expr(value);
            }

            Stmt::Println { args, .. } => {
                for expression in args {
                    self.scan_expr(expression);
                }
            }

            Stmt::If {
                cond,
                then_block,
                else_block,
            } => {
                self.scan_expr(cond);
                self.scan_block(then_block);

                if let Some(block) = else_block {
                    self.scan_block(block);
                }
            }

            Stmt::Loop { body } => {
                self.scan_block(body);
            }

            Stmt::Break { .. } | Stmt::Continue { .. } => {}
        }
    }

    fn scan_expr(&mut self, expression: &Expr) {
        match &expression.kind {
            ExprKind::Binary { op, lhs, rhs } => {
                self.scan_expr(lhs);
                self.scan_expr(rhs);

                if matches!(*op, BinOp::Div | BinOp::Rem) && is_literal_zero(rhs) {
                    let message = match op {
                        BinOp::Div => "SEC001: pembagian dengan nol terdeteksi",
                        _ => "SEC001: operasi modulo dengan nol terdeteksi",
                    };

                    self.findings.push(
                        OnionError::new(message, expression.span.clone())
                            .with_help("Pastikan pembagi bukan nol sebelum melakukan operasi."),
                    );
                }
            }

            ExprKind::Unary { operand, .. } => {
                self.scan_expr(operand);
            }

            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Str(_)
            | ExprKind::Var(_) => {}
        }
    }
}

fn is_literal_zero(expression: &Expr) -> bool {
    match &expression.kind {
        ExprKind::Int(value) => *value == 0,
        ExprKind::Float(value) => *value == 0.0,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;

    fn analyze(source: &str) -> Vec<OnionError> {
        let tokens = lex(source).unwrap();
        let program = parse(tokens, source.len()).unwrap();
        scan(&program)
    }

    #[test]
    fn safe_division_passes() {
        let findings = analyze("fn main() { let result = 10 / 2 }");

        assert!(findings.is_empty());
    }

    #[test]
    fn division_by_zero_is_detected() {
        let findings = analyze("fn main() { let result = 10 / 0 }");

        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("SEC001"));
    }

    #[test]
    fn modulo_by_zero_is_detected() {
        let findings = analyze("fn main() { let result = 10 % 0 }");

        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn nested_division_is_detected() {
        let findings = analyze("fn main() { let result = 5 + (10 / 0) }");

        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn division_inside_branch_is_detected() {
        let findings = analyze("fn main() { if true { let result = 10 / 0 } }");

        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn division_inside_loop_is_detected() {
        let findings = analyze("fn main() { loop { let result = 10 / 0 break } }");

        assert_eq!(findings.len(), 1);
    }
}
