use rhai::Dynamic;
use rhai_console::print::format;

#[test]
fn unit_returns_none() {
    assert!(format(&Dynamic::UNIT).is_none());
}

#[test]
fn integer_is_serialized() {
    let out = format(&Dynamic::from(42_i64)).unwrap();
    assert_eq!(out, "42");
}

#[test]
fn string_is_quoted() {
    let out = format(&Dynamic::from("hello".to_string())).unwrap();
    assert_eq!(out, "\"hello\"");
}
