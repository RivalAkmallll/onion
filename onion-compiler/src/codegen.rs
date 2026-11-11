use crate::ast::{
    BinOp, Block as AstBlock, Expr, ExprKind, FormatPart, Function, Program, Stmt, Type, UnOp,
};
use crate::error::OnionError;
use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::{
    types, AbiParam, Block, InstBuilder, MemFlags, Type as ClifType, Value,
};
use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{DataDescription, DataId, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use std::collections::HashMap;

type Res<T> = Result<T, String>;

/// Compiles the program to a native object file (COFF on Windows). Returns the bytes.
pub fn compile(program: &Program) -> Result<Vec<u8>, OnionError> {
    let main_fn = program
        .functions
        .iter()
        .find(|f| f.name == "main")
        .ok_or_else(|| OnionError::without_span("`main` function not found"))?;

    build(main_fn).map_err(|msg| OnionError::without_span(format!("code generation failed: {msg}")))
}

fn clif_type(ty: Type) -> ClifType {
    match ty {
        Type::I32 => types::I32,
        Type::F64 => types::F64,
        Type::Bool => types::I8,
        Type::Str => types::I64, // pointer to NUL-terminated bytes
    }
}

/// C runtime functions we call. Every one is imported from the CRT.
/// Note: printf always takes (i64, i64). Floats are passed as raw bits in an
/// integer register, because Win64 varargs read doubles from integer slots.
fn libc_sig(name: &str) -> (Vec<ClifType>, Option<ClifType>) {
    match name {
        "printf" => (vec![types::I64, types::I64], Some(types::I32)),
        "fflush" => (vec![types::I64], Some(types::I32)),
        "strlen" => (vec![types::I64], Some(types::I64)),
        "strcmp" => (vec![types::I64, types::I64], Some(types::I32)),
        "malloc" => (vec![types::I64], Some(types::I64)),
        "memcpy" => (vec![types::I64, types::I64, types::I64], Some(types::I64)),
        "fmod" => (vec![types::F64, types::F64], Some(types::F64)),
        "_write" => (vec![types::I32, types::I64, types::I32], Some(types::I32)),
        "exit" => (vec![types::I32], None),
        _ => panic!("unknown runtime function {name}"),
    }
}

// ------------------------------------------------------------ backend

struct Backend {
    module: ObjectModule,
    strings: HashMap<Vec<u8>, DataId>,
    funcs: HashMap<&'static str, FuncId>,
}

impl Backend {
    fn new() -> Res<Self> {
        let mut flags = settings::builder();
        flags.set("opt_level", "speed").map_err(|e| e.to_string())?;

        let isa = cranelift_native::builder()
            .map_err(|e| e.to_string())?
            .finish(settings::Flags::new(flags))
            .map_err(|e| e.to_string())?;

        let builder = ObjectBuilder::new(
            isa,
            "onion_program",
            cranelift_module::default_libcall_names(),
        )
        .map_err(|e| e.to_string())?;

        Ok(Self {
            module: ObjectModule::new(builder),
            strings: HashMap::new(),
            funcs: HashMap::new(),
        })
    }

    /// Stores a NUL-terminated string in the data section (deduplicated).
    fn string_data(&mut self, text: &str) -> Res<DataId> {
        let mut bytes = text.as_bytes().to_vec();
        bytes.push(0);

        if let Some(id) = self.strings.get(&bytes) {
            return Ok(*id);
        }

        let id = self
            .module
            .declare_anonymous_data(false, false)
            .map_err(|e| e.to_string())?;
        let mut desc = DataDescription::new();
        desc.define(bytes.clone().into_boxed_slice());
        self.module
            .define_data(id, &desc)
            .map_err(|e| e.to_string())?;

        self.strings.insert(bytes, id);
        Ok(id)
    }

    fn func_id(&mut self, name: &'static str) -> Res<FuncId> {
        if let Some(id) = self.funcs.get(name) {
            return Ok(*id);
        }

        let (params, ret) = libc_sig(name);
        let mut sig = self.module.make_signature();
        for p in params {
            sig.params.push(AbiParam::new(p));
        }
        if let Some(r) = ret {
            sig.returns.push(AbiParam::new(r));
        }

        let id = self
            .module
            .declare_function(name, Linkage::Import, &sig)
            .map_err(|e| e.to_string())?;
        self.funcs.insert(name, id);
        Ok(id)
    }
}

fn build(main_fn: &Function) -> Res<Vec<u8>> {
    let mut be = Backend::new()?;

    let mut ctx = be.module.make_context();
    ctx.func.signature.returns.push(AbiParam::new(types::I32));
    let main_id = be
        .module
        .declare_function("main", Linkage::Export, &ctx.func.signature)
        .map_err(|e| e.to_string())?;

    let mut fb_ctx = FunctionBuilderContext::new();
    {
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut fb_ctx);
        let entry = builder.create_block();
        builder.switch_to_block(entry);

        let mut g = FnGen {
            be: &mut be,
            b: builder,
            scopes: vec![HashMap::new()],
            loops: Vec::new(),
            next_var: 0,
        };

        g.block(&main_fn.body)?;

        let zero = g.b.ins().iconst(types::I32, 0);
        g.b.ins().return_(&[zero]);
        g.b.seal_all_blocks();
        g.b.finalize();
    }

    be.module
        .define_function(main_id, &mut ctx)
        .map_err(|e| format!("{e:?}"))?;

    let product = be.module.finish();
    product.emit().map_err(|e| e.to_string())
}

// ----------------------------------------------------- function codegen

struct FnGen<'a, 'b> {
    be: &'a mut Backend,
    b: FunctionBuilder<'b>,
    scopes: Vec<HashMap<String, (Variable, Type)>>,
    /// (continue target, break target) for each enclosing loop.
    loops: Vec<(Block, Block)>,
    /// Counter for fresh Cranelift variable ids.
    next_var: usize,
}

fn int_cc(op: BinOp) -> Option<IntCC> {
    Some(match op {
        BinOp::Equal => IntCC::Equal,
        BinOp::NotEqual => IntCC::NotEqual,
        BinOp::Less => IntCC::SignedLessThan,
        BinOp::LessEq => IntCC::SignedLessThanOrEqual,
        BinOp::Greater => IntCC::SignedGreaterThan,
        BinOp::GreaterEq => IntCC::SignedGreaterThanOrEqual,
        _ => return None,
    })
}

fn float_cc(op: BinOp) -> Option<FloatCC> {
    Some(match op {
        BinOp::Equal => FloatCC::Equal,
        BinOp::NotEqual => FloatCC::NotEqual,
        BinOp::Less => FloatCC::LessThan,
        BinOp::LessEq => FloatCC::LessThanOrEqual,
        BinOp::Greater => FloatCC::GreaterThan,
        BinOp::GreaterEq => FloatCC::GreaterThanOrEqual,
        _ => return None,
    })
}

impl<'a, 'b> FnGen<'a, 'b> {
    // ---------- helpers ----------

    /// Calls a C runtime function.
    fn call(&mut self, name: &'static str, args: &[Value]) -> Res<Option<Value>> {
        let id = self.be.func_id(name)?;
        let fref = self.be.module.declare_func_in_func(id, self.b.func);
        let inst = self.b.ins().call(fref, args);
        Ok(self.b.inst_results(inst).first().copied())
    }

    fn str_ptr(&mut self, text: &str) -> Res<Value> {
        let id = self.be.string_data(text)?;
        let gv = self.be.module.declare_data_in_func(id, self.b.func);
        Ok(self.b.ins().global_value(types::I64, gv))
    }

    fn lookup(&self, name: &str) -> Res<(Variable, Type)> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
            .ok_or_else(|| format!("undefined variable `{name}`"))
    }

    /// If `cond` is true: print an error, flush, and exit(1). Otherwise continue.
    fn panic_if(&mut self, cond: Value, msg: &str) -> Res<()> {
        let bad = self.b.create_block();
        let ok = self.b.create_block();
        self.b.ins().brif(cond, bad, &[], ok, &[]);

        self.b.switch_to_block(bad);
        let text = format!("runtime error: {msg}\n");
        let ptr = self.str_ptr(&text)?;
        let len = self.b.ins().iconst(types::I32, text.len() as i64);
        let fd = self.b.ins().iconst(types::I32, 2);
        let null = self.b.ins().iconst(types::I64, 0);
        self.call("fflush", &[null])?;
        self.call("_write", &[fd, ptr, len])?;
        let code = self.b.ins().iconst(types::I32, 1);
        self.call("exit", &[code])?;
        // exit never returns, but the block still needs a terminator.
        self.b.ins().jump(ok, &[]);

        self.b.switch_to_block(ok);
        Ok(())
    }

    // ---------- printing ----------

    fn print_cstr(&mut self, text: &str) -> Res<()> {
        let fmt = self.str_ptr("%s")?;
        let ptr = self.str_ptr(text)?;
        self.call("printf", &[fmt, ptr])?;
        Ok(())
    }

    fn print_value(&mut self, value: Value, ty: Type) -> Res<()> {
        match ty {
            Type::I32 => {
                let fmt = self.str_ptr("%d")?;
                let wide = self.b.ins().sextend(types::I64, value);
                self.call("printf", &[fmt, wide])?;
            }
            Type::F64 => {
                // Whole numbers below 1e15 print like Onion: `2.0`. Others use %.15g.
                let abs = self.b.ins().fabs(value);
                let floor = self.b.ins().floor(value);
                let whole = self.b.ins().fcmp(FloatCC::Equal, value, floor);
                let limit = self.b.ins().f64const(1e15);
                let small = self.b.ins().fcmp(FloatCC::LessThan, abs, limit);
                let use_fixed = self.b.ins().band(whole, small);
                let fixed = self.str_ptr("%.1f")?;
                let general = self.str_ptr("%.15g")?;
                let fmt = self.b.ins().select(use_fixed, fixed, general);
                let bits = self.b.ins().bitcast(types::I64, MemFlags::new(), value);
                self.call("printf", &[fmt, bits])?;
            }
            Type::Bool => {
                let fmt = self.str_ptr("%s")?;
                let t = self.str_ptr("true")?;
                let f = self.str_ptr("false")?;
                let text = self.b.ins().select(value, t, f);
                self.call("printf", &[fmt, text])?;
            }
            Type::Str => {
                let fmt = self.str_ptr("%s")?;
                self.call("printf", &[fmt, value])?;
            }
        }
        Ok(())
    }

    // ---------- statements ----------

    fn block(&mut self, block: &AstBlock) -> Res<()> {
        self.scopes.push(HashMap::new());
        for stmt in &block.stmts {
            self.stmt(stmt)?;
        }
        self.scopes.pop();
        Ok(())
    }

    /// After `break` / `continue` the current block is finished.
    /// Switch to a fresh (unreachable) block so later code still has a home.
    fn start_dead_block(&mut self) {
        let dead = self.b.create_block();
        self.b.switch_to_block(dead);
    }

    fn stmt(&mut self, stmt: &Stmt) -> Res<()> {
        match stmt {
            Stmt::Let { name, init, .. } => {
                let (value, ty) = self.expr(init)?;
                let var = Variable::from_u32(self.next_var as u32);
                self.next_var += 1;
                self.b.declare_var(var, clif_type(ty));
                self.b.def_var(var, value);
                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert(name.clone(), (var, ty));
                }
            }

            Stmt::Assign { name, value, .. } => {
                let (val, _) = self.expr(value)?;
                let (var, _) = self.lookup(name)?;
                self.b.def_var(var, val);
            }

            Stmt::Println { parts, args } => {
                let mut values: Vec<(Value, Type)> = Vec::with_capacity(args.len());
                for arg in args {
                    values.push(self.expr(arg)?);
                }
                let mut next = 0;
                for part in parts {
                    match part {
                        FormatPart::Lit(text) => self.print_cstr(text)?,
                        FormatPart::Placeholder => {
                            let (value, ty) = values[next];
                            next += 1;
                            self.print_value(value, ty)?;
                        }
                    }
                }
                self.print_cstr("\n")?;
            }

            Stmt::If {
                cond,
                then_block,
                else_block,
            } => {
                let (c, _) = self.expr(cond)?;
                let then_b = self.b.create_block();
                let merge = self.b.create_block();
                let else_b = if else_block.is_some() {
                    self.b.create_block()
                } else {
                    merge
                };

                self.b.ins().brif(c, then_b, &[], else_b, &[]);

                self.b.switch_to_block(then_b);
                self.block(then_block)?;
                self.b.ins().jump(merge, &[]);

                if let Some(block) = else_block {
                    self.b.switch_to_block(else_b);
                    self.block(block)?;
                    self.b.ins().jump(merge, &[]);
                }

                self.b.switch_to_block(merge);
            }

            Stmt::Loop { body } => {
                let header = self.b.create_block();
                let exit = self.b.create_block();

                self.b.ins().jump(header, &[]);
                self.b.switch_to_block(header);

                self.loops.push((header, exit));
                self.block(body)?;
                self.loops.pop();

                self.b.ins().jump(header, &[]);
                self.b.switch_to_block(exit);
            }

            Stmt::Break { .. } => {
                let (_, exit) = match self.loops.last() {
                    Some(pair) => *pair,
                    None => return Err("`break` outside loop".to_string()),
                };
                self.b.ins().jump(exit, &[]);
                self.start_dead_block();
            }

            Stmt::Continue { .. } => {
                let (header, _) = match self.loops.last() {
                    Some(pair) => *pair,
                    None => return Err("`continue` outside loop".to_string()),
                };
                self.b.ins().jump(header, &[]);
                self.start_dead_block();
            }
        }
        Ok(())
    }

    // ---------- expressions ----------

    fn expr(&mut self, expr: &Expr) -> Res<(Value, Type)> {
        match &expr.kind {
            ExprKind::Int(v) => {
                let value = self.b.ins().iconst(types::I32, *v);
                Ok((value, Type::I32))
            }
            ExprKind::Float(v) => {
                let value = self.b.ins().f64const(*v);
                Ok((value, Type::F64))
            }
            ExprKind::Bool(v) => {
                let value = self.b.ins().iconst(types::I8, *v as i64);
                Ok((value, Type::Bool))
            }
            ExprKind::Str(s) => {
                let value = self.str_ptr(s)?;
                Ok((value, Type::Str))
            }

            ExprKind::Var(name) => {
                let (var, ty) = self.lookup(name)?;
                let value = self.b.use_var(var);
                Ok((value, ty))
            }

            ExprKind::Unary { op, operand } => {
                let (v, ty) = self.expr(operand)?;
                match (*op, ty) {
                    (UnOp::Neg, Type::I32) => {
                        let is_min = self.b.ins().icmp_imm(IntCC::Equal, v, i32::MIN as i64);
                        self.panic_if(is_min, "arithmetic overflow in negation")?;
                        let value = self.b.ins().ineg(v);
                        Ok((value, Type::I32))
                    }
                    (UnOp::Neg, Type::F64) => {
                        let value = self.b.ins().fneg(v);
                        Ok((value, Type::F64))
                    }
                    (UnOp::Not, Type::Bool) => {
                        let value = self.b.ins().icmp_imm(IntCC::Equal, v, 0);
                        Ok((value, Type::Bool))
                    }
                    _ => Err("invalid unary operand".to_string()),
                }
            }

            ExprKind::Binary { op, lhs, rhs } => self.binary(*op, lhs, rhs),
        }
    }

    fn binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr) -> Res<(Value, Type)> {
        if matches!(op, BinOp::And | BinOp::Or) {
            return self.short_circuit(op, lhs, rhs);
        }

        let (x, ty) = self.expr(lhs)?;
        let (y, _) = self.expr(rhs)?;

        match ty {
            Type::I32 => self.int_binary(op, x, y),
            Type::F64 => self.float_binary(op, x, y),
            Type::Bool => {
                let cc = if op == BinOp::Equal {
                    IntCC::Equal
                } else {
                    IntCC::NotEqual
                };
                let value = self.b.ins().icmp(cc, x, y);
                Ok((value, Type::Bool))
            }
            Type::Str => self.str_binary(op, x, y),
        }
    }

    /// `&&` and `||`: the right side only runs when needed.
    fn short_circuit(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr) -> Res<(Value, Type)> {
        let (l, _) = self.expr(lhs)?;
        let rhs_b = self.b.create_block();
        let merge = self.b.create_block();
        let result = self.b.append_block_param(merge, types::I8);

        if op == BinOp::And {
            self.b.ins().brif(l, rhs_b, &[], merge, &[l]);
        } else {
            self.b.ins().brif(l, merge, &[l], rhs_b, &[]);
        }

        self.b.switch_to_block(rhs_b);
        let (r, _) = self.expr(rhs)?;
        self.b.ins().jump(merge, &[r]);

        self.b.switch_to_block(merge);
        Ok((result, Type::Bool))
    }

    fn int_binary(&mut self, op: BinOp, x: Value, y: Value) -> Res<(Value, Type)> {
        if let Some(cc) = int_cc(op) {
            let value = self.b.ins().icmp(cc, x, y);
            return Ok((value, Type::Bool));
        }

        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul => {
                // Do the math in 64 bits, then check the result still fits in i32.
                let wx = self.b.ins().sextend(types::I64, x);
                let wy = self.b.ins().sextend(types::I64, y);
                let wide = match op {
                    BinOp::Add => self.b.ins().iadd(wx, wy),
                    BinOp::Sub => self.b.ins().isub(wx, wy),
                    _ => self.b.ins().imul(wx, wy),
                };
                let narrow = self.b.ins().ireduce(types::I32, wide);
                let back = self.b.ins().sextend(types::I64, narrow);
                let overflow = self.b.ins().icmp(IntCC::NotEqual, back, wide);
                self.panic_if(overflow, "arithmetic overflow")?;
                Ok((narrow, Type::I32))
            }
            BinOp::Div | BinOp::Rem => {
                let zero = self.b.ins().icmp_imm(IntCC::Equal, y, 0);
                self.panic_if(zero, "attempt to divide by zero")?;

                let is_min = self.b.ins().icmp_imm(IntCC::Equal, x, i32::MIN as i64);
                let is_neg1 = self.b.ins().icmp_imm(IntCC::Equal, y, -1);
                let both = self.b.ins().band(is_min, is_neg1);
                self.panic_if(both, "arithmetic overflow")?;

                let value = if op == BinOp::Div {
                    self.b.ins().sdiv(x, y)
                } else {
                    self.b.ins().srem(x, y)
                };
                Ok((value, Type::I32))
            }
            _ => Err("invalid i32 operator".to_string()),
        }
    }

    fn float_binary(&mut self, op: BinOp, x: Value, y: Value) -> Res<(Value, Type)> {
        if let Some(cc) = float_cc(op) {
            let value = self.b.ins().fcmp(cc, x, y);
            return Ok((value, Type::Bool));
        }

        let value = match op {
            BinOp::Add => self.b.ins().fadd(x, y),
            BinOp::Sub => self.b.ins().fsub(x, y),
            BinOp::Mul => self.b.ins().fmul(x, y),
            BinOp::Div => self.b.ins().fdiv(x, y),
            BinOp::Rem => match self.call("fmod", &[x, y])? {
                Some(v) => v,
                None => return Err("fmod returned nothing".to_string()),
            },
            _ => return Err("invalid f64 operator".to_string()),
        };
        Ok((value, Type::F64))
    }

    fn str_binary(&mut self, op: BinOp, x: Value, y: Value) -> Res<(Value, Type)> {
        match op {
            BinOp::Equal | BinOp::NotEqual => {
                let cmp = self.call_value("strcmp", &[x, y])?;
                let cc = if op == BinOp::Equal {
                    IntCC::Equal
                } else {
                    IntCC::NotEqual
                };
                let value = self.b.ins().icmp_imm(cc, cmp, 0);
                Ok((value, Type::Bool))
            }
            BinOp::Add => {
                // concat: malloc(len a + len b + 1), copy a, copy b with its NUL.
                let la = self.call_value("strlen", &[x])?;
                let lb = self.call_value("strlen", &[y])?;
                let sum = self.b.ins().iadd(la, lb);
                let total = self.b.ins().iadd_imm(sum, 1);
                let buf = self.call_value("malloc", &[total])?;

                self.call("memcpy", &[buf, x, la])?;
                let dst = self.b.ins().iadd(buf, la);
                let lb1 = self.b.ins().iadd_imm(lb, 1);
                self.call("memcpy", &[dst, y, lb1])?;
                Ok((buf, Type::Str))
            }
            _ => Err("invalid String operator".to_string()),
        }
    }

    /// Like `call`, but for functions that always return a value.
    fn call_value(&mut self, name: &'static str, args: &[Value]) -> Res<Value> {
        match self.call(name, args)? {
            Some(v) => Ok(v),
            None => Err(format!("`{name}` returned nothing")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::parse;

    fn compile_src(src: &str) -> Result<Vec<u8>, OnionError> {
        let program = parse(lex(src).unwrap(), src.len()).unwrap();
        compile(&program)
    }

    #[test]
    fn compiles_sample_to_object_bytes() {
        let src = r#"
            fn main() {
                let mut n = 0
                let pi = 3.5
                let name = "on" + "ion"
                loop {
                    n += 1
                    if n % 2 == 0 { continue }
                    if n > 5 && name == "onion" { break }
                }
                println!("{} {} {} {}", n, pi * 2.0, name, !true)
            }
        "#;
        let bytes = compile_src(src).unwrap();
        assert!(bytes.len() > 100);
    }

    #[test]
    fn missing_main_is_error() {
        assert!(compile_src("fn other() { }").is_err());
    }
}
