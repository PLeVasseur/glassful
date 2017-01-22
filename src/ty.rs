use std::fmt::Write;
use syntax::ast;
use syntax::parse::ParseSess;

pub fn translate(sess: &ParseSess,
                 out: &mut String,
                 ty: &ast::Ty) {
    let diag = &sess.span_diagnostic;

    match ty.node {
        ast::TyKind::Tup(ref t) if t.len() == 0 => {
            write!(out, "void").unwrap();
        }

        ast::TyKind::Path(_, ref p) => match ::util::simple_path(p) {
            None => {
                diag.span_err(ty.span, "can't translate qualified / parametrized name");
            }
            Some(name) => {
                let name = match &name[..] {
                    "f32" => "float",
                    name => name,
                };
                write!(out, "{}", name).unwrap();
            }
        },

        _ => {
            diag.span_err(ty.span, "can't translate this sort of type");
        }
    }
}
