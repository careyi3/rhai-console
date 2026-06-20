use std::path::Path;

use rhai::Engine;

use crate::engine::Completions;
use crate::{repl, script, ColorChoice, Result};

pub(crate) fn dispatch(
    engine: &Engine,
    intro: Option<&str>,
    help: &str,
    completions: &Completions,
    color: ColorChoice,
    args: Vec<String>,
) -> Result<()> {
    if matches!(args.first().map(String::as_str), Some("-h" | "--help")) {
        print_usage();
        return Ok(());
    }

    match args.split_first() {
        None => repl::run(engine, intro, help, completions, color),
        Some((path, script_args)) => script::run(engine, Path::new(path), script_args, color),
    }
}

fn print_usage() {
    let prog = std::env::args()
        .next()
        .and_then(|p| {
            Path::new(&p)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "console".into());

    println!(
        "{prog} \u{2014} Rhai REPL and script runner

Usage:
  {prog}                              start an interactive REPL
  {prog} <path.rhai> [args...]        execute a script file and exit
  {prog} -h | --help                  show this help

Inside a script, arguments are available as an `args` array of strings:
  let a = args[0];
  let b = parse_int(args[1]);
"
    );
}
