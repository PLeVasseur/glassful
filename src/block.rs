use std::fmt::Write;
use syntax::ast;
use syntax::parse::ParseSess;

pub fn translate(sess: &ParseSess,
                 out: &mut String,
                 block: &ast::Block,
                 allow_return: bool) {
    let diag = &sess.span_diagnostic;

    for st in block.stmts.iter() {
        match st.node {
            ast::StmtKind::Local(ref loc) => translate_let(sess, out, &**loc),
            ast::StmtKind::Item(_) => {
                diag.span_err(st.span, "items in functions not supported")
            },
            ast::StmtKind::Expr(ref expr) | ast::StmtKind::Semi(ref expr) => {
                ::expr::translate(sess, out, &**expr);
                write!(out, ";\n").unwrap();
            }
            ast::StmtKind::Mac(..) => {
                diag.span_bug(st.span, "macros should be gone by now");
            }
        }
    }

    // ast::Block.expr removed
    /*if let Some(ref expr) = block.expr {
        if !allow_return {
            diag.span_err(expr.span, "can't translate a value-producing block here");
        }

        write!(out, "return ").unwrap();
        ::expr::translate(sess, out, &**expr);
        write!(out, ";\n").unwrap();
    }*/
}

fn translate_let(sess: &ParseSess,
                 out: &mut String,
                 loc: &ast::Local) {
    let diag = &sess.span_diagnostic;

    let name = match ::util::pat_to_var(&*loc.pat) {
        Some(n) => n,
        None => {
            diag.span_err(loc.span, "`let` binding must be a variable");
            return;
        }
    };

    let ty = match loc.ty.as_ref() {
        Some(t) => &**t,
        None => {
            diag.span_err(loc.span, "`let` bindings must specify a type");
            return;
        }
    };

    ::ty::translate(sess, out, ty);
    write!(out, " {}", name).unwrap();
    if let Some(init) = loc.init.as_ref() {
        write!(out, " = ").unwrap();
        ::expr::translate(sess, out, &**init);
    }
    write!(out, ";\n").unwrap();
}
