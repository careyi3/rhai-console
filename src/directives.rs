#[derive(Debug, Clone, Copy)]
pub(crate) struct Directive {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
}

impl Directive {
    pub fn matches(&self, input: &str) -> bool {
        input == self.name || self.aliases.contains(&input)
    }
}

pub(crate) const HELP: Directive = Directive {
    name: "help",
    aliases: &["h"],
    description: "show available commands",
};

pub(crate) const QUIT: Directive = Directive {
    name: "quit",
    aliases: &["q", "exit"],
    description: "exit (or Ctrl-D)",
};

pub(crate) const ALL: &[Directive] = &[HELP, QUIT];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_canonical_name() {
        assert!(HELP.matches("help"));
        assert!(QUIT.matches("quit"));
    }

    #[test]
    fn matches_aliases() {
        assert!(HELP.matches("h"));
        assert!(QUIT.matches("q"));
        assert!(QUIT.matches("exit"));
    }

    #[test]
    fn does_not_match_unrelated() {
        assert!(!HELP.matches("quit"));
        assert!(!QUIT.matches("help"));
        assert!(!HELP.matches(""));
        assert!(!HELP.matches("HELP"));
    }

    #[test]
    fn all_lists_each_directive_once() {
        assert_eq!(ALL.len(), 2);
        assert!(ALL.iter().any(|d| d.name == "help"));
        assert!(ALL.iter().any(|d| d.name == "quit"));
    }
}
