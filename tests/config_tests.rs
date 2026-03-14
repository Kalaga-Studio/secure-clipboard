use secure_clipboard::config::AppConfig;

#[test]
fn default_config_has_all_rules_enabled() {
    let cfg = AppConfig::default();
    assert!(cfg.enabled);
    assert!(cfg.redaction.redact_email);
    assert!(cfg.redaction.redact_phone);
    assert!(cfg.redaction.redact_address);
    assert!(cfg.redaction.redact_urls_with_tokens);
    assert!(cfg.redaction.redact_account_numbers);
    assert!(cfg.redaction.redact_ssn);
    assert!(cfg.redaction.redact_labeled_fields);
    assert!(cfg.redaction.redact_dates);
    assert!(cfg.redaction.redact_names_in_salutations);
    assert!(cfg.redaction.redact_ip_addresses);
    assert!(cfg.redaction.redact_iban);
    assert!(cfg.redaction.redact_passport);
}

#[test]
fn default_config_hotkey() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.hotkey.modifiers, vec!["ctrl", "shift"]);
    assert_eq!(cfg.hotkey.key, "C");
    assert!(!cfg.hotkey.copy_before_redact);
}

#[test]
fn default_config_retry_values() {
    let cfg = AppConfig::default();
    assert_eq!(cfg.clipboard_retry_count, 6);
    assert_eq!(cfg.clipboard_retry_delay_ms, 40);
}

#[test]
fn config_round_trip_toml() {
    let cfg = AppConfig::default();
    let toml_str = toml::to_string_pretty(&cfg).expect("serialize");
    let parsed: AppConfig = toml::from_str(&toml_str).expect("deserialize");
    assert_eq!(parsed.enabled, cfg.enabled);
    assert_eq!(parsed.redaction.redact_email, cfg.redaction.redact_email);
    assert_eq!(
        parsed.redaction.redact_ip_addresses,
        cfg.redaction.redact_ip_addresses
    );
    assert_eq!(parsed.redaction.redact_iban, cfg.redaction.redact_iban);
    assert_eq!(
        parsed.redaction.redact_passport,
        cfg.redaction.redact_passport
    );
}

#[test]
fn config_partial_toml_uses_defaults() {
    let partial = r#"
        enabled = false
        clipboard_retry_count = 3
        clipboard_retry_delay_ms = 100

        [hotkey]
        modifiers = ["alt"]
        key = "R"
        copy_before_redact = true
        copy_settle_delay_ms = 50

        [redaction]
        redact_email = false
    "#;
    let cfg: AppConfig = toml::from_str(partial).expect("parse partial config");
    assert!(!cfg.enabled);
    assert!(!cfg.redaction.redact_email);
    // Other redaction fields should default to true
    assert!(cfg.redaction.redact_phone);
    assert!(cfg.redaction.redact_ssn);
    assert!(cfg.redaction.redact_ip_addresses);
}

#[test]
fn config_empty_custom_names_and_allowlist() {
    let cfg = AppConfig::default();
    assert!(cfg.redaction.custom_names.is_empty());
    assert!(cfg.redaction.allowlist_tokens.is_empty());
}

#[test]
fn config_custom_names_serialize() {
    let mut cfg = AppConfig::default();
    cfg.redaction.custom_names = vec!["Alice".into(), "Bob".into()];
    let toml_str = toml::to_string_pretty(&cfg).expect("serialize");
    let parsed: AppConfig = toml::from_str(&toml_str).expect("deserialize");
    assert_eq!(parsed.redaction.custom_names, vec!["Alice", "Bob"]);
}

#[test]
fn redaction_config_defaults_match_appconfig_defaults() {
    let app_cfg = AppConfig::default();
    let r = &app_cfg.redaction;
    assert!(r.redact_email);
    assert!(r.redact_phone);
    assert!(r.redact_address);
    assert!(r.redact_urls_with_tokens);
    assert!(r.redact_account_numbers);
    assert!(r.redact_ssn);
    assert!(r.redact_labeled_fields);
    assert!(r.redact_dates);
    assert!(r.redact_names_in_salutations);
    assert!(r.redact_ip_addresses);
    assert!(r.redact_iban);
    assert!(r.redact_passport);
}
