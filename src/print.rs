use rhai::Dynamic;

pub fn pretty(value: &Dynamic) {
    if let Some(s) = format(value) {
        println!("{s}");
    }
}

pub fn format(value: &Dynamic) -> Option<String> {
    if value.is_unit() {
        return None;
    }
    Some(serde_json::to_string_pretty(value).unwrap_or_else(|_| format!("{value:?}")))
}

