use std::path::Path;

use rhai::Engine;

use crate::color::{ColorChoice, Stream};
use crate::print;
use crate::trace;
use crate::{Error, Result};

pub(crate) fn run(engine: &Engine, path: &Path, args: &[String], color: ColorChoice) -> Result<()> {
    let out_style = color.style_for(Stream::Stdout);
    let err_style = color.style_for(Stream::Stderr);

    let source = std::fs::read_to_string(path).map_err(Error::Io)?;

    let ast = match engine.compile(&source) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!(
                "{}: {e}",
                err_style.error(&format!("parse error in {}", path.display()))
            );
            std::process::exit(1);
        }
    };

    let mut scope = rhai::Scope::new();
    let args_array: rhai::Array = args
        .iter()
        .map(|s| rhai::Dynamic::from(s.clone()))
        .collect();
    scope.push_constant("args", args_array);

    match engine.eval_ast_with_scope::<rhai::Dynamic>(&mut scope, &ast) {
        Ok(value) => {
            print::pretty_styled(&value, out_style);
            Ok(())
        }
        Err(e) => {
            trace::script_error_styled(&e, &source, path, err_style);
            std::process::exit(1);
        }
    }
}
