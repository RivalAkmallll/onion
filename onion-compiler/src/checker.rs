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
    /// Set when ownership of a String has been transferred.
    moved: bool,
    id: usize,
    declared_depth: usize,
    ref_target: Option<usize>,
    ref_mutable: bool,
}

struct Checker {
    scopes: Vec<HashMap<String, VarInfo>>,
    errors: Vec<OnionError>,
    loop_depth: usize,
    next_id: usize,
    loop_scope_depths: Vec<usize>,
}

/// Type-checks and scope-checks the whole program.
/// Returns *every* error found, not just the first one.
pub fn check(program: &Program) -> Result<(), Vec<OnionError>> {
    let mut checker = Checker {
        scopes: Vec::new(),
        errors: Vec::new(),
        loop_depth: 0,
        next_id: 0,
        loop_scope_depths: Vec::new(),
    };
    checker.check_program(program);

    let security_findings = crate::security::scan(program);
    checker.errors.extend(security_findings);

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

fn expr_mentions(expr: &Expr, name: &str) -> bool {
    match &expr.kind {
        ExprKind::Var(found) => found == name,
        ExprKind::Unary { operand, .. } => expr_mentions(operand, name),
        ExprKind::Binary { lhs, rhs, .. } => expr_mentions(lhs, name) || expr_mentions(rhs, name),
        _ => false,
    }
}

// Control flow is a barrier, never evidence that a reference is dead.
fn stmt_mentions(stmt: &Stmt, name: &str) -> bool {
    match stmt {
        Stmt::Let {
            name: declared,
            init,
            ..
        } => declared == name || expr_mentions(init, name),
        Stmt::Assign {
            name: assigned,
            value,
            ..
        } => assigned == name || expr_mentions(value, name),
        Stmt::DerefAssign { target, value } => {
            expr_mentions(target, name) || expr_mentions(value, name)
        }
        Stmt::Println { args, .. } => args.iter().any(|arg| expr_mentions(arg, name)),
        _ => true,
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

    fn declare(&mut self, name: &str, mut info: VarInfo) {
        info.id = self.next_id;
        self.next_id += 1;
        info.declared_depth = self.scopes.len();
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), info);
        }
    }

    fn active_mutable_reference(&self, id: usize) -> bool {
        self.scopes.iter().any(|scope| {
            scope
                .values()
                .any(|binding| binding.ref_target == Some(id) && binding.ref_mutable)
        })
    }

    fn active_references(&self, id: usize) -> bool {
        self.scopes
            .iter()
            .any(|scope| scope.values().any(|binding| binding.ref_target == Some(id)))
    }

    // Reference creation is deliberately limited to a named, live binding.
    fn reference_origin(&self, expr: &Expr) -> Option<usize> {
        match &expr.kind {
            ExprKind::Unary {
                op: UnOp::Borrow | UnOp::BorrowMut,
                operand,
            } => {
                if let ExprKind::Var(name) = &operand.kind {
                    self.lookup(name).map(|info| info.id)
                } else {
                    None
                }
            }
            ExprKind::Var(name) => self.lookup(name).and_then(|info| info.ref_target),
            _ => None,
        }
    }

    fn binding_by_id(&self, id: usize) -> Option<VarInfo> {
        self.scopes
            .iter()
            .flat_map(|s| s.values())
            .find(|info| info.id == id)
            .copied()
    }

    fn lookup(&self, name: &str) -> Option<VarInfo> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn update_binding(&mut self, name: &str, f: impl FnOnce(&mut VarInfo)) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(info) = scope.get_mut(name) {
                f(info);
                return;
            }
        }
    }

    // Initial ownership model: only a bare String variable on the right
    // side of `let` or `=` is moved. Numbers and booleans remain Copy.
    // Compound expressions are not ownership transfers yet.
    fn consume_string_binding(&mut self, value: &Expr) {
        if let ExprKind::Unary {
            op: UnOp::Deref,
            operand,
        } = &value.kind
        {
            if let ExprKind::Var(name) = &operand.kind {
                if self
                    .lookup(name)
                    .and_then(|info| info.ty)
                    .is_some_and(|ty| ty.dereference() == Some(Type::Str))
                {
                    self.report(OnionError::new(
                        "cannot move String out of a reference",
                        value.span.clone(),
                    ));
                    return;
                }
            }
        }
        if let ExprKind::Var(name) = &value.kind {
            if let Some(info) = self.lookup(name) {
                if info.ty == Some(Type::Str) && !info.moved {
                    if self
                        .loop_scope_depths
                        .last()
                        .is_some_and(|depth| info.declared_depth <= *depth)
                    {
                        self.report(OnionError::new(
                            "moving an outer String inside a loop requires control-flow ownership analysis",
                            value.span.clone(),
                        ));
                        return;
                    }
                    if self.active_references(info.id) {
                        self.report(OnionError::new(
                            format!("cannot move `{name}` while it is borrowed"),
                            value.span.clone(),
                        ));
                    } else {
                        self.update_binding(name, |v| v.moved = true);
                    }
                }
            }
        }
    }

    fn check_reference_merge(
        &mut self,
        first: &[HashMap<String, VarInfo>],
        second: &[HashMap<String, VarInfo>],
    ) {
        for (i, scope) in first.iter().enumerate() {
            for (name, a) in scope {
                if let Some(b) = second[i].get(name) {
                    if a.ref_target != b.ref_target {
                        self.report(OnionError::without_span(format!(
                            "reference target differs across control-flow branches for `{name}`"
                        )));
                    }
                }
            }
        }
    }

    fn merge_moved_states(
        &mut self,
        first: &[HashMap<String, VarInfo>],
        second: &[HashMap<String, VarInfo>],
    ) {
        for (index, scope) in self.scopes.iter_mut().enumerate() {
            for (name, info) in scope.iter_mut() {
                let a = first[index].get(name).map(|v| v.moved).unwrap_or(false);
                let b = second[index].get(name).map(|v| v.moved).unwrap_or(false);
                info.moved = a || b;
                // Reject branch-dependent reference origins; they need dataflow analysis.
                let ref_a = first[index].get(name).and_then(|v| v.ref_target);
                let ref_b = second[index].get(name).and_then(|v| v.ref_target);
                if ref_a != ref_b {
                    info.ref_target = None;
                }
            }
        }
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
        let straight_line = self.loop_depth == 0
            && block.stmts.iter().all(|stmt| {
                matches!(
                    stmt,
                    Stmt::Let { .. }
                        | Stmt::Assign { .. }
                        | Stmt::DerefAssign { .. }
                        | Stmt::Println { .. }
                )
            });
        for (index, stmt) in block.stmts.iter().enumerate() {
            // Only local, non-escaping exclusive references in a straight-line
            // block qualify. Capture identities before shadowing can replace them.
            let ending: Vec<(String, usize)> = if straight_line {
                self.scopes
                    .last()
                    .into_iter()
                    .flat_map(|scope| scope.iter())
                    .filter(|(name, info)| {
                        info.ref_mutable
                            && info.ref_target.is_some()
                            && stmt_mentions(stmt, name)
                            && !block.stmts[index + 1..]
                                .iter()
                                .any(|later| stmt_mentions(later, name))
                    })
                    .map(|(name, info)| (name.clone(), info.id))
                    .collect()
            } else {
                Vec::new()
            };
            self.check_stmt(stmt);
            for (name, id) in ending {
                self.update_binding(&name, |info| {
                    if info.id == id {
                        info.ref_target = None;
                    }
                });
            }
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

                if init_ty.is_some() && var_ty == init_ty {
                    self.consume_string_binding(init);
                }
                if let Some(existing) = self.lookup(name) {
                    if self.active_references(existing.id) {
                        self.report(OnionError::new(
                            format!("cannot shadow `{name}` while it is borrowed"),
                            init.span.clone(),
                        ));
                    }
                }
                if var_ty.is_some_and(Type::is_mut_reference)
                    && !matches!(
                        &init.kind,
                        ExprKind::Unary {
                            op: UnOp::BorrowMut,
                            ..
                        }
                    )
                {
                    self.report(OnionError::new(
                        "mutable references cannot be copied or rebound yet",
                        init.span.clone(),
                    ));
                }
                let origin = if var_ty.and_then(Type::dereference).is_some() {
                    self.reference_origin(init)
                } else {
                    None
                };
                if let Some(id) = origin {
                    if let Some(source) = self.binding_by_id(id) {
                        if source.declared_depth > self.scopes.len() {
                            self.report(OnionError::new(
                                "reference outlives its source",
                                init.span.clone(),
                            ));
                        }
                    }
                }
                self.declare(
                    name,
                    VarInfo {
                        ty: var_ty,
                        mutable: *mutable,
                        moved: false,
                        id: 0,
                        declared_depth: 0,
                        ref_target: origin,
                        ref_mutable: var_ty.is_some_and(Type::is_mut_reference),
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
                        if info.ty.is_some_and(|ty| ty.dereference().is_some())
                            && self
                                .loop_scope_depths
                                .last()
                                .is_some_and(|depth| info.declared_depth <= *depth)
                        {
                            self.report(OnionError::new(
                                "reassigning an outer reference inside a loop requires control-flow lifetime analysis",
                                value.span.clone(),
                            ));
                        }
                        if info.ty.is_some_and(Type::is_mut_reference) {
                            self.report(OnionError::new(
                                "mutable references cannot be rebound yet",
                                value.span.clone(),
                            ));
                        }
                        if !info.mutable {
                            self.report(
                                OnionError::new(
                                    format!("cannot assign twice to immutable variable `{name}`"),
                                    name_span.clone(),
                                )
                                .with_help(format!("declare it with `let mut {name}`")),
                            );
                        }
                        if self.active_references(info.id) {
                            self.report(OnionError::new(
                                format!("cannot assign to `{name}` while it is borrowed"),
                                name_span.clone(),
                            ));
                        }
                        let origin = self.reference_origin(value);
                        if let Some(id) = origin {
                            if let Some(source) = self.binding_by_id(id) {
                                if source.declared_depth > info.declared_depth {
                                    self.report(OnionError::new(
                                        "reference does not live long enough: it would escape its scope",
                                        value.span.clone(),
                                    ));
                                }
                            }
                        }
                        if let (Some(expected), Some(found)) = (info.ty, value_ty) {
                            if expected != found {
                                self.report(mismatch(expected, found, value.span.clone()));
                            }
                        }
                        if info.mutable && info.ty.is_some() && info.ty == value_ty {
                            // Self-assignment is not treated as a move.
                            let from_self =
                                matches!(&value.kind, ExprKind::Var(src) if src == name);
                            if !from_self {
                                self.consume_string_binding(value);
                            }
                            self.update_binding(name, |v| {
                                v.moved = false;
                                v.ref_target = origin;
                            });
                        }
                    }
                }
            }

            Stmt::DerefAssign { target, value } => {
                let target_ty = self.check_expr(target);
                let value_ty = self.check_expr(value);
                match target_ty {
                    Some(ty) if ty.is_mut_reference() => {
                        if let (Some(inner), Some(found)) = (ty.dereference(), value_ty) {
                            if inner != found {
                                self.report(mismatch(inner, found, value.span.clone()));
                            } else {
                                self.consume_string_binding(value);
                            }
                        }
                        if let ExprKind::Var(name) = &target.kind {
                            if self.lookup(name).and_then(|info| info.ref_target).is_none() {
                                self.report(OnionError::new(
                                    "mutable reference has no valid target",
                                    target.span.clone(),
                                ));
                            }
                        }
                    }
                    Some(_) => self.report(OnionError::new(
                        "cannot assign through an immutable reference or non-reference",
                        target.span.clone(),
                    )),
                    None => {}
                }
            }

            Stmt::Println { args, .. } => {
                for arg in args {
                    if let Some(ty) = self.check_expr(arg) {
                        if ty.dereference().is_some() {
                            self.report(OnionError::new(
                                "print the referenced value using `*reference`",
                                arg.span.clone(),
                            ));
                        }
                    }
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
                let before = self.scopes.clone();
                self.check_block(then_block);
                let after_then = self.scopes.clone();
                self.scopes = before.clone();
                if let Some(block) = else_block {
                    self.check_block(block);
                }
                let after_else = self.scopes.clone();
                self.scopes = before;
                self.check_reference_merge(&after_then, &after_else);
                self.merge_moved_states(&after_then, &after_else);
            }

            Stmt::Loop { body } => {
                self.loop_depth += 1;
                self.loop_scope_depths.push(self.scopes.len());
                let before = self.scopes.clone();
                self.check_block(body);
                let after_once = self.scopes.clone();
                // Check another iteration to catch repeated moves of outer bindings.
                self.check_block(body);
                let after_twice = self.scopes.clone();
                self.scopes = before.clone();
                self.check_reference_merge(&after_once, &after_twice);
                self.merge_moved_states(&after_once, &after_twice);
                // An unreachable assignment after an exit must not restore an
                // outer binding that was already moved on loop entry.
                for (index, scope) in self.scopes.iter_mut().enumerate() {
                    for (name, info) in scope {
                        if let Some(entry) = before.get(index).and_then(|scope| scope.get(name)) {
                            info.moved |= entry.moved;
                        }
                    }
                }
                self.loop_scope_depths.pop();
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

    fn check_variable(&mut self, name: &str, span: &Span, read: bool) -> Option<Type> {
        match self.lookup(name) {
            Some(info) if info.moved => {
                self.report(
                    OnionError::new(format!("use of moved value `{name}`"), span.clone())
                        .with_help(format!(
                            "`{name}` was moved into another binding; use the new binding instead"
                        )),
                );
                None
            }
            Some(info) if read && self.active_mutable_reference(info.id) => {
                self.report(OnionError::new(
                    format!("cannot read `{name}` while mutably borrowed"),
                    span.clone(),
                ));
                None
            }
            Some(info) => info.ty,
            None => {
                self.report(OnionError::new(
                    format!("cannot find variable `{name}` in this scope"),
                    span.clone(),
                ));
                None
            }
        }
    }

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

            ExprKind::Var(name) => self.check_variable(name, &expr.span, true),

            ExprKind::Unary { op, operand } => {
                if *op == UnOp::BorrowMut {
                    let ExprKind::Var(name) = &operand.kind else {
                        self.report(OnionError::new(
                            "mutable borrowing requires a named variable",
                            expr.span.clone(),
                        ));
                        self.check_expr(operand);
                        return None;
                    };
                    let info = self.lookup(name);
                    let ty = self.check_variable(name, &operand.span, false)?;
                    let info = info?;
                    if !info.mutable {
                        self.report(OnionError::new(
                            format!("cannot mutably borrow immutable variable `{name}`"),
                            expr.span.clone(),
                        ));
                        return None;
                    }
                    if self.active_references(info.id) {
                        self.report(OnionError::new(
                            format!("cannot mutably borrow `{name}` while it is already borrowed"),
                            expr.span.clone(),
                        ));
                        return None;
                    }
                    if ty.dereference().is_some() {
                        self.report(OnionError::new(
                            "references to references are not supported yet",
                            expr.span.clone(),
                        ));
                        return None;
                    }
                    return ty.mutable_reference_to();
                }
                if *op == UnOp::Borrow {
                    let ExprKind::Var(name) = &operand.kind else {
                        self.report(OnionError::new(
                            "borrowing temporary expressions is not supported yet",
                            expr.span.clone(),
                        ));
                        self.check_expr(operand);
                        return None;
                    };
                    let info = self.lookup(name);
                    let ty = self.check_variable(name, &operand.span, false)?;
                    if info.and_then(|i| i.ref_target).is_some() || ty.dereference().is_some() {
                        self.report(OnionError::new(
                            "references to references are not supported yet",
                            expr.span.clone(),
                        ));
                        return None;
                    }
                    if let Some(info) = info {
                        if self.active_mutable_reference(info.id) {
                            self.report(OnionError::new(
                                format!("cannot immutably borrow `{name}` while mutably borrowed"),
                                expr.span.clone(),
                            ));
                            return None;
                        }
                    }
                    return ty.reference_to();
                }
                let ty = self.check_expr(operand)?;
                if *op == UnOp::Deref {
                    return match ty.dereference() {
                        Some(inner) => Some(inner),
                        None => {
                            self.report(OnionError::new(
                                format!("cannot dereference non-reference type `{ty}`"),
                                expr.span.clone(),
                            ));
                            None
                        }
                    };
                }
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
                    BinOp::Equal | BinOp::NotEqual if l == r && l.dereference().is_none() => {
                        Some(Type::Bool)
                    }
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

#[cfg(test)]
mod ownership_tests {
    use super::*;
    use crate::{lexer::lex, parser::parse};

    fn verify(src: &str) -> Result<(), Vec<OnionError>> {
        let program = parse(lex(src).unwrap(), src.len()).unwrap();
        check(&program)
    }

    #[test]
    fn string_move_rejects_old_binding() {
        let errors =
            verify(r#"fn main() { let a = "hello" let b = a println!("{}", a) }"#).unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("moved value `a`")));
    }

    #[test]
    fn string_move_allows_new_binding() {
        assert!(verify(r#"fn main() { let a = "hello" let b = a println!("{}", b) }"#).is_ok());
    }

    #[test]
    fn integer_is_copyable() {
        assert!(verify(r#"fn main() { let a = 42 let b = a println!("{} {}", a, b) }"#).is_ok());
    }

    #[test]
    fn conditional_move_invalidates_binding() {
        let errors =
            verify(r#"fn main() { let a = "hello" if true { let b = a } println!("{}", a) }"#)
                .unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("moved value `a`")));
    }

    #[test]
    fn mutable_reassignment_restores_moved_binding() {
        assert!(verify(
            r#"fn main() { let mut a = "hello" let b = a a = "new" println!("{}", a) }"#
        )
        .is_ok());
    }
}

#[cfg(test)]
mod immutable_borrow_tests {
    use super::*;
    use crate::{lexer::lex, parser::parse};

    fn verify(source: &str) -> Result<(), Vec<OnionError>> {
        let ast = parse(lex(source).unwrap(), source.len()).unwrap();
        check(&ast)
    }

    #[test]
    fn immutable_borrow_is_valid() {
        assert!(verify(
            r#"fn main() { let s = "hello" let r = &s println!("{}", *r) println!("{}", s) }"#
        )
        .is_ok());
    }

    #[test]
    fn cannot_move_borrowed_string() {
        let errors =
            verify(r#"fn main() { let s = "hello" let r = &s let moved = s println!("{}", *r) }"#)
                .unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("while it is borrowed")));
    }

    #[test]
    fn cannot_mutate_borrowed_variable() {
        let errors = verify(r#"fn main() { let mut x = 1 let r = &x x = 2 println!("{}", *r) }"#)
            .unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("while it is borrowed")));
    }

    #[test]
    fn cannot_escape_reference_scope() {
        let errors = verify(r#"fn main() { let a = 1 let mut r = &a if true { let b = 2 r = &b } println!("{}", *r) }"#).unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("does not live long enough")));
    }

    #[test]
    fn cannot_shadow_borrowed_variable() {
        let errors = verify(r#"fn main() { let a = 1 let r = &a let a = 2 println!("{}", *r) }"#)
            .unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("cannot shadow")));
    }

    #[test]
    fn mutable_borrow_writes_value() {
        assert!(
            verify(r#"fn main() { let mut x = 10 let r = &mut x *r = 25 println!("{}", x) }"#)
                .is_ok()
        );
    }

    #[test]
    fn conflicting_mutable_borrows_are_rejected() {
        let errors =
            verify("fn main() { let mut x = 1 let r = &mut x let s = &mut x }").unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("already borrowed")));
    }

    #[test]
    fn immutable_borrow_conflicts_with_mutable() {
        let errors = verify("fn main() { let mut x = 1 let r = &x let s = &mut x }").unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("already borrowed")));
    }

    #[test]
    fn mutable_borrow_conflicts_with_immutable() {
        let errors = verify("fn main() { let mut x = 1 let r = &mut x let s = &x }").unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("mutably borrowed")));
    }

    #[test]
    fn copying_mutable_reference_is_rejected() {
        let errors = verify("fn main() { let mut x = 1 let r = &mut x let s = r }").unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("cannot be copied")));
    }

    #[test]
    fn cannot_write_via_shared_reference() {
        let errors = verify("fn main() { let mut x = 1 let r = &x *r = 3 }").unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("immutable reference")));
    }

    #[test]
    fn cannot_mutably_borrow_immutable_variable() {
        let errors = verify("fn main() { let x = 1 let r = &mut x }").unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("immutable variable")));
    }

    #[test]
    fn cannot_move_string_out_of_reference() {
        let errors =
            verify(r#"fn main() { let mut x = "hello" let r = &mut x let y = *r }"#).unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("cannot move String")));
    }

    #[test]
    fn cannot_deref_non_reference() {
        let errors = verify(r#"fn main() { let a = 1 println!("{}", *a) }"#).unwrap_err();
        assert!(errors
            .iter()
            .any(|e| e.message.contains("cannot dereference")));
    }
}
