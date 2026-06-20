use rhai::{Engine, ImmutableString, Module, Position};
use rhai_console::{reg, wrap_result};

#[test]
fn wrap_result_serializes_ok_into_dynamic() {
    let r: Result<i64, String> = Ok(42);
    let value = wrap_result(r, Position::NONE).unwrap();
    assert_eq!(value.as_int().unwrap(), 42);
}

#[test]
fn wrap_result_maps_err_preserving_position() {
    let pos = Position::new(7, 3);
    let r: Result<i64, String> = Err("kaboom".into());
    let err = wrap_result(r, pos).unwrap_err();
    assert_eq!(err.position(), pos);
    assert!(format!("{err}").contains("kaboom"));
}

#[test]
fn reg_zero_arg_function() {
    let mut m = Module::new();
    let state = 7_i64;

    reg!(&mut m, state, "answer", |s| Ok::<_, String>(*s));

    let mut engine = Engine::new();
    engine.register_static_module("t", m.into());

    let result: i64 = engine.eval("t::answer()").unwrap();
    assert_eq!(result, 7);
}

#[test]
fn reg_multi_arg_function_returns_value() {
    let mut m = Module::new();
    let state = ();

    reg!(&mut m, state, "add", |_s, a: i64, b: i64| Ok::<_, String>(
        a + b
    ));

    let mut engine = Engine::new();
    engine.register_static_module("math", m.into());

    let result: i64 = engine.eval("math::add(3, 5)").unwrap();
    assert_eq!(result, 8);
}

#[test]
fn reg_propagates_domain_error_with_call_position() {
    let mut m = Module::new();
    let state = ();

    reg!(&mut m, state, "boom", |_s| Err::<i64, _>(
        "validation: bad input".to_string()
    ));

    let mut engine = Engine::new();
    engine.register_static_module("t", m.into());

    let err = engine.eval::<i64>("t::boom()").unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("validation: bad input"));
    assert!(!err.position().is_none(), "expected call-site position");
}

#[test]
fn reg_serializes_struct_via_serde() {
    use serde::Serialize;

    #[derive(Serialize, Clone)]
    struct Item {
        name: String,
        qty: i64,
    }

    let mut m = Module::new();
    let state = Item {
        name: "widget".into(),
        qty: 3,
    };

    reg!(&mut m, state, "get", |s| Ok::<_, String>(s.clone()));

    let mut engine = Engine::new();
    engine.register_static_module("t", m.into());

    let value: rhai::Map = engine.eval("t::get()").unwrap();
    assert_eq!(
        value.get("name").and_then(|v| v.clone().into_string().ok()),
        Some("widget".to_string())
    );
    assert_eq!(value.get("qty").and_then(|v| v.as_int().ok()), Some(3_i64));
}

#[test]
fn reg_attaches_param_metadata() {
    let mut m = Module::new();
    let state = ();

    reg!(&mut m, state, "find", |_s, sku: ImmutableString| Ok::<
        _,
        String,
    >(
        sku.len() as i64
    ));

    let sigs: Vec<String> = m.gen_fn_signatures_with_mapper(|t| t.into()).collect();
    let sig = sigs
        .iter()
        .find(|s| s.contains("find"))
        .expect("find present");
    assert!(sig.contains("sku: ImmutableString"), "got: {sig}");
}
