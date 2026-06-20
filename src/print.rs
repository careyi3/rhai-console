use rhai::Dynamic;

use crate::color::Style;

/// Print a value as pretty JSON to stdout. Unit (`()`) prints nothing.
pub fn pretty(value: &Dynamic) {
    if let Some(s) = format(value) {
        println!("{s}");
    }
}

pub(crate) fn pretty_styled(value: &Dynamic, style: Style) {
    if let Some(s) = format(value) {
        println!("{}", style.value(&s));
    }
}

/// Render a value as pretty JSON, or `None` for unit (`()`).
pub fn format(value: &Dynamic) -> Option<String> {
    if value.is_unit() {
        return None;
    }
    Some(serde_json::to_string_pretty(value).unwrap_or_else(|_| format!("{value:?}")))
}
