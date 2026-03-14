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

pub fn mask_alphanumeric(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_alphanumeric() { 'X' } else { ch })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_email_preserves_domain() {
        assert_eq!(mask_email("john@example.com"), "XXXX@example.com");
        assert_eq!(mask_email("ab@foo.org"), "XXX@foo.org");
    }

    #[test]
    fn mask_email_empty_local() {
        assert_eq!(mask_email("@domain.com"), "XXX@domain.com");
    }

    #[test]
    fn mask_email_no_at_sign() {
        let result = mask_email("notanemail");
        assert_eq!(result, "XXXXXXXXXX@");
    }

    #[test]
    fn mask_phone_replaces_digits() {
        assert_eq!(mask_phone("+1 771-322-0123"), "+X XXX-XXX-XXXX");
        assert_eq!(mask_phone("(555) 123-4567"), "(XXX) XXX-XXXX");
    }

    #[test]
    fn mask_phone_no_digits() {
        assert_eq!(mask_phone("abc-def"), "abc-def");
    }

    #[test]
    fn mask_phone_dotted_format() {
        assert_eq!(mask_phone("555.123.4567"), "XXX.XXX.XXXX");
    }

    #[test]
    fn mask_account_like_replaces_digits() {
        assert_eq!(mask_account_like("1234567890"), "XXXXXXXXXX");
        assert_eq!(
            mask_account_like("4111 1111 1111 1111"),
            "XXXX XXXX XXXX XXXX"
        );
    }

    #[test]
    fn mask_account_like_preserves_separators() {
        assert_eq!(mask_account_like("12-34"), "XX-XX");
    }

    #[test]
    fn mask_account_like_single_digit() {
        assert_eq!(mask_account_like("5"), "X");
    }

    #[test]
    fn mask_address_multiple_words() {
        let result = mask_address("123 Main Street Apt 4B");
        assert_eq!(result, "XXX XXXX XXXX XXXX XXXX");
    }

    #[test]
    fn mask_address_empty_string() {
        assert_eq!(mask_address(""), "XXXX");
    }

    #[test]
    fn mask_address_numeric_only_word() {
        assert_eq!(mask_address("456"), "XXX");
    }

    #[test]
    fn mask_address_single_word() {
        assert_eq!(mask_address("Broadway"), "XXXX");
    }

    #[test]
    fn mask_url_keep_host_with_query() {
        let result = mask_url_keep_host("https://api.example.com/v1?token=abc123&other=val");
        assert_eq!(result, "https://api.example.com/v1?redacted=XXXX");
    }

    #[test]
    fn mask_url_keep_host_no_query_string() {
        let result = mask_url_keep_host("https://example.com/path");
        assert_eq!(result, "https://redacted.local/?redacted=XXXX");
    }

    #[test]
    fn mask_url_keep_host_with_fragment() {
        let result = mask_url_keep_host("https://example.com/page?token=abc#section");
        assert_eq!(result, "https://example.com/page?redacted=XXXX");
    }

    #[test]
    fn mask_name_replaces_all_chars() {
        assert_eq!(mask_name("John"), "XXXX");
        assert_eq!(mask_name("Jane Doe"), "XXXXXXXX");
    }

    #[test]
    fn mask_name_empty_string() {
        assert_eq!(mask_name(""), "");
    }

    #[test]
    fn mask_name_unicode() {
        assert_eq!(mask_name("José"), "XXXX");
    }

    #[test]
    fn mask_labeled_field_preserves_key() {
        assert_eq!(mask_labeled_field("Password: secret123"), "Password: XXXX");
        assert_eq!(mask_labeled_field("CVV:999"), "CVV:XXXX");
    }

    #[test]
    fn mask_labeled_field_no_colon() {
        assert_eq!(mask_labeled_field("nokey"), "XXXX");
    }

    #[test]
    fn mask_labeled_field_colon_with_extra_spaces() {
        assert_eq!(mask_labeled_field("Token:   abc"), "Token:   XXXX");
    }

    #[test]
    fn mask_labeled_field_equals_separator() {
        assert_eq!(mask_labeled_field("Key: value"), "Key: XXXX");
    }

    #[test]
    fn mask_alphanumeric_preserves_spaces_and_dashes() {
        assert_eq!(mask_alphanumeric("GB29 NWBK 6016"), "XXXX XXXX XXXX");
        assert_eq!(mask_alphanumeric("AB-123"), "XX-XXX");
    }

    #[test]
    fn mask_alphanumeric_empty() {
        assert_eq!(mask_alphanumeric(""), "");
    }
}
