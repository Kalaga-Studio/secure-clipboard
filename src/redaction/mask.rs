pub fn mask_email(value: &str) -> String {
    let mut parts = value.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();
    format!("{}@{}", "X".repeat(local.chars().count().max(3)), domain)
}

pub fn mask_phone(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_digit() { 'X' } else { ch })
        .collect()
}

pub fn mask_account_like(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_digit() { 'X' } else { ch })
        .collect()
}

pub fn mask_address(value: &str) -> String {
    let words: Vec<String> = value
        .split_whitespace()
        .map(|w| {
            if w.chars().all(|c| c.is_ascii_digit()) {
                "X".repeat(w.len())
            } else {
                "XXXX".to_string()
            }
        })
        .collect();
    if words.is_empty() {
        return "XXXX".into();
    }
    words.join(" ")
}

pub fn mask_url_keep_host(value: &str) -> String {
    if let Some((prefix, _)) = value.split_once('?') {
        return format!("{}?redacted=XXXX", prefix);
    }
    "https://redacted.local/?redacted=XXXX".into()
}

pub fn mask_name(value: &str) -> String {
    value.chars().map(|_| 'X').collect()
}

// Preserve the label key ("Password:") and mask only the value after the colon.
pub fn mask_labeled_field(value: &str) -> String {
    if let Some(pos) = value.find(':') {
        let key = &value[..=pos];
        let rest = &value[pos + 1..];
        let leading_space = rest.len() - rest.trim_start().len();
        let spaces = &rest[..leading_space];
        format!("{}{}XXXX", key, spaces)
    } else {
        "XXXX".into()
    }
}
