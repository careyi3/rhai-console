use std::fmt::Write;
use std::path::Path;

use rhai::{EvalAltResult, Position};

pub fn script_error(err: &EvalAltResult, source: &str, path: &Path) {
    eprintln!("{}", format_script_error(err, source, path));
}

pub fn repl_error(err: &EvalAltResult, source: &str) {
    eprintln!("{}", format_repl_error(err, source));
}

pub fn format_script_error(err: &EvalAltResult, source: &str, path: &Path) -> String {
    let mut out = format!("error in {}: {}", path.display(), leaf_message(err));
    if let Some(tb) = format_traceback(err, source, true) {
        out.push('\n');
        out.push_str(&tb);
    }
    out
}

pub fn format_repl_error(err: &EvalAltResult, source: &str) -> String {
    let mut out = format!("error: {}", leaf_message(err));
    if let Some(tb) = format_traceback(err, source, false) {
        out.push('\n');
        out.push_str(&tb);
    }
    out
}

struct Frame {
    label: String,
    position: Position,
}

fn collect_frames(err: &EvalAltResult) -> (Vec<Frame>, &EvalAltResult) {
    let mut frames = Vec::new();
    let mut current = err;
    loop {
        match current {
            EvalAltResult::ErrorInFunctionCall(name, src, inner, pos) => {
                let label = if src.is_empty() {
                    format!("{name}()")
                } else {
                    format!("{src}::{name}()")
                };
                frames.push(Frame {
                    label,
                    position: *pos,
                });
                current = inner;
            }
            _ => return (frames, current),
        }
    }
}

fn leaf_message(err: &EvalAltResult) -> String {
    let (_, leaf) = collect_frames(err);
    match leaf {
        EvalAltResult::ErrorRuntime(v, _) => format!("{v}"),
        other => other.to_string(),
    }
}

fn format_traceback(err: &EvalAltResult, source: &str, always: bool) -> Option<String> {
    let (mut frames, leaf) = collect_frames(err);
    let leaf_pos = leaf.position();
    frames.push(Frame {
        label: "<script>".into(),
        position: leaf_pos,
    });

    let any_pos = frames.iter().any(|f| !f.position.is_none());
    if !any_pos && !always {
        return None;
    }
    if frames.len() == 1 && frames[0].position.is_none() {
        return None;
    }

    let mut out = String::from("traceback (innermost last):\n");
    for (i, frame) in frames.iter().rev().enumerate() {
        let loc = match (frame.position.line(), frame.position.position()) {
            (Some(l), Some(c)) => format!("line {l}:{c}"),
            (Some(l), None) => format!("line {l}"),
            _ => "unknown".into(),
        };
        writeln!(out, "  in {} at {}", frame.label, loc).ok();
        if let Some(l) = frame.position.line() {
            if let Some(src_line) = source.lines().nth(l.saturating_sub(1)) {
                writeln!(out, "    | {}", src_line.trim_end()).ok();
                if let Some(c) = frame.position.position() {
                    if c > 0 {
                        writeln!(out, "    | {}^", " ".repeat(c - 1)).ok();
                    }
                }
            }
        }
        if i + 1 < frames.len() {
            // no separator between frames
        }
    }
    Some(out.trim_end().to_string())
}

