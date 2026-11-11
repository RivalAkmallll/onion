use crate::ast::*;
use crate::error::{OnionError, Span};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Int(i32),
    Float(f64),
    Bool(bool),
    Str(String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{v}"),
            // `{:?}` keeps the ".0" so floats print as `2.0`, not `2`.
            Value::Float(v) => write!(f, "{v:?}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Str(s) => f.write_str(s),
        }
    }
}

/// What happened after running a statement.
enum Flow {
    Normal,
    Break,
    Continue,
}

struct Interpreter {
    scopes: Vec<HashMap<String, Value>>,
}

/// Runs the program: finds `fn main()` and executes its body.
pub fn run(program: &Program) -> Result<(), OnionError> {
    let main_fn = program
        .functions
        .iter()
        .find(|f| f.name == "main")
        .ok_or_else(|| {
            OnionError::without_span("`main` function not found; add `fn main() { ... }`")
        })?;

    let mut interpreter = Interpreter { scopes: Vec::new() };
    interpreter.exec_block(&main_fn.body)?;
    Ok(())
}

fn internal_error(span: &Span) -> OnionError {
    OnionError::new(
        "internal error: invalid operands (the type checker should have caught this)",
        span.clone(),
    )
}

impl Interpreter {
    // ---------- environment ----------

    fn declare(&mut self, name: &str, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), value);
        }
    }

    fn assign(&mut self, name: &str, value: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(slot) = scope.get_mut(name) {
                *slot = value;
                return;
            }
        }
    }

    fn lookup(&self, name: &str, span: &Span) -> Result<Value, OnionError> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
            .cloned()
            .ok_or_else(|| {
                OnionError::new(
                    format!("internal error: undefined variable `{name}`"),
                    span.clone(),
                )
            })
    }

    // ---------- statements ----------

    fn exec_block(&mut self, block: &Block) -> Result<Flow, OnionError> {
        self.scopes.push(HashMap::new());
        let result = self.exec_stmts(&block.stmts);
        self.scopes.pop();
        result
    }

    fn exec_stmts(&mut self, stmts: &[Stmt]) -> Result<Flow, OnionError> {
        for stmt in stmts {
            match self.exec_stmt(stmt)? {
                Flow::Normal => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal)
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<Flow, OnionError> {
        match stmt {
            Stmt::Let { name, init, .. } => {
                let value = self.eval(init)?;
                self.declare(name, value);
            }

            Stmt::Assign { name, value, .. } => {
                let value = self.eval(value)?;
                self.assign(name, value);
            }

            Stmt::Println { parts, args } => {
                let mut values = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.eval(arg)?);
                }

                let mut out = String::new();
                let mut next = 0;
                for part in parts {
                    match part {
                        FormatPart::Lit(text) => out.push_str(text),
                        FormatPart::Placeholder => {
                            if let Some(value) = values.get(next) {
                                out.push_str(&value.to_string());
                            }
                            next += 1;
                        }
                    }
                }
                println!("{out}");
            }

            Stmt::If {
                cond,
                then_block,
                else_block,
            } => {
                let take_then = matches!(self.eval(cond)?, Value::Bool(true));
                if take_then {
                    return self.exec_block(then_block);
                }
                if let Some(block) = else_block {
                    return self.exec_block(block);
                }
            }

            Stmt::Loop { body } => loop {
                if let Flow::Break = self.exec_block(body)? {
                    break;
                }
            },

            Stmt::Break { .. } => return Ok(Flow::Break),
            Stmt::Continue { .. } => return Ok(Flow::Continue),
        }

        Ok(Flow::Normal)
    }

    // ---------- expressions ----------

    fn eval(&mut self, expr: &Expr) -> Result<Value, OnionError> {
        match &expr.kind {
            ExprKind::Int(v) => Ok(Value::Int(*v as i32)),
            ExprKind::Float(v) => Ok(Value::Float(*v)),
            ExprKind::Bool(v) => Ok(Value::Bool(*v)),
            ExprKind::Str(s) => Ok(Value::Str(s.clone())),
            ExprKind::Var(name) => self.lookup(name, &expr.span),

            ExprKind::Unary { op, operand } => {
                let value = self.eval(operand)?;
                match (*op, value) {
                    (UnOp::Neg, Value::Int(i)) => {
                        i.checked_neg().map(Value::Int).ok_or_else(|| {
                            OnionError::new("arithmetic overflow in negation", expr.span.clone())
                        })
                    }
                    (UnOp::Neg, Value::Float(x)) => Ok(Value::Float(-x)),
                    (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    _ => Err(internal_error(&expr.span)),
                }
            }

            ExprKind::Binary { op, lhs, rhs } => {
                // `&&` and `||` short-circuit: the right side may never run.
                if matches!(op, BinOp::And | BinOp::Or) {
                    let left = self.eval(lhs)?;
                    match (*op, &left) {
                        (BinOp::And, Value::Bool(false)) => return Ok(Value::Bool(false)),
                        (BinOp::Or, Value::Bool(true)) => return Ok(Value::Bool(true)),
                        _ => {}
                    }
                    return self.eval(rhs);
                }

                let left = self.eval(lhs)?;
                let right = self.eval(rhs)?;
                apply_binary(*op, left, right, &expr.span)
            }
        }
    }
}

fn apply_binary(op: BinOp, left: Value, right: Value, span: &Span) -> Result<Value, OnionError> {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => int_op(op, a, b, span),

        (Value::Float(a), Value::Float(b)) => {
            float_op(op, a, b).ok_or_else(|| internal_error(span))
        }

        (Value::Bool(a), Value::Bool(b)) => match op {
            BinOp::Equal => Ok(Value::Bool(a == b)),
            BinOp::NotEqual => Ok(Value::Bool(a != b)),
            _ => Err(internal_error(span)),
        },

        (Value::Str(a), Value::Str(b)) => match op {
            BinOp::Add => Ok(Value::Str(a + &b)),
            BinOp::Equal => Ok(Value::Bool(a == b)),
            BinOp::NotEqual => Ok(Value::Bool(a != b)),
            _ => Err(internal_error(span)),
        },

        _ => Err(internal_error(span)),
    }
}

/// i32 arithmetic with overflow and division-by-zero detection.
fn int_op(op: BinOp, a: i32, b: i32, span: &Span) -> Result<Value, OnionError> {
    let overflow = || {
        OnionError::new(
            format!("arithmetic overflow in `{a} {op} {b}`"),
            span.clone(),
        )
    };

    let value = match op {
        BinOp::Add => a.checked_add(b).map(Value::Int).ok_or_else(overflow)?,
        BinOp::Sub => a.checked_sub(b).map(Value::Int).ok_or_else(overflow)?,
        BinOp::Mul => a.checked_mul(b).map(Value::Int).ok_or_else(overflow)?,
        BinOp::Div | BinOp::Rem if b == 0 => {
            return Err(OnionError::new("attempt to divide by zero", span.clone()))
        }
        BinOp::Div => a.checked_div(b).map(Value::Int).ok_or_else(overflow)?,
        BinOp::Rem => a.checked_rem(b).map(Value::Int).ok_or_else(overflow)?,
        BinOp::Equal => Value::Bool(a == b),
        BinOp::NotEqual => Value::Bool(a != b),
        BinOp::Less => Value::Bool(a < b),
        BinOp::LessEq => Value::Bool(a <= b),
        BinOp::Greater => Value::Bool(a > b),
        BinOp::GreaterEq => Value::Bool(a >= b),
        BinOp::And | BinOp::Or => return Err(internal_error(span)),
    };
    Ok(value)
}

fn float_op(op: BinOp, a: f64, b: f64) -> Option<Value> {
    Some(match op {
        BinOp::Add => Value::Float(a + b),
        BinOp::Sub => Value::Float(a - b),
        BinOp::Mul => Value::Float(a * b),
        BinOp::Div => Value::Float(a / b),
        BinOp::Rem => Value::Float(a % b),
        BinOp::Equal => Value::Bool(a == b),
        BinOp::NotEqual => Value::Bool(a != b),
        BinOp::Less => Value::Bool(a < b),
        BinOp::LessEq => Value::Bool(a <= b),
        BinOp::Greater => Value::Bool(a > b),
        BinOp::GreaterEq => Value::Bool(a >= b),
        BinOp::And | BinOp::Or => return None,
    })
}
