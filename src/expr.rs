use std::fmt::Write;
use syntax::ast;
use syntax::parse::ParseSess;
use syntax::codemap::Span;
use syntax::errors::Handler;

fn get_binop(diag: &Handler, sp: Span, binop: ast::BinOp) -> &'static str {
    match binop.node {
        ast::BinOpKind::Add => "+",
        ast::BinOpKind::Sub => "-",
        ast::BinOpKind::Mul => "*",
        ast::BinOpKind::Div => "/",
        ast::BinOpKind::And => "&&",
        ast::BinOpKind::Or => "||",
        ast::BinOpKind::Eq => "==",
        ast::BinOpKind::Lt => "<",
        ast::BinOpKind::Le => "<=",
        ast::BinOpKind::Ne => "!=",
        ast::BinOpKind::Ge => ">=",
        ast::BinOpKind::Gt => ">",
        _ => {
            diag.span_err(sp, "binary operator not supported");
            ""
        }
    }
}

fn get_unop(diag: &Handler, sp: Span, unop: ast::UnOp) -> &'static str {
    match unop {
        ast::UnOp::Not => "!",
        ast::UnOp::Neg => "-",
        _ => {
            diag.span_err(sp, "unary operator not supported");
            ""
        }
    }
}

pub fn translate(sess: &ParseSess,
                 out: &mut String,
                 expr: &ast::Expr) {
    let diag = &sess.span_diagnostic;

    match expr.node {
        ast::ExprKind::Lit(ref lit) => match lit.node {
            ast::LitKind::Int(n, _) => {
                write!(out, "{}", n).unwrap();
            }
            ast::LitKind::Float(ref f, _) | ast::LitKind::FloatUnsuffixed(ref f) => {
                write!(out, "{}", f).unwrap();
            }
            _ => {
                diag.span_err(expr.span, "can't translate this literal");
            }
        },

        ast::ExprKind::Path(_, ref p) => match ::util::simple_path(p) {
            Some(name) => {
                let name = match &name[..] {
                    "mod_" => "mod",
                    name => name,
                };
                write!(out, "{}", name).unwrap();
            }

            _ => {
                diag.span_err(expr.span, "can't translate qualified / parametrized name");
                return;
            }
        },

        ast::ExprKind::Binary(binop, ref lhs, ref rhs) => {
            write!(out, "(").unwrap();
            translate(sess, out, &**lhs);
            write!(out, " {} ", get_binop(diag, expr.span, binop)).unwrap();
            translate(sess, out, &**rhs);
            write!(out, ")").unwrap();
        }

        ast::ExprKind::Unary(unop, ref rhs) => {
            write!(out, "({} ", get_unop(diag, expr.span, unop)).unwrap();
            translate(sess, out, &**rhs);
            write!(out, ")").unwrap();
        }

        ast::ExprKind::If(ref cond, ref thn, ref els) => {
            write!(out, "if (").unwrap();
            translate(sess, out, &**cond);
            write!(out, ") {{\n").unwrap();
            ::block::translate(sess, out, &**thn, false);
            if let Some(els) = els.as_ref() {
                write!(out, "}}\nelse ").unwrap();
                translate(sess, out, &**els);
            } else {
                write!(out, "}}\n").unwrap();
            }
        }

        ast::ExprKind::Assign(ref lhs, ref rhs) => {
            write!(out, "(").unwrap();
            translate(sess, out, &**lhs);
            write!(out, " = ").unwrap();
            translate(sess, out, &**rhs);
            write!(out, ")").unwrap();
        }

        ast::ExprKind::Ret(ref val) => {
            write!(out, "return").unwrap();
            if let Some(val) = val.as_ref() {
                write!(out, " ").unwrap();
                translate(sess, out, &**val);
            }
            write!(out, ";\n").unwrap();
        }

        ast::ExprKind::Call(ref fun, ref args) => {
            translate(sess, out, &**fun);
            write!(out, "(").unwrap();
            for (i, arg) in args.iter().enumerate() {
                if i != 0 {
                    write!(out, ", ").unwrap();
                }
                translate(sess, out, &**arg);
            }
            write!(out, ")").unwrap();
        }

        ast::ExprKind::Field(ref lhs, id) => {
            translate(sess, out, &**lhs);
            write!(out, ".{}", id.node.name.as_str()).unwrap();
        }

        ast::ExprKind::Paren(ref inside) => translate(sess, out, &**inside),

        ast::ExprKind::Block(ref inside) => {
            write!(out, "{{\n").unwrap();
            ::block::translate(sess, out, &**inside, false);
            write!(out, "}}\n").unwrap();
        }

        ast::ExprKind::Mac(_) => {
            diag.span_bug(expr.span, "macros should be gone by now");
        }

        _ => {
            diag.span_err(expr.span, "can't translate this sort of expression");
        }
    }
}
