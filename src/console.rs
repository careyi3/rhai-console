use rhai::Module;

use crate::engine::{self, ModuleBuilder};
use crate::{cli, Result};

pub struct Console<S>
where
    S: Clone + Send + Sync + 'static,
{
    state: S,
    builders: Vec<(&'static str, ModuleBuilder<S>)>,
    intro: Option<String>,
}

impl<S> Console<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(state: S) -> Self {
        Self {
            state,
            builders: Vec::new(),
            intro: None,
        }
    }

    pub fn module<F>(mut self, name: &'static str, builder: F) -> Self
    where
        F: FnOnce(&mut Module, S) + Send + 'static,
    {
        self.builders.push((name, Box::new(builder)));
        self
    }

    pub fn intro(mut self, intro: impl Into<String>) -> Self {
        self.intro = Some(intro.into());
        self
    }

    pub fn run(self) -> Result<()> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        self.run_with_args(args)
    }

    pub fn run_with_args(self, args: Vec<String>) -> Result<()> {
        let (engine, help) = engine::build(self.state, self.builders, self.intro.as_deref());
        cli::dispatch(&engine, self.intro.as_deref(), &help, args)
    }
}
