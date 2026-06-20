use std::fmt::Display;

use rhai::{Dynamic, EvalAltResult, Position};
use serde::Serialize;

/// Convert a `Result<T, E>` into a Rhai call result: `Ok` is serialized to a [`Dynamic`] via
/// serde, `Err` becomes a runtime error at `pos`. Used by [`reg!`](crate::reg).
pub fn wrap_result<T, E>(r: Result<T, E>, pos: Position) -> Result<Dynamic, Box<EvalAltResult>>
where
    T: Serialize,
    E: Display,
{
    match r {
        Ok(v) => rhai::serde::to_dynamic(&v).map_err(|e| runtime_err(pos, format!("serde: {e}"))),
        Err(e) => Err(runtime_err(pos, e.to_string())),
    }
}

/// Build a Rhai runtime error carrying `msg` at source position `pos`.
pub fn runtime_err(pos: Position, msg: impl Into<String>) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(msg.into().into(), pos))
}

/// Register a function on a module, wrapping the call with source-position capture, error
/// mapping, and serde conversion of the result into a Rhai value.
///
/// The first closure parameter binds your state; the rest are the Rhai arguments. The body
/// returns a `Result<T, E>` where `T: Serialize` and `E: Display`; `?` is supported.
///
/// ```ignore
/// reg!(m, svc, "find", |s, id: i64| s.products().find(id));
/// ```
#[macro_export]
macro_rules! reg {
    ($m:expr, $svc:expr, $name:expr, |$s:ident $(, $a:ident: $t:ty)* $(,)?| $body:expr) => {{
        let svc = $svc.clone();
        ::rhai::FuncRegistration::new($name)
            .with_params_info([
                $( concat!(stringify!($a), ": ", stringify!($t)), )*
                "Dynamic",
            ])
            .set_into_module(
                $m,
                move |__ctx: ::rhai::NativeCallContext $(, $a: $t)*|
                    -> ::std::result::Result<::rhai::Dynamic, ::std::boxed::Box<::rhai::EvalAltResult>>
                {
                    let pos = __ctx.call_position();
                    let $s = &svc;
                    let __result = (|| { $body })();
                    $crate::wrap_result(__result, pos)
                },
            );
    }};
}
