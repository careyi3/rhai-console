use std::fmt::Display;

use rhai::{Dynamic, EvalAltResult, Position};
use serde::Serialize;

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

pub fn runtime_err(pos: Position, msg: impl Into<String>) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(msg.into().into(), pos))
}

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
