use secure_clipboard::config::RedactionConfig;
use secure_clipboard::redaction::RedactionEngine;

fn default_cfg() -> RedactionConfig {
    RedactionConfig {
        redact_email: true,
        redact_phone: true,
        redact_address: true,
        redact_urls_with_tokens: true,
        redact_account_numbers: true,
        redact_ssn: true,
        redact_labeled_fields: true,
        redact_dates: true,
        redact_names_in_salutations: true,
        redact_ip_addresses: true,
        redact_iban: true,
        redact_passport: true,
        custom_names: vec![],
        allowlist_tokens: vec![],
    }
}

fn only(mut cfg: RedactionConfig) -> RedactionConfig {
    cfg.redact_email = false;
    cfg.redact_phone = false;
    cfg.redact_address = false;
    cfg.redact_urls_with_tokens = false;
    cfg.redact_account_numbers = false;
    cfg.redact_ssn = false;
    cfg.redact_labeled_fields = false;
    cfg.redact_dates = false;
    cfg.redact_names_in_salutations = false;
    cfg.redact_ip_addresses = false;
    cfg.redact_iban = false;
    cfg.redact_passport = false;
    cfg
}

// --- Email ---

#[test]
fn email_simple() {
    let engine = RedactionEngine::new(default_cfg()).unwrap();
    let r = engine.redact("Contact us at user@example.com please.");
    assert!(r.changed);
    assert!(r.redacted_text.contains("@example.com"));
    assert!(!r.redacted_text.contains("user@"));
}

#[test]
fn email_with_dots_and_plus() {
    let engine = RedactionEngine::new(default_cfg()).unwrap();
    let r = engine.redact("Send to first.last+tag@sub.domain.co.uk");
    assert!(r.changed);
    assert!(r.redacted_text.contains("@sub.domain.co.uk"));
}

#[test]
fn email_multiple_in_text() {
    let engine = RedactionEngine::new(default_cfg()).unwrap();
    let r = engine.redact("CC: a@b.com and c@d.com");
    assert!(r.changed);
    assert_eq!(r.matches.len(), 2);
}

// --- Phone ---

#[test]
fn phone_us_dashed() {
    let mut cfg = only(default_cfg());
    cfg.redact_phone = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Call 555-123-4567 now");
    assert!(r.changed);
    assert!(r.redacted_text.contains("XXX-XXX-XXXX"));
}

#[test]
fn phone_us_parenthesized() {
    let mut cfg = only(default_cfg());
    cfg.redact_phone = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Call (555) 123-4567 now");
    assert!(r.changed);
    assert!(r.redacted_text.contains("(XXX) XXX-XXXX"));
}

#[test]
fn phone_international() {
    let mut cfg = only(default_cfg());
    cfg.redact_phone = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Reach me at +1 771-322-0123.");
    assert!(r.changed);
    assert!(!r.redacted_text.contains("771"));
}

#[test]
fn phone_dotted() {
    let mut cfg = only(default_cfg());
    cfg.redact_phone = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Fax: 555.123.4567");
    assert!(r.changed);
    assert!(r.redacted_text.contains("XXX.XXX.XXXX"));
}

// --- SSN ---

#[test]
fn ssn_standard() {
    let mut cfg = only(default_cfg());
    cfg.redact_ssn = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("SSN is 123-45-6789.");
    assert!(r.changed);
    assert!(r.redacted_text.contains("XXX-XX-XXXX"));
}

#[test]
fn ssn_with_spaces() {
    let mut cfg = only(default_cfg());
    cfg.redact_ssn = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("SSN: 123 45 6789");
    assert!(r.changed);
    assert!(r.redacted_text.contains("XXX XX XXXX"));
}

// --- Labeled fields ---

#[test]
fn labeled_password() {
    let mut cfg = only(default_cfg());
    cfg.redact_labeled_fields = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Password: hunter2");
    assert!(r.changed);
    assert!(r.redacted_text.contains("Password: XXXX"));
    assert!(!r.redacted_text.contains("hunter2"));
}

#[test]
fn labeled_cvv() {
    let mut cfg = only(default_cfg());
    cfg.redact_labeled_fields = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("CVV: 345");
    assert!(r.changed);
    assert!(r.redacted_text.contains("CVV: XXXX"));
}

#[test]
fn labeled_api_key() {
    let mut cfg = only(default_cfg());
    cfg.redact_labeled_fields = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("api_key: sk-abc123xyz");
    assert!(r.changed);
    assert!(!r.redacted_text.contains("sk-abc123xyz"));
}

#[test]
fn labeled_with_equals() {
    let mut cfg = only(default_cfg());
    cfg.redact_labeled_fields = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("token=eyJhbGciOiJ");
    assert!(r.changed);
    assert!(!r.redacted_text.contains("eyJhbGciOiJ"));
}

// --- Dates ---

#[test]
fn date_us_format() {
    let mut cfg = only(default_cfg());
    cfg.redact_dates = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("DOB: 01/23/1985");
    assert!(r.changed);
    assert!(r.redacted_text.contains("XX/XX/XXXX"));
}

#[test]
fn date_iso_format() {
    let mut cfg = only(default_cfg());
    cfg.redact_dates = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Born 1985-01-23 in NYC");
    assert!(r.changed);
    assert!(r.redacted_text.contains("XXXX-XX-XX"));
}

#[test]
fn date_european_dot_format() {
    let mut cfg = only(default_cfg());
    cfg.redact_dates = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Date: 23.01.1985");
    assert!(r.changed);
    assert!(r.redacted_text.contains("XX.XX.XXXX"));
}

// --- URL tokens ---

#[test]
fn url_with_api_key() {
    let mut cfg = only(default_cfg());
    cfg.redact_urls_with_tokens = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Visit https://api.example.com/data?apikey=sk123abc&format=json");
    assert!(r.changed);
    assert!(r.redacted_text.contains("?redacted=XXXX"));
    assert!(!r.redacted_text.contains("sk123abc"));
}

#[test]
fn url_without_token_unchanged() {
    let mut cfg = only(default_cfg());
    cfg.redact_urls_with_tokens = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("See https://docs.rust-lang.org/book/");
    assert!(!r.changed);
}

// --- Addresses ---

#[test]
fn address_standard() {
    let mut cfg = only(default_cfg());
    cfg.redact_address = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Ship to 742 Evergreen Terrace Drive");
    assert!(r.changed);
    assert!(!r.redacted_text.contains("742"));
    assert!(!r.redacted_text.contains("Evergreen"));
}

// --- Names ---

#[test]
fn name_in_salutation() {
    let mut cfg = only(default_cfg());
    cfg.redact_names_in_salutations = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Dear Alice,\nThank you.");
    assert!(r.changed);
    assert!(!r.redacted_text.contains("Alice"));
}

#[test]
fn name_in_signoff() {
    let mut cfg = only(default_cfg());
    cfg.redact_names_in_salutations = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Thanks,\nBob Smith");
    assert!(r.changed);
    assert!(!r.redacted_text.contains("Bob"));
    assert!(!r.redacted_text.contains("Smith"));
}

#[test]
fn name_custom_name() {
    let mut cfg = only(default_cfg());
    cfg.custom_names = vec!["Voldemort".into()];
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Do not say Voldemort.");
    assert!(r.changed);
    assert!(!r.redacted_text.contains("Voldemort"));
}

// --- IP Addresses ---

#[test]
fn ip_v4_standard() {
    let mut cfg = only(default_cfg());
    cfg.redact_ip_addresses = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Server at 192.168.1.100 is down");
    assert!(r.changed);
    assert!(!r.redacted_text.contains("192"));
}

#[test]
fn ip_v4_boundary_values() {
    let mut cfg = only(default_cfg());
    cfg.redact_ip_addresses = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Range: 0.0.0.0 to 255.255.255.255");
    assert!(r.changed);
    assert_eq!(r.matches.len(), 2);
}

#[test]
fn ip_v4_not_version_string() {
    let mut cfg = only(default_cfg());
    cfg.redact_ip_addresses = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Running version 1.2.3");
    assert!(!r.changed);
}

// --- IBAN ---

#[test]
fn iban_german() {
    let mut cfg = only(default_cfg());
    cfg.redact_iban = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Pay to DE89370400440532013000");
    assert!(r.changed);
    assert!(!r.redacted_text.contains("3704"));
}

#[test]
fn iban_with_spaces() {
    let mut cfg = only(default_cfg());
    cfg.redact_iban = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("IBAN: GB29 NWBK 6016 1331 9268 19");
    assert!(r.changed);
    assert!(!r.redacted_text.contains("NWBK"));
    assert!(!r.redacted_text.contains("6016"));
}

// --- Passport ---

#[test]
fn passport_with_label() {
    let mut cfg = only(default_cfg());
    cfg.redact_passport = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Passport number: AB1234567");
    assert!(r.changed);
    assert!(!r.redacted_text.contains("AB1234567"));
}

#[test]
fn passport_no_label_unchanged() {
    let mut cfg = only(default_cfg());
    cfg.redact_passport = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Code AB1234567 is valid");
    assert!(!r.changed);
}

// --- Account Numbers ---

#[test]
fn credit_card_number() {
    let mut cfg = only(default_cfg());
    cfg.redact_account_numbers = true;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Card: 4111 1111 1111 1111");
    assert!(r.changed);
    assert!(r.redacted_text.contains("XXXX XXXX XXXX XXXX"));
}

// --- Allowlist ---

#[test]
fn allowlist_restores_token() {
    let mut cfg = default_cfg();
    cfg.custom_names = vec!["Acme".into()];
    cfg.allowlist_tokens = vec!["Acme".into()];
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("The Acme project ships tomorrow.");
    assert_eq!(r.redacted_text, "The Acme project ships tomorrow.");
}

#[test]
fn allowlist_empty_token_ignored() {
    let mut cfg = default_cfg();
    cfg.allowlist_tokens = vec!["".into()];
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Nothing special here.");
    assert!(!r.changed);
}

// --- Disabled rules ---

#[test]
fn disabled_email_leaves_email() {
    let mut cfg = default_cfg();
    cfg.redact_email = false;
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Email me at user@example.com");
    assert!(
        r.redacted_text.contains("user@example.com"),
        "email should remain when redact_email=false"
    );
}

#[test]
fn disabled_phone_leaves_phone() {
    let cfg = only(default_cfg());
    let engine = RedactionEngine::new(cfg).unwrap();
    let r = engine.redact("Call 555-123-4567");
    assert!(!r.changed);
}

// --- No sensitive data ---

#[test]
fn plain_text_no_change() {
    let engine = RedactionEngine::new(default_cfg()).unwrap();
    let r = engine.redact("The quick brown fox jumps over the lazy dog.");
    assert!(!r.changed);
    assert!(r.matches.is_empty());
}

// --- Mixed entities ---

#[test]
fn mixed_email_phone_ssn() {
    let engine = RedactionEngine::new(default_cfg()).unwrap();
    let input = "Email: test@example.com, Phone: 555-123-4567, SSN: 123-45-6789";
    let r = engine.redact(input);
    assert!(r.changed);
    assert!(!r.redacted_text.contains("test@"));
    assert!(!r.redacted_text.contains("555-123"));
    assert!(r.matches.len() >= 3);
}

// --- Edge cases ---

#[test]
fn empty_input() {
    let engine = RedactionEngine::new(default_cfg()).unwrap();
    let r = engine.redact("");
    assert!(!r.changed);
    assert_eq!(r.redacted_text, "");
}

#[test]
fn whitespace_only() {
    let engine = RedactionEngine::new(default_cfg()).unwrap();
    let r = engine.redact("   \n\t  ");
    assert!(!r.changed);
}

#[test]
fn very_long_input_without_pii() {
    let engine = RedactionEngine::new(default_cfg()).unwrap();
    let long_text = "lorem ipsum dolor sit amet ".repeat(1000);
    let r = engine.redact(&long_text);
    assert!(!r.changed);
}
