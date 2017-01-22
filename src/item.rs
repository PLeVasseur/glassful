use std::fmt::Write;
use syntax::ast;
use syntax::abi;
use syntax::parse::ParseSess;

pub fn translate(sess: &ParseSess, out: &mut String, item: &ast::Item) {
    let diag = &sess.span_diagnostic;

    match item.vis {
        ast::Visibility::Inherited => (),
        _ => diag.span_err(item.span, "`pub` visibility has no meaning"),
    }

    match item.node {
        ast::ItemKind::Static(ref ty, muta, ref expr) => {
            match muta {
                ast::Mutability::Immutable => (),
                _ => diag.span_err(item.span, "variables are implicitly mutable"),
            }

            ::var::translate(sess, out, &item.attrs[..], item.ident,
                             &**ty, Some(&**expr));
        }

        ast::ItemKind::Const(ref ty, ref expr) => {
            write!(out, "const ").unwrap();
            ::var::translate(sess, out, &item.attrs[..], item.ident,
                             &**ty, Some(&**expr));
        }

        ast::ItemKind::Fn(ref decl, unsafety, _, abi, ref generics, ref block) => {
            for attr in item.attrs.iter() {
                diag.span_err(attr.span, "no function attributes are supported");
            }

            let ast::FnDecl { ref inputs, ref output, variadic }
                = **decl;

            if variadic {
                diag.span_err(item.span, "can't translate variadic functions");
            }

            match unsafety {
                ast::Unsafety::Normal => (),
                _ => diag.span_err(item.span, "can't translate unsafe functions"),
            }

            match abi {
                abi::Abi::Rust => (),
                _ => diag.span_err(item.span, "can't translate non-default ABI"),
            }

            if generics.is_parameterized() {
                diag.span_err(item.span, "can't translate generic functions");
            }

            let output = match *output {
                /*ast::NoReturn(..) => {
                    diag.span_err(item.span, "function doesn't return");
                    return;
                }*/
                ast::FunctionRetTy::Default(..) => None,
                ast::FunctionRetTy::Ty(ref t) => Some(&**t),
            };

            ::fun::translate(sess, out, item.ident, &inputs[..], output, &**block);
        }

        ast::ItemKind::Mac(_) => {
            diag.span_bug(item.span, "macros should be gone by now");
        }

        _ => {
            diag.span_err(item.span, "can't translate this sort of item");
        }
    }
}
