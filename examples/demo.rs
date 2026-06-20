//! Minimal playground for poking at the REPL locally.
//!
//! Run it with:
//!
//! ```sh
//! cargo run --example demo
//! ```
//!
//! Then try tab completion: type `ma<Tab>` to get `math::`, `math::<Tab>` to
//! list its functions, `let x = 41` then `x<Tab>`, or `:<Tab>` for directives.

use rhai_console::prelude::*;
use rhai_console::reg;

/// Trivial application state, just to show how it threads into a module.
#[derive(Clone)]
struct App {
    name: String,
}

fn main() -> rhai_console::Result<()> {
    let app = App {
        name: "demo".to_string(),
    };

    Console::new(app)
        .intro("rhai-console demo \u{2014} a tiny playground")
        .module("math", |m: &mut Module, s: App| {
            reg!(m, s, "add", |_s, a: i64, b: i64| Ok::<_, String>(a + b));
            reg!(m, s, "add", |_s, a: f64, b: f64| Ok::<_, String>(a + b));
            reg!(m, s, "abs", |_s, n: i64| Ok::<_, String>(n.abs()));
            reg!(m, s, "double", |_s, n: i64| Ok::<_, String>(n * 2));
        })
        .module("text", |m: &mut Module, s: App| {
            reg!(m, s, "shout", |_s, t: String| Ok::<_, String>(
                t.to_uppercase()
            ));
            reg!(m, s, "length", |_s, t: String| Ok::<_, String>(
                t.len() as i64
            ));
        })
        .module("app", |m: &mut Module, s: App| {
            reg!(m, s, "name", |s| Ok::<_, String>(s.name.clone()));
        })
        .run()
}
