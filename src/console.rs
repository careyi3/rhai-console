use rhai::Module;

use crate::engine::{self, ModuleBuilder};
use crate::{cli, ColorChoice, Result};

/// Builder for an embedded Rhai console.
///
/// Register modules against your application state, then [`run`](Console::run) the REPL or a
/// script. `S` is your state type; it is cloned into each module so functions can reach it.
pub struct Console<S>
where
    S: Clone + Send + Sync + 'static,
{
    state: S,
    builders: Vec<(&'static str, ModuleBuilder<S>)>,
    intro: Option<String>,
    color: ColorChoice,
}

impl<S> Console<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Start a console backed by `state`.
    pub fn new(state: S) -> Self {
        Self {
            state,
            builders: Vec::new(),
            intro: None,
            color: ColorChoice::default(),
        }
    }

    /// Register a module under `name`; `builder` populates it from your state. Its functions
    /// are then callable as `name::fn(...)` in the REPL and scripts.
    pub fn module<F>(mut self, name: &'static str, builder: F) -> Self
    where
        F: FnOnce(&mut Module, S) + Send + 'static,
    {
        self.builders.push((name, Box::new(builder)));
        self
    }

    /// Set a banner shown at REPL startup and atop the `:help` listing.
    pub fn intro(mut self, intro: impl Into<String>) -> Self {
        self.intro = Some(intro.into());
        self
    }

    /// Override color output. Defaults to [`ColorChoice::Auto`].
    pub fn color(mut self, color: ColorChoice) -> Self {
        self.color = color;
        self
    }

    /// Dispatch on `std::env::args()`: no arguments starts the REPL, otherwise the first
    /// argument is run as a script path and the rest are passed to it as `args`.
    pub fn run(self) -> Result<()> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        self.run_with_args(args)
    }

    /// Like [`run`](Console::run), but with explicit arguments (the program name is not included).
    pub fn run_with_args(self, args: Vec<String>) -> Result<()> {
        let (engine, help, completions) =
            engine::build(self.state, self.builders, self.intro.as_deref(), self.color);
        cli::dispatch(
            &engine,
            self.intro.as_deref(),
            &help,
            &completions,
            self.color,
            args,
        )
    }
}
