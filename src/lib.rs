mod cli;
mod console;
mod directives;
mod engine;
mod error;
mod macros;
pub mod print;
mod repl;
mod script;
pub mod trace;

pub use console::Console;
pub use error::{Error, Result};
pub use macros::{runtime_err, wrap_result};

pub use rhai;

pub mod prelude {
    pub use crate::reg;
    pub use crate::Console;
    pub use rhai::{
        Array, Dynamic, EvalAltResult, ImmutableString, Map, Module, NativeCallContext, Scope,
    };
}
