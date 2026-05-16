use rhai_console::Error;

#[test]
fn io_error_converts_via_from() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing file");
    let err: Error = io.into();
    assert!(matches!(err, Error::Io(_)));
    assert!(format!("{err}").contains("missing file"));
}

#[test]
fn script_variant_displays_with_prefix() {
    let err = Error::Script("ran out of fuel".into());
    assert_eq!(format!("{err}"), "script: ran out of fuel");
}

#[test]
fn readline_variant_displays_with_prefix() {
    let err = Error::Readline("eof".into());
    assert_eq!(format!("{err}"), "readline: eof");
}
