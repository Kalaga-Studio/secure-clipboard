use pretty_assertions::assert_eq;
use secure_clipboard::config::RedactionConfig;
use secure_clipboard::redaction::RedactionEngine;

fn default_cfg() -> RedactionConfig {
    RedactionConfig {
        redact_email: true,
        redact_phone: true,
        redact_address: true,
        redact_urls_with_tokens: true,
        redact_account_numbers: true,
        redact_names_in_salutations: true,
        custom_names: vec![],
        allowlist_tokens: vec![],
    }
}

#[test]
fn redacts_sample_email_body() {
    let input = "Dear Fresno,\n\nI am reaching out to share my contact info for easier communication. Here's my phone +1 771-322-0123. Looking forward to meeting you.\n\nBest,\nAlberto";
    let engine = RedactionEngine::new(default_cfg()).unwrap();
    let result = engine.redact(input);

    assert!(result.changed);
    assert!(result.redacted_text.contains("Dear XXXXXX"));
    assert!(result.redacted_text.contains("+X XXX-XXX-XXXX"));
    assert!(result.redacted_text.contains("Best,\nXXXXXXX"));
}

#[test]
fn leaves_non_sensitive_text_unchanged() {
    let input = "Let's meet tomorrow to review architecture diagrams.";
    let engine = RedactionEngine::new(default_cfg()).unwrap();
    let result = engine.redact(input);

    assert!(!result.changed);
    assert_eq!(result.redacted_text, input);
}

#[test]
fn redacts_email_and_account_patterns() {
    let input = "Email me at test.user@example.com and use card 4111 1111 1111 1111.";
    let engine = RedactionEngine::new(default_cfg()).unwrap();
    let result = engine.redact(input);

    assert!(result.changed);
    assert!(result.redacted_text.contains("@example.com"));
    assert!(result.redacted_text.contains("XXXX XXXX XXXX XXXX"));
}

#[test]
fn supports_allowlist_override() {
    let mut cfg = default_cfg();
    cfg.custom_names = vec!["Acme".into()];
    cfg.allowlist_tokens = vec!["Acme".into()];
    let engine = RedactionEngine::new(cfg).unwrap();

    let result = engine.redact("Acme project update");
    assert_eq!(result.redacted_text, "Acme project update");
}
