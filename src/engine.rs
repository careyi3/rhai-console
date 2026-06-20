use std::fmt::Write;

use rhai::{Engine, Module};

use crate::color::{ColorChoice, Stream, Style};
use crate::directives;

pub(crate) type ModuleBuilder<S> = Box<dyn FnOnce(&mut Module, S) + Send>;

#[derive(Default)]
pub(crate) struct Completions {
    pub modules: Vec<(&'static str, Vec<String>)>,
    pub globals: Vec<String>,
}

pub(crate) fn build<S>(
    state: S,
    builders: Vec<(&'static str, ModuleBuilder<S>)>,
    intro: Option<&str>,
    color: ColorChoice,
) -> (Engine, String, Completions)
where
    S: Clone + Send + Sync + 'static,
{
    let mut engine = Engine::new();
    engine.set_max_expr_depths(64, 32);
    engine.set_max_call_levels(64);
    engine.set_max_operations(1_000_000);

    let mut modules: Vec<(&'static str, Module)> = Vec::with_capacity(builders.len());
    for (name, builder) in builders {
        let mut m = Module::new();
        builder(&mut m, state.clone());
        modules.push((name, m));
    }

    let help = build_help(&modules, intro, color.style_for(Stream::Stdout));

    let completions = Completions {
        modules: modules
            .iter()
            .map(|(ns, m)| (*ns, module_signatures(m)))
            .collect(),
        globals: vec!["help()".to_string()],
    };

    for (name, module) in modules {
        engine.register_static_module(name, module.into());
    }

    let help_for_fn = help.clone();
    engine.register_fn("help", move || println!("{help_for_fn}"));

    (engine, help, completions)
}

fn module_signatures(module: &Module) -> Vec<String> {
    let mut sigs: Vec<String> = module
        .gen_fn_signatures_with_mapper(|t| t.into())
        .map(|sig| prettify(&sig))
        .collect();
    sigs.sort();
    sigs
}

fn format_directives_block(style: Style) -> String {
    let width = directives::ALL
        .iter()
        .map(|d| d.name.len())
        .max()
        .unwrap_or(0);
    let mut out = format!("{}\n", style.heading("REPL directives:"));
    for d in directives::ALL {
        let pad = " ".repeat(width.saturating_sub(d.name.len()));
        writeln!(
            out,
            "  {}{}  {}",
            style.ident(&format!(":{}", d.name)),
            pad,
            d.description
        )
        .ok();
    }
    out
}

fn build_help(modules: &[(&'static str, Module)], intro: Option<&str>, style: Style) -> String {
    let mut out = String::new();
    if let Some(s) = intro {
        out.push_str(s);
        if !s.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(&format_directives_block(style));
    writeln!(out, "\n{}", style.heading("Available modules:")).ok();
    let mut first = true;
    for (ns, module) in modules {
        let sigs = module_signatures(module);
        if sigs.is_empty() {
            continue;
        }
        if !first {
            out.push('\n');
        }
        for sig in sigs {
            writeln!(out, "  {}", style.signature(&format!("{ns}::{sig}"))).ok();
        }
        first = false;
    }
    out
}

fn prettify(sig: &str) -> String {
    let mut s = sig.trim().to_string();
    if let Some(stripped) = s.strip_suffix(" -> Dynamic") {
        s = stripped.to_string();
    }
    s.replace("rhai::", "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Module;

    #[test]
    fn prettify_strips_dynamic_return() {
        assert_eq!(prettify("list() -> Dynamic"), "list()");
        assert_eq!(prettify("find(id: i64) -> Dynamic"), "find(id: i64)");
    }

    #[test]
    fn prettify_strips_rhai_namespace_prefix() {
        assert_eq!(
            prettify("find(sku: rhai::ImmutableString) -> Dynamic"),
            "find(sku: ImmutableString)"
        );
    }

    #[test]
    fn prettify_leaves_unrelated_signatures_alone() {
        assert_eq!(prettify("foo(x: i64) -> i64"), "foo(x: i64) -> i64");
    }

    #[test]
    fn directives_block_starts_with_header_and_lists_each_directive() {
        let out = format_directives_block(Style::none());
        assert!(out.starts_with("REPL directives:\n"));
        assert!(out.contains(":help"));
        assert!(out.contains(":quit"));
        assert!(out.contains("show available commands"));
        assert!(out.contains("exit (or Ctrl-D)"));
    }

    #[test]
    fn directives_block_aligns_description_columns() {
        let out = format_directives_block(Style::none());
        let help_line = out.lines().find(|l| l.contains(":help")).unwrap();
        let quit_line = out.lines().find(|l| l.contains(":quit")).unwrap();
        let help_desc_col = help_line.find("show").unwrap();
        let quit_desc_col = quit_line.find("exit").unwrap();
        assert_eq!(help_desc_col, quit_desc_col);
    }

    #[test]
    fn build_help_includes_intro_when_provided() {
        let help = build_help(&[], Some("MY-APP intro"), Style::none());
        assert!(help.contains("MY-APP intro"));
        assert!(help.contains("REPL directives:"));
        assert!(help.contains("Available modules:"));
    }

    #[test]
    fn build_help_omits_intro_when_not_provided() {
        let help = build_help(&[], None, Style::none());
        assert!(help.starts_with("REPL directives:"));
    }

    #[test]
    fn build_help_lists_registered_modules() {
        let mut m = Module::new();
        rhai::FuncRegistration::new("answer")
            .with_params_info(["i64"])
            .set_into_module(&mut m, || -> Result<i64, Box<rhai::EvalAltResult>> {
                Ok(42)
            });
        let modules = vec![("ns", m)];
        let help = build_help(&modules, None, Style::none());
        assert!(help.contains("ns::answer"), "got: {help}");
    }

    #[test]
    fn build_works_end_to_end() {
        let builders: Vec<(&'static str, ModuleBuilder<i64>)> = vec![(
            "demo",
            Box::new(|m, state| {
                rhai::FuncRegistration::new("get")
                    .with_params_info(["i64"])
                    .set_into_module(m, move || -> Result<i64, Box<rhai::EvalAltResult>> {
                        Ok(state)
                    });
            }),
        )];

        let (engine, help, completions) = build(7_i64, builders, None, ColorChoice::Never);
        let result: i64 = engine.eval("demo::get()").unwrap();
        assert_eq!(result, 7);
        assert!(help.contains("demo::get"));
        let (ns, sigs) = &completions.modules[0];
        assert_eq!(*ns, "demo");
        assert_eq!(sigs.len(), 1);
        assert!(sigs[0].starts_with("get("), "got: {sigs:?}");
        assert!(completions.globals.contains(&"help()".to_string()));
    }

    #[test]
    fn module_signatures_are_sorted_and_keep_overloads() {
        let mut m = Module::new();
        rhai::FuncRegistration::new("list")
            .with_params_info(["i64"])
            .set_into_module(&mut m, || -> Result<i64, Box<rhai::EvalAltResult>> {
                Ok(0)
            });
        rhai::FuncRegistration::new("get")
            .with_params_info(["i64", "i64"])
            .set_into_module(&mut m, |_: i64| -> Result<i64, Box<rhai::EvalAltResult>> {
                Ok(0)
            });
        rhai::FuncRegistration::new("get")
            .with_params_info(["rhai::ImmutableString", "i64"])
            .set_into_module(
                &mut m,
                |_: rhai::ImmutableString| -> Result<i64, Box<rhai::EvalAltResult>> { Ok(0) },
            );

        let sigs = module_signatures(&m);
        assert_eq!(sigs.len(), 3, "got: {sigs:?}");
        let mut sorted = sigs.clone();
        sorted.sort();
        assert_eq!(sigs, sorted);
        assert!(
            sigs.iter().any(|s| s.contains("ImmutableString")),
            "got: {sigs:?}"
        );
        assert!(!sigs.iter().any(|s| s.contains("rhai::")), "got: {sigs:?}");
    }
}
