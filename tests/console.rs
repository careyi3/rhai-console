use std::io::Write;

use rhai_console::prelude::*;
use rhai_console::reg;

#[test]
fn help_flag_short_returns_ok() {
    let r = Console::new(()).run_with_args(vec!["-h".into()]);
    assert!(r.is_ok());
}

#[test]
fn help_flag_long_returns_ok() {
    let r = Console::new(()).run_with_args(vec!["--help".into()]);
    assert!(r.is_ok());
}

#[test]
fn builder_chain_compiles_and_runs() {
    let r = Console::new(())
        .intro("test banner")
        .module("greet", |m: &mut Module, s: ()| {
            reg!(m, s, "hello", |_s| Ok::<_, String>("hi".to_string()));
        })
        .run_with_args(vec!["--help".into()]);
    assert!(r.is_ok());
}

#[test]
fn runs_script_file_happy_path() {
    let mut file = tempfile::Builder::new()
        .suffix(".rhai")
        .tempfile()
        .unwrap();
    writeln!(file, "1 + 2").unwrap();
    let path = file.path().to_string_lossy().into_owned();

    let r = Console::new(()).run_with_args(vec![path]);
    assert!(r.is_ok());
}

#[test]
fn script_can_access_registered_module() {
    let mut file = tempfile::Builder::new()
        .suffix(".rhai")
        .tempfile()
        .unwrap();
    writeln!(file, "math::double(21)").unwrap();
    let path = file.path().to_string_lossy().into_owned();

    let r = Console::new(())
        .module("math", |m: &mut Module, s: ()| {
            reg!(m, s, "double", |_s, n: i64| Ok::<_, String>(n * 2));
        })
        .run_with_args(vec![path]);
    assert!(r.is_ok());
}

#[test]
fn script_receives_positional_args() {
    let mut file = tempfile::Builder::new()
        .suffix(".rhai")
        .tempfile()
        .unwrap();
    writeln!(file, "assert::ok(args.len == 2);").unwrap();
    writeln!(file, "assert::ok(args[0] == \"alpha\");").unwrap();
    writeln!(file, "assert::ok(args[1] == \"beta\");").unwrap();
    let path = file.path().to_string_lossy().into_owned();

    let r = Console::new(())
        .module("assert", |m: &mut Module, s: ()| {
            reg!(m, s, "ok", |_s, cond: bool| {
                if cond {
                    Ok::<_, String>(())
                } else {
                    Err("assertion failed".into())
                }
            });
        })
        .run_with_args(vec![path, "alpha".into(), "beta".into()]);
    assert!(r.is_ok(), "args weren't injected correctly: {r:?}");
}
