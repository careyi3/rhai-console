use std::io::Write;

use rhai_console::{Console, Error};

#[test]
fn missing_script_file_returns_io_error() {
    let r = Console::new(()).run_with_args(vec!["/this/path/does/not/exist/missing.rhai".into()]);
    assert!(matches!(r, Err(Error::Io(_))), "got: {r:?}");
}

#[test]
fn empty_script_runs_successfully() {
    let file = tempfile::Builder::new().suffix(".rhai").tempfile().unwrap();
    let path = file.path().to_string_lossy().into_owned();

    let r = Console::new(()).run_with_args(vec![path]);
    assert!(r.is_ok());
}

#[test]
fn args_array_is_empty_when_no_args_given() {
    let mut file = tempfile::Builder::new().suffix(".rhai").tempfile().unwrap();
    writeln!(file, "if args.len != 0 {{ throw \"expected empty args\" }}").unwrap();
    let path = file.path().to_string_lossy().into_owned();

    let r = Console::new(()).run_with_args(vec![path]);
    assert!(r.is_ok());
}

#[test]
fn script_can_use_rhai_stdlib() {
    let mut file = tempfile::Builder::new().suffix(".rhai").tempfile().unwrap();
    writeln!(file, "let xs = [1, 2, 3];").unwrap();
    writeln!(file, "if xs.len != 3 {{ throw \"len wrong\" }}").unwrap();
    let path = file.path().to_string_lossy().into_owned();

    let r = Console::new(()).run_with_args(vec![path]);
    assert!(r.is_ok());
}
