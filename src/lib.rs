#![feature(rustc_private, slice_patterns, plugin_registrar)]
//#![deny(warnings)]

extern crate syntax;

use std::borrow::ToOwned;
use std::thread;
use syntax::parse::{self, ParseSess};
use syntax::ext::base::ExtCtxt;
use syntax::ext::base::DummyResolver;
use syntax::ext::expand;
//use syntax::attr::AttrMetaMethods;

mod item;
mod var;
mod util;
mod ty;
mod fun;
mod expr;
mod block;

const NAME: &'static str = "<glassful shader>";

/// Translate a glassful program to GLSL, or panic.
pub fn translate(source: String) -> String {
    // parse
    let sess = ParseSess::new();
    let diag = &sess.span_diagnostic;
    let krate = parse::parse_crate_from_source_str(
        NAME.to_owned(), source, &sess);

    diag.abort_if_errors();


    // expand macros
    let ecfg = expand::ExpansionConfig::default(NAME.to_owned());
    let mut resolver = DummyResolver;
    let mut ctxt = ExtCtxt::new(&sess, ecfg, &mut resolver);
    let mut macexpdr = expand::MacroExpander::new(&mut ctxt, false);

    let krate = macexpdr.expand_crate(krate.unwrap());

    // process attributes
    let mut glsl_version = None;
    for attr in krate.attrs.iter() {
        if attr.check_name("version") {
            if let Some(val) = attr.value_str() {
                if glsl_version.is_some() {
                    diag.span_err(attr.span, "version given twice");
                }
                glsl_version = Some(val.as_str());
            } else {
                diag.span_err(attr.span, "version not given");
            }
        } else {
            diag.span_err(attr.span, "unknown attribute");
        }
    }

    diag.abort_if_errors();

    // translate!

    let mut out = match glsl_version {
        Some(v) => format!("#version {}\n\n", v),
        None => "".to_owned(),
    };

    for item in krate.module.items.iter() {
        item::translate(&sess, &mut out, &**item);
    }

    diag.abort_if_errors();

    out
}

/// Translate a glassful program to GLSL, or return `None'.
///
/// Because the `libsyntax` parser uses `panic!` internally,
/// this spawns a new thread for the duration of the call.
pub fn try_translate(source: String) -> Option<String> {
    match thread::spawn(move || { translate(source) }).join() {
        Ok(s) => Some(s),
        Err(_) => None
    }
}
