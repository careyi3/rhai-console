use std::path::Path;

use rhai::{EvalAltResult, Position};
use rhai_console::trace::{format_repl_error, format_script_error};

fn make_runtime(msg: &str, pos: Position) -> EvalAltResult {
    EvalAltResult::ErrorRuntime(msg.into(), pos)
}

fn wrap_call(name: &str, src: &str, inner: EvalAltResult, pos: Position) -> EvalAltResult {
    EvalAltResult::ErrorInFunctionCall(name.into(), src.into(), Box::new(inner), pos)
}

#[test]
fn leaf_only_no_position_skips_traceback() {
    let err = make_runtime("boom", Position::NONE);
    let out = format_repl_error(&err, "let x = 1;");
    assert_eq!(out, "error: boom");
}

#[test]
fn leaf_with_position_shows_source_line() {
    let err = make_runtime("boom", Position::new(2, 5));
    let source = "let a = 1;\nlet b = 2;\nlet c = 3;";
    let out = format_repl_error(&err, source);
    assert!(out.starts_with("error: boom"));
    assert!(out.contains("traceback"));
    assert!(out.contains("line 2:5"));
    assert!(out.contains("let b = 2;"));
}

#[test]
fn walks_nested_function_call_chain() {
    let source = "\
fn lookup(sku) {
    products::find_by_sku(sku)
}

fn outer(sku) {
    let p = lookup(sku);
    p.name
}

outer(\"DOESNOTEXIST\");
";
    let inner = make_runtime("not found", Position::NONE);
    let middle = wrap_call("find_by_sku", "products", inner, Position::new(2, 15));
    let outer_call = wrap_call("lookup", "", middle, Position::new(6, 13));
    let top = wrap_call("outer", "", outer_call, Position::new(10, 1));

    let out = format_script_error(&top, source, Path::new("script.rhai"));

    assert!(out.starts_with("error in script.rhai: not found"));
    assert!(out.contains("in products::find_by_sku() at line 2:15"));
    assert!(out.contains("in lookup() at line 6:13"));
    assert!(out.contains("in outer() at line 10:1"));
    assert!(out.contains("products::find_by_sku(sku)"));
}

#[test]
fn extracts_message_from_runtime_dynamic() {
    let err = make_runtime("validation: bad input", Position::new(1, 1));
    let out = format_repl_error(&err, "x");
    assert!(out.starts_with("error: validation: bad input"));
}
