//! Operational REPL and script runner for embedding Rhai in Rust applications.
//!
//! Configure your application state and a few module registrations and you get back a CLI
//! with an interactive REPL and a `.rhai` script runner, both running in your application's
//! context. Start from [`Console`].
//!
//! ```no_run
//! use rhai_console::prelude::*;
//!
//! #[derive(Clone)]
//! struct Services;
//!
//! fn main() -> rhai_console::Result<()> {
//!     Console::new(Services)
//!         .intro("my-app REPL")
//!         .module("math", |m: &mut Module, svc: Services| {
//!             reg!(m, svc, "add", |_s, a: i64, b: i64| Ok::<_, String>(a + b));
//!         })
//!         .run()
//! }
//! ```

mod cli;
mod color;
mod console;
mod directives;
mod engine;
mod error;
mod macros;
/// Formatting of REPL and script result values.
pub mod print;
mod repl;
mod script;
/// Formatting of runtime errors and their tracebacks.
pub mod trace;

pub use color::ColorChoice;
pub use console::Console;
pub use error::{Error, Result};
pub use macros::{runtime_err, wrap_result};

/// Re-export of the `rhai` crate this was built against, so consumers can match versions.
pub use rhai;

/// Everything you need to set up a [`Console`] and register modules.
pub mod prelude {
    pub use crate::reg;
    pub use crate::ColorChoice;
    pub use crate::Console;
    pub use rhai::{
        Array, Dynamic, EvalAltResult, ImmutableString, Map, Module, NativeCallContext, Scope,
    };
}
