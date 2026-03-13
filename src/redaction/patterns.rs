use anyhow::Result;
use regex::Regex;

use crate::redaction::mask::mask_phone;
use crate::redaction::{EntityType, MatchRecord};

pub fn email_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"(?i)\b[a-z0-9._%+\-]+@[a-z0-9.\-]+\.[a-z]{2,}\b",
    )?)
}

// Matches long digit strings: credit cards (16), bank accounts (10+), etc.
pub fn account_pattern() -> Result<Regex> {
    Ok(Regex::new(r"\b(?:\d[ \-]?){10,19}\b")?)
}

// SSN: 123-45-6789
pub fn ssn_pattern() -> Result<Regex> {
    Ok(Regex::new(r"\b\d{3}-\d{2}-\d{4}\b")?)
}

// Labeled sensitive fields: "Password: abc123", "CVV: 123", "Exp: 12/26", etc.
pub fn labeled_pii_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"(?i)\b(?:password|pass(?:word)?|cvv|cvc|exp(?:iry|iration)?|dob|d\.?o\.?b\.?|license|licence)\s*:\s*\S+",
    )?)
}

// Date-of-birth style dates: 01/23/1985  or  1985-01-23
pub fn date_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"\b(?:\d{1,2}[/\-]\d{1,2}[/\-]\d{4}|\d{4}[/\-]\d{1,2}[/\-]\d{1,2})\b",
    )?)
}

pub fn url_with_token_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"(?i)\bhttps?://[^\s]+(?:token|apikey|api_key|auth|session|key|sig|signature|code)=[^\s&]+[^\s]*",
    )?)
}

pub fn address_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"(?im)\b\d{1,6}\s+[A-Za-z0-9][A-Za-z0-9.\- ]{2,40}\s(?:street|st|avenue|ave|road|rd|boulevard|blvd|drive|dr|lane|ln|way|court|ct)\b(?:[^\n,]{0,40})",
    )?)
}

// Captures first + optional last name after a greeting.
// Uses non-capturing group for keyword so (?i) doesn't bleed into the name capture.
pub fn salutation_name_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"(?m)(?i:(?:dear|hello|hi))\s+(?P<name>[A-Z][a-z]{1,30}(?:\s+[A-Z][a-z]{1,30})?)\b",
    )?)
}

// Captures first + optional last name in a sign-off block.
pub fn signoff_name_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"(?m)(?i:(?:best|thanks|thank\s+you|regards|sincerely|cheers))\s*,?\s*\r?\n\s*(?P<name>[A-Z][a-z]{1,30}(?:\s+[A-Z][a-z]{1,30})?)",
    )?)
}

pub struct PhonePatternSet {
    patterns: Vec<Regex>,
}

impl PhonePatternSet {
    pub fn new() -> Result<Self> {
        let patterns = vec![
            // +1 771-322-0123
            Regex::new(r"\+\d{1,3}[ \-]?\(?\d{3}\)?[ \-]?\d{3}[ \-]?\d{4}\b")?,
            // (555) 123-4567
            Regex::new(r"\(\d{3}\)\s*\d{3}[ \-]?\d{4}\b")?,
            // 555-123-4567 or 555 123 4567  (bare 10-digit, no country code)
            Regex::new(r"\b\d{3}[ \-]\d{3}[ \-]\d{4}\b")?,
        ];
        Ok(Self { patterns })
    }

    pub fn apply(&self, text: &mut String, records: &mut Vec<MatchRecord>) {
        for re in &self.patterns {
            apply_phone_pattern(text, re, records);
        }
    }
}

fn apply_phone_pattern(text: &mut String, re: &Regex, records: &mut Vec<MatchRecord>) {
    let snapshot = text.clone();
    let mut cursor = 0usize;
    let mut output = String::with_capacity(snapshot.len());

    for m in re.find_iter(&snapshot) {
        output.push_str(&snapshot[cursor..m.start()]);
        output.push_str(&mask_phone(m.as_str()));
        records.push(MatchRecord {
            entity: EntityType::Phone,
            span: (m.start(), m.end()),
        });
        cursor = m.end();
    }

    output.push_str(&snapshot[cursor..]);
    *text = output;
}
