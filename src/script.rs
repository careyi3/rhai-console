use std::path::Path;

use rhai::Engine;

use crate::print;
use crate::trace;
use crate::{Error, Result};

pub(crate) fn run(engine: &Engine, path: &Path, args: &[String]) -> Result<()> {
    let source = std::fs::read_to_string(path).map_err(Error::Io)?;

    let ast = match engine.compile(&source) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("parse error in {}: {e}", path.display());
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
            print::pretty(&value);
            Ok(())
        }
        Err(e) => {
            trace::script_error(&e, &source, path);
            std::process::exit(1);
        }
    }
}
