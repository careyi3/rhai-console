use rhai::{Engine, EvalAltResult, ParseErrorType};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::directives::{HELP, QUIT};
use crate::print;
use crate::trace;
use crate::{Error, Result};

pub(crate) fn run(engine: &Engine, intro: Option<&str>, help: &str) -> Result<()> {
    if let Some(s) = intro {
        println!("{s}");
        if !s.ends_with('\n') {
            println!();
        }
    }
    println!("Type `:{}` for commands, `:{}` to exit.\n", HELP.name, QUIT.name);

    let mut rl = DefaultEditor::new().map_err(|e| Error::Readline(e.to_string()))?;
    let mut scope = rhai::Scope::new();
    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() { "rhai> " } else { "  ... " };
        match rl.readline(prompt) {
            Ok(line) => {
                if buffer.is_empty() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Some(rest) = trimmed.strip_prefix(':') {
                        handle_directive(rest, help);
                        continue;
                    }
                }
                let _ = rl.add_history_entry(&line);
                buffer.push_str(&line);
                buffer.push('\n');

                match engine.eval_with_scope::<rhai::Dynamic>(&mut scope, &buffer) {
                    Ok(value) => {
                        print::pretty(&value);
                        buffer.clear();
                    }
                    Err(e) if is_incomplete(&e) => {}
                    Err(e) => {
                        trace::repl_error(&e, &buffer);
                        buffer.clear();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                buffer.clear();
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                println!();
                break;
            }
            Err(e) => {
                return Err(Error::Readline(e.to_string()));
            }
        }
    }
    Ok(())
}

fn is_incomplete(e: &EvalAltResult) -> bool {
    matches!(
        e,
        EvalAltResult::ErrorParsing(ParseErrorType::UnexpectedEOF, _)
    )
}

fn handle_directive(s: &str, help: &str) {
    let trimmed = s.trim();
    if HELP.matches(trimmed) {
        println!("{help}");
    } else if QUIT.matches(trimmed) {
        std::process::exit(0);
    } else {
        eprintln!("unknown directive: :{trimmed}  (try :{})", HELP.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::{ParseErrorType, Position};

    #[test]
    fn is_incomplete_detects_unexpected_eof() {
        let err = EvalAltResult::ErrorParsing(ParseErrorType::UnexpectedEOF, Position::NONE);
        assert!(is_incomplete(&err));
    }

    #[test]
    fn is_incomplete_rejects_runtime_errors() {
        let err = EvalAltResult::ErrorRuntime("boom".into(), Position::NONE);
        assert!(!is_incomplete(&err));
    }

    #[test]
    fn is_incomplete_rejects_other_parse_errors() {
        let err = EvalAltResult::ErrorParsing(
            ParseErrorType::BadInput(rhai::LexError::UnterminatedString),
            Position::new(1, 1),
        );
        assert!(!is_incomplete(&err));
    }
}
