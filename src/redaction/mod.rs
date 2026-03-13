pub mod mask;
pub mod patterns;

use regex::Regex;

use crate::config::RedactionConfig;
use crate::redaction::mask::{
    mask_account_like, mask_address, mask_email, mask_labeled_field, mask_name, mask_url_keep_host,
};
use crate::redaction::patterns::{
    account_pattern, address_pattern, date_pattern, email_pattern, labeled_pii_pattern,
    salutation_name_pattern, signoff_name_pattern, ssn_pattern, url_with_token_pattern,
    PhonePatternSet,
};

#[derive(Debug, Clone)]
pub struct MatchRecord {
    pub entity: EntityType,
    pub span: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    Email,
    Phone,
    Address,
    UrlToken,
    AccountLike,
    Ssn,
    LabeledField,
    Date,
    Name,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Email => "email",
            EntityType::Phone => "phone",
            EntityType::Address => "address",
            EntityType::UrlToken => "url_token",
            EntityType::AccountLike => "account_like",
            EntityType::Ssn => "ssn",
            EntityType::LabeledField => "labeled_field",
            EntityType::Date => "date",
            EntityType::Name => "name",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RedactionResult {
    pub redacted_text: String,
    pub changed: bool,
    pub matches: Vec<MatchRecord>,
}

pub struct RedactionEngine {
    config: RedactionConfig,
    email_re: Regex,
    address_re: Regex,
    account_re: Regex,
    ssn_re: Regex,
    labeled_pii_re: Regex,
    date_re: Regex,
    url_token_re: Regex,
    salutation_re: Regex,
    signoff_re: Regex,
    phone_patterns: PhonePatternSet,
}

impl RedactionEngine {
    pub fn new(config: RedactionConfig) -> anyhow::Result<Self> {
        Ok(Self {
            config,
            email_re: email_pattern()?,
            address_re: address_pattern()?,
            account_re: account_pattern()?,
            ssn_re: ssn_pattern()?,
            labeled_pii_re: labeled_pii_pattern()?,
            date_re: date_pattern()?,
            url_token_re: url_with_token_pattern()?,
            salutation_re: salutation_name_pattern()?,
            signoff_re: signoff_name_pattern()?,
            phone_patterns: PhonePatternSet::new()?,
        })
    }

    pub fn redact(&self, input: &str) -> RedactionResult {
        let mut text = input.to_string();
        let mut records = Vec::new();

        log::info!(
            "redact: input {} chars | flags: email={} phone={} ssn={} acct={} labeled={} dates={} addr={} names={}",
            input.len(),
            self.config.redact_email,
            self.config.redact_phone,
            self.config.redact_ssn,
            self.config.redact_account_numbers,
            self.config.redact_labeled_fields,
            self.config.redact_dates,
            self.config.redact_address,
            self.config.redact_names_in_salutations,
        );

        // Labeled fields first so their values don't get partially matched
        // by more generic number/date patterns below.
        if self.config.redact_labeled_fields {
            let n = records.len();
            apply_regex_replacement(
                &mut text,
                &self.labeled_pii_re,
                &mut records,
                EntityType::LabeledField,
                |m| mask_labeled_field(m.as_str()),
            );
            log::info!("  labeled_fields: {} match(es)", records.len() - n);
        }

        if self.config.redact_email {
            let n = records.len();
            apply_regex_replacement(
                &mut text,
                &self.email_re,
                &mut records,
                EntityType::Email,
                |m| mask_email(m.as_str()),
            );
            log::info!("  email: {} match(es)", records.len() - n);
        }

        if self.config.redact_phone {
            let n = records.len();
            self.phone_patterns.apply(&mut text, &mut records);
            log::info!("  phone: {} match(es)", records.len() - n);
        }

        if self.config.redact_ssn {
            let n = records.len();
            apply_regex_replacement(
                &mut text,
                &self.ssn_re,
                &mut records,
                EntityType::Ssn,
                |m| mask_account_like(m.as_str()),
            );
            log::info!("  ssn: {} match(es)", records.len() - n);
        }

        if self.config.redact_urls_with_tokens {
            let n = records.len();
            apply_regex_replacement(
                &mut text,
                &self.url_token_re,
                &mut records,
                EntityType::UrlToken,
                |m| mask_url_keep_host(m.as_str()),
            );
            log::info!("  url_tokens: {} match(es)", records.len() - n);
        }

        if self.config.redact_account_numbers {
            let n = records.len();
            apply_regex_replacement(
                &mut text,
                &self.account_re,
                &mut records,
                EntityType::AccountLike,
                |m| mask_account_like(m.as_str()),
            );
            log::info!("  account_numbers: {} match(es)", records.len() - n);
        }

        if self.config.redact_dates {
            let n = records.len();
            apply_regex_replacement(
                &mut text,
                &self.date_re,
                &mut records,
                EntityType::Date,
                |m| mask_account_like(m.as_str()),
            );
            log::info!("  dates: {} match(es)", records.len() - n);
        }

        if self.config.redact_address {
            let n = records.len();
            apply_regex_replacement(
                &mut text,
                &self.address_re,
                &mut records,
                EntityType::Address,
                |m| mask_address(m.as_str()),
            );
            log::info!("  address: {} match(es)", records.len() - n);
        }

        if self.config.redact_names_in_salutations {
            let n = records.len();
            apply_name_redaction(&mut text, &self.salutation_re, &mut records);
            apply_name_redaction(&mut text, &self.signoff_re, &mut records);
            log::info!("  names: {} match(es)", records.len() - n);
        }

        for custom_name in self
            .config
            .custom_names
            .iter()
            .filter(|n| !n.trim().is_empty())
        {
            let escaped = regex::escape(custom_name.trim());
            if let Ok(name_re) = Regex::new(&format!(r"\b{}\b", escaped)) {
                apply_regex_replacement(
                    &mut text,
                    &name_re,
                    &mut records,
                    EntityType::Name,
                    |m| mask_name(m.as_str()),
                );
            }
        }

        if !self.config.allowlist_tokens.is_empty() {
            for token in &self.config.allowlist_tokens {
                if token.is_empty() {
                    continue;
                }
                let masked = mask_name(token);
                if text.contains(&masked) {
                    text = text.replace(&masked, token);
                }
            }
        }

        RedactionResult {
            changed: text != input,
            redacted_text: text,
            matches: records,
        }
    }
}

fn apply_name_redaction(text: &mut String, re: &Regex, records: &mut Vec<MatchRecord>) {
    let snapshot = text.clone();
    let mut cursor = 0usize;
    let mut output = String::with_capacity(snapshot.len());

    for captures in re.captures_iter(&snapshot) {
        let whole = captures.get(0).expect("whole match");
        let name = captures.name("name");
        if let Some(name_match) = name {
            output.push_str(&snapshot[cursor..name_match.start()]);
            output.push_str(&mask_name(name_match.as_str()));
            cursor = name_match.end();
            records.push(MatchRecord {
                entity: EntityType::Name,
                span: (name_match.start(), name_match.end()),
            });
        } else {
            output.push_str(&snapshot[cursor..whole.end()]);
            cursor = whole.end();
        }
    }
    output.push_str(&snapshot[cursor..]);
    *text = output;
}

fn apply_regex_replacement<F>(
    text: &mut String,
    re: &Regex,
    records: &mut Vec<MatchRecord>,
    entity: EntityType,
    replacer: F,
) where
    F: Fn(&regex::Match<'_>) -> String,
{
    let snapshot = text.clone();
    let mut cursor = 0usize;
    let mut output = String::with_capacity(snapshot.len());

    for m in re.find_iter(&snapshot) {
        output.push_str(&snapshot[cursor..m.start()]);
        output.push_str(&replacer(&m));
        records.push(MatchRecord {
            entity: entity.clone(),
            span: (m.start(), m.end()),
        });
        cursor = m.end();
    }
    output.push_str(&snapshot[cursor..]);
    *text = output;
}
