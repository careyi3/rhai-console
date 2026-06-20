use std::io::IsTerminal;

/// Whether to colorize console output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorChoice {
    /// Colorize when the stream is a terminal, honoring `NO_COLOR` and `CLICOLOR_FORCE`. The default.
    #[default]
    Auto,
    /// Always colorize, even when output is piped or redirected.
    Always,
    /// Never colorize.
    Never,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Stream {
    Stdout,
    Stderr,
}

impl ColorChoice {
    pub(crate) fn style_for(self, stream: Stream) -> Style {
        Style::new(self.enabled_for(stream))
    }

    fn enabled_for(self, stream: Stream) -> bool {
        match self {
            ColorChoice::Always => true,
            ColorChoice::Never => false,
            ColorChoice::Auto => auto(stream),
        }
    }
}

fn auto(stream: Stream) -> bool {
    if no_color() {
        return false;
    }
    if clicolor_force() {
        return true;
    }
    match stream {
        Stream::Stdout => std::io::stdout().is_terminal(),
        Stream::Stderr => std::io::stderr().is_terminal(),
    }
}

fn no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty())
}

fn clicolor_force() -> bool {
    std::env::var_os("CLICOLOR_FORCE").is_some_and(|v| !v.is_empty() && v != "0")
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Style {
    enabled: bool,
}

impl Style {
    pub(crate) fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub(crate) fn none() -> Self {
        Self::new(false)
    }

    fn paint(&self, code: &str, s: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_owned()
        }
    }

    pub(crate) fn prompt(&self, s: &str) -> String {
        self.paint("36", s)
    }

    pub(crate) fn value(&self, s: &str) -> String {
        self.paint("32", s)
    }

    pub(crate) fn error(&self, s: &str) -> String {
        self.paint("1;31", s)
    }

    pub(crate) fn dim(&self, s: &str) -> String {
        self.paint("2", s)
    }

    pub(crate) fn caret(&self, s: &str) -> String {
        self.paint("31", s)
    }

    pub(crate) fn ident(&self, s: &str) -> String {
        self.paint("36", s)
    }

    pub(crate) fn heading(&self, s: &str) -> String {
        self.paint("1", s)
    }

    pub(crate) fn signature(&self, s: &str) -> String {
        match s.split_once('(') {
            Some((name, rest)) => format!("{}{}", self.ident(name), self.dim(&format!("({rest}"))),
            None => self.ident(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_and_never_ignore_environment() {
        assert!(ColorChoice::Always.style_for(Stream::Stdout).enabled);
        assert!(!ColorChoice::Never.style_for(Stream::Stderr).enabled);
    }

    #[test]
    fn disabled_style_leaves_text_untouched() {
        let s = Style::none();
        assert_eq!(s.error("boom"), "boom");
        assert_eq!(s.prompt("rhai> "), "rhai> ");
    }

    #[test]
    fn enabled_style_wraps_with_ansi_codes() {
        let s = Style::new(true);
        assert_eq!(s.error("boom"), "\x1b[1;31mboom\x1b[0m");
        assert_eq!(s.value("42"), "\x1b[32m42\x1b[0m");
    }

    #[test]
    fn signature_colors_name_and_params_separately() {
        let s = Style::new(true);
        assert_eq!(
            s.signature("add(a: i64)"),
            "\x1b[36madd\x1b[0m\x1b[2m(a: i64)\x1b[0m"
        );
        assert_eq!(s.signature("demo::"), "\x1b[36mdemo::\x1b[0m");
    }

    #[test]
    fn disabled_signature_is_unchanged() {
        let s = Style::none();
        assert_eq!(s.signature("add(a: i64)"), "add(a: i64)");
        assert_eq!(s.signature("demo::"), "demo::");
    }
}
