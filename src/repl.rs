use std::borrow::Cow;

use rhai::{Engine, EvalAltResult, ParseErrorType};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::{ColorMode, CompletionType, Config, Context, Editor, Helper, Hinter, Validator};

use crate::color::{ColorChoice, Stream, Style};
use crate::directives::{self, HELP, QUIT};
use crate::engine::Completions;
use crate::print;
use crate::trace;
use crate::{Error, Result};

#[derive(Helper, Hinter, Validator)]
struct ReplHelper {
    modules: Vec<(String, Vec<String>)>,
    globals: Vec<String>,
    vars: Vec<String>,
    style: Style,
}

impl Highlighter for ReplHelper {
    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: CompletionType,
    ) -> Cow<'c, str> {
        Cow::Owned(self.style.signature(candidate))
    }
}

impl ReplHelper {
    fn candidates(&self, line: &str, pos: usize) -> (usize, Vec<(String, String)>) {
        let start = line[..pos]
            .rfind(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':'))
            .map_or(0, |i| i + 1);
        let word = &line[start..pos];

        if let Some(rest) = word.strip_prefix(':') {
            let cands = directives::ALL
                .iter()
                .map(|d| d.name)
                .filter(|name| name.starts_with(rest))
                .map(|name| {
                    let full = format!(":{name}");
                    (full.clone(), full)
                })
                .collect();
            return (start, cands);
        }

        if let Some(colon) = word.find(':') {
            let ns = &word[..colon];
            let fn_prefix = word[colon..].trim_start_matches(':');
            let cands = self
                .modules
                .iter()
                .find(|(name, _)| name == ns)
                .map(|(_, sigs)| {
                    sigs.iter()
                        .filter(|sig| fn_name(sig).starts_with(fn_prefix))
                        .map(|sig| {
                            let display = format!("{ns}::{sig}");
                            let replacement = format!("{ns}::{}(", fn_name(sig));
                            (display, replacement)
                        })
                        .collect()
                })
                .unwrap_or_default();
            return (start, cands);
        }

        if word.is_empty() {
            if let Some(open) = unmatched_open_paren(&line[..start]) {
                let token = trailing_ident(&line[..open]);
                return (start, self.signatures_for(token));
            }
        }

        let mut cands = Vec::new();
        for (ns, _) in &self.modules {
            if ns.starts_with(word) {
                let r = format!("{ns}::");
                cands.push((r.clone(), r));
            }
        }
        for g in &self.globals {
            if fn_name(g).starts_with(word) {
                cands.push((g.clone(), g.clone()));
            }
        }
        for v in &self.vars {
            if v.starts_with(word) {
                cands.push((v.clone(), v.clone()));
            }
        }
        (start, cands)
    }
}

impl ReplHelper {
    fn signatures_for(&self, token: &str) -> Vec<(String, String)> {
        if let Some(idx) = token.rfind("::") {
            let ns = &token[..idx];
            let name = &token[idx + 2..];
            self.modules
                .iter()
                .find(|(n, _)| n == ns)
                .map(|(_, sigs)| {
                    sigs.iter()
                        .filter(|sig| fn_name(sig) == name)
                        .map(|sig| (format!("{ns}::{sig}"), String::new()))
                        .collect()
                })
                .unwrap_or_default()
        } else {
            self.globals
                .iter()
                .filter(|g| fn_name(g) == token)
                .map(|g| (g.clone(), String::new()))
                .collect()
        }
    }
}

fn fn_name(sig: &str) -> &str {
    sig.split('(').next().unwrap_or(sig).trim()
}

fn unmatched_open_paren(head: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in head.char_indices().rev() {
        match c {
            ')' => depth += 1,
            '(' if depth == 0 => return Some(i),
            '(' => depth -= 1,
            _ => {}
        }
    }
    None
}

fn trailing_ident(s: &str) -> &str {
    let start = s
        .rfind(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':'))
        .map_or(0, |i| i + 1);
    &s[start..]
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let (start, cands) = self.candidates(line, pos);
        let pairs = cands
            .into_iter()
            .map(|(display, replacement)| Pair {
                display,
                replacement,
            })
            .collect();
        Ok((start, pairs))
    }
}

pub(crate) fn run(
    engine: &Engine,
    intro: Option<&str>,
    help: &str,
    completions: &Completions,
    color: ColorChoice,
) -> Result<()> {
    let out_style = color.style_for(Stream::Stdout);
    let err_style = color.style_for(Stream::Stderr);

    if let Some(s) = intro {
        println!("{s}");
        if !s.ends_with('\n') {
            println!();
        }
    }
    println!(
        "{}",
        out_style.dim(&format!(
            "Type `:{}` for commands, `:{}` to exit.\n",
            HELP.name, QUIT.name
        ))
    );

    let helper = ReplHelper {
        modules: completions
            .modules
            .iter()
            .map(|(ns, fns)| (ns.to_string(), fns.clone()))
            .collect(),
        globals: completions.globals.clone(),
        vars: Vec::new(),
        style: out_style,
    };
    let color_mode = match color {
        ColorChoice::Always => ColorMode::Forced,
        ColorChoice::Never => ColorMode::Disabled,
        ColorChoice::Auto => ColorMode::Enabled,
    };
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .color_mode(color_mode)
        .build();
    let mut rl: Editor<ReplHelper, _> =
        Editor::with_config(config).map_err(|e| Error::Readline(e.to_string()))?;
    rl.set_helper(Some(helper));
    let mut scope = rhai::Scope::new();
    let mut buffer = String::new();

    loop {
        let prompt = out_style.prompt(if buffer.is_empty() {
            "rhai> "
        } else {
            "  ... "
        });
        match rl.readline(&prompt) {
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
                        print::pretty_styled(&value, out_style);
                        buffer.clear();
                        if let Some(h) = rl.helper_mut() {
                            h.vars = scope.iter().map(|(name, _, _)| name.to_string()).collect();
                        }
                    }
                    Err(e) if is_incomplete(&e) => {}
                    Err(e) => {
                        trace::repl_error_styled(&e, &buffer, err_style);
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

    fn helper() -> ReplHelper {
        ReplHelper {
            modules: vec![
                (
                    "demo".to_string(),
                    vec!["get(id: i64)".to_string(), "list()".to_string()],
                ),
                ("orders".to_string(), vec!["find(sku: String)".to_string()]),
                (
                    "math".to_string(),
                    vec![
                        "abs(n: i64)".to_string(),
                        "add(a: f64, b: f64)".to_string(),
                        "add(a: i64, b: i64)".to_string(),
                    ],
                ),
            ],
            globals: vec!["help()".to_string()],
            vars: vec!["total".to_string(), "tax".to_string()],
            style: Style::none(),
        }
    }

    fn complete(line: &str) -> Vec<String> {
        let (_, cands) = helper().candidates(line, line.len());
        cands.into_iter().map(|(_, r)| r).collect()
    }

    fn display(line: &str) -> Vec<String> {
        let (_, cands) = helper().candidates(line, line.len());
        cands.into_iter().map(|(d, _)| d).collect()
    }

    #[test]
    fn completes_module_namespaces_with_trailing_separator() {
        assert_eq!(complete("de"), vec!["demo::"]);
    }

    #[test]
    fn completes_functions_inserting_call_and_showing_params() {
        assert_eq!(complete("demo::"), vec!["demo::get(", "demo::list("]);
        assert_eq!(complete("demo::l"), vec!["demo::list("]);
        assert_eq!(
            display("demo::"),
            vec!["demo::get(id: i64)", "demo::list()"]
        );
    }

    #[test]
    fn single_colon_completes_namespace_functions() {
        assert_eq!(complete("demo:"), complete("demo::"));
        assert_eq!(complete("demo:"), vec!["demo::get(", "demo::list("]);
        assert_eq!(display("demo:l"), vec!["demo::list()"]);
    }

    #[test]
    fn shared_prefix_lists_every_matching_function() {
        assert_eq!(
            display("math::a"),
            vec![
                "math::abs(n: i64)",
                "math::add(a: f64, b: f64)",
                "math::add(a: i64, b: i64)",
            ]
        );
        assert_eq!(
            display("math::ad"),
            vec!["math::add(a: f64, b: f64)", "math::add(a: i64, b: i64)"]
        );
    }

    #[test]
    fn blank_line_offers_the_full_top_level_list() {
        assert_eq!(
            complete(""),
            vec!["demo::", "orders::", "math::", "help()", "total", "tax"]
        );
    }

    #[test]
    fn open_bracket_shows_matching_signatures() {
        assert_eq!(
            display("math::add("),
            vec!["math::add(a: f64, b: f64)", "math::add(a: i64, b: i64)"]
        );
        assert_eq!(complete("math::add("), vec!["", ""]);
        assert_eq!(
            display("math::add(1, "),
            vec!["math::add(a: f64, b: f64)", "math::add(a: i64, b: i64)"]
        );
        assert_eq!(display("help("), vec!["help()"]);
    }

    #[test]
    fn open_bracket_of_unknown_or_grouping_offers_nothing() {
        assert!(complete("nope::foo(").is_empty());
        assert!(complete("(").is_empty());
        assert!(complete("(1 + ").is_empty());
    }

    #[test]
    fn non_empty_word_inside_a_call_still_completes() {
        assert_eq!(complete("demo::get(t"), vec!["total", "tax"]);
        assert_eq!(complete("demo::get(de"), vec!["demo::"]);
    }

    #[test]
    fn unknown_namespace_yields_no_function_candidates() {
        assert!(complete("nope::").is_empty());
        assert!(complete("nope:").is_empty());
    }

    #[test]
    fn highlight_candidate_colors_name_and_params() {
        let mut h = helper();
        h.style = Style::new(true);
        let out = h.highlight_candidate("math::add(a: i64)", CompletionType::List);
        assert_eq!(out, "\x1b[36mmath::add\x1b[0m\x1b[2m(a: i64)\x1b[0m");
    }

    #[test]
    fn completes_scope_variables_and_globals_as_callable() {
        assert_eq!(complete("t"), vec!["total", "tax"]);
        assert_eq!(complete("he"), vec!["help()"]);
    }

    #[test]
    fn completes_directives_after_leading_colon() {
        assert_eq!(complete(":h"), vec![":help"]);
        assert_eq!(complete(":q"), vec![":quit"]);
    }

    #[test]
    fn completion_replaces_only_the_trailing_word() {
        let h = helper();
        let line = "let x = demo::g";
        let (start, cands) = h.candidates(line, line.len());
        assert_eq!(&line[start..], "demo::g");
        assert_eq!(
            cands,
            vec![("demo::get(id: i64)".to_string(), "demo::get(".to_string())]
        );
    }

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
