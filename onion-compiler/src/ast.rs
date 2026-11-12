use crate::error::Span;
use std::fmt;

// ---------------------------------------------------------------- types

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    I32,
    F64,
    Bool,
    Str,
    RefI32,
    RefF64,
    RefBool,
    RefStr,
    MutRefI32,
    MutRefF64,
    MutRefBool,
    MutRefStr,
}

impl Type {
    pub fn from_name(name: &str) -> Option<Type> {
        match name {
            "i32" => Some(Type::I32),
            "f64" => Some(Type::F64),
            "bool" => Some(Type::Bool),
            "String" => Some(Type::Str),
            _ => None,
        }
    }

    pub fn reference_to(self) -> Option<Type> {
        match self {
            Type::I32 => Some(Type::RefI32),
            Type::F64 => Some(Type::RefF64),
            Type::Bool => Some(Type::RefBool),
            Type::Str => Some(Type::RefStr),
            _ => None,
        }
    }

    pub fn mutable_reference_to(self) -> Option<Type> {
        match self {
            Type::I32 => Some(Type::MutRefI32),
            Type::F64 => Some(Type::MutRefF64),
            Type::Bool => Some(Type::MutRefBool),
            Type::Str => Some(Type::MutRefStr),
            _ => None,
        }
    }

    pub fn is_mut_reference(self) -> bool {
        matches!(
            self,
            Type::MutRefI32 | Type::MutRefF64 | Type::MutRefBool | Type::MutRefStr
        )
    }

    pub fn dereference(self) -> Option<Type> {
        match self {
            Type::RefI32 => Some(Type::I32),
            Type::RefF64 => Some(Type::F64),
            Type::RefBool => Some(Type::Bool),
            Type::RefStr | Type::MutRefStr => Some(Type::Str),
            Type::MutRefI32 => Some(Type::I32),
            Type::MutRefF64 => Some(Type::F64),
            Type::MutRefBool => Some(Type::Bool),
            _ => None,
        }
    }

    pub fn is_numeric(self) -> bool {
        matches!(self, Type::I32 | Type::F64)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Type::I32 => "i32",
            Type::F64 => "f64",
            Type::Bool => "bool",
            Type::Str => "String",
            Type::RefI32 => "&i32",
            Type::RefF64 => "&f64",
            Type::RefBool => "&bool",
            Type::RefStr => "&String",
            Type::MutRefI32 => "&mut i32",
            Type::MutRefF64 => "&mut f64",
            Type::MutRefBool => "&mut bool",
            Type::MutRefStr => "&mut String",
        })
    }
}

// ------------------------------------------------------------ operators

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Equal,
    NotEqual,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    And,
    Or,
}

impl BinOp {
    /// True for comparison and logical operators (their result is always `bool`).
    pub fn yields_bool(self) -> bool {
        !matches!(
            self,
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem
        )
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Rem => "%",
            BinOp::Equal => "==",
            BinOp::NotEqual => "!=",
            BinOp::Less => "<",
            BinOp::LessEq => "<=",
            BinOp::Greater => ">",
            BinOp::GreaterEq => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
    Borrow,
    BorrowMut,
    Deref,
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            UnOp::Neg => "-",
            UnOp::Not => "!",
            UnOp::Borrow => "&",
            UnOp::BorrowMut => "&mut ",
            UnOp::Deref => "*",
        })
    }
}

// ---------------------------------------------------------- expressions

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Var(String),
    Unary {
        op: UnOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

// ----------------------------------------------------------- statements

/// A piece of a `println!` format string.
#[derive(Debug, Clone, PartialEq)]
pub enum FormatPart {
    Lit(String),
    Placeholder,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `let [mut] name [: type] = init`
    Let {
        name: String,
        name_span: Span,
        mutable: bool,
        ty: Option<Type>,
        init: Expr,
    },
    /// `name = value` (compound forms like `+=` are desugared by the parser)
    Assign {
        name: String,
        name_span: Span,
        value: Expr,
    },
    /// `*reference = value` (mutable-reference assignment)
    DerefAssign {
        target: Expr,
        value: Expr,
    },
    /// `println!("... {} ...", args)`
    Println {
        parts: Vec<FormatPart>,
        args: Vec<Expr>,
    },
    If {
        cond: Expr,
        then_block: Block,
        else_block: Option<Block>,
    },
    Loop {
        body: Block,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
}

// ------------------------------------------------------------- program

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub name_span: Span,
    pub body: Block,
}

/// The entire contents of a single `.oni` file.
#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Function>,
}
