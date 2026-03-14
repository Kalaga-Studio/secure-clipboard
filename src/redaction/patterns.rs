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
// Requires word boundaries and digit groups separated by optional spaces/dashes.
pub fn account_pattern() -> Result<Regex> {
    Ok(Regex::new(r"\b(?:\d[ \-]?){10,19}\b")?)
}

// SSN: 123-45-6789 — also catches space-separated variants.
pub fn ssn_pattern() -> Result<Regex> {
    Ok(Regex::new(r"\b\d{3}[\- ]\d{2}[\- ]\d{4}\b")?)
}

// Labeled sensitive fields: "Password: abc123", "CVV: 123", "Exp: 12/26",
// "API Key: ...", "Secret: ...", "Token: ...", etc.
pub fn labeled_pii_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"(?i)\b(?:password|pass(?:word)?|pin|cvv|cvc|exp(?:iry|iration)?|dob|d\.?o\.?b\.?|license|licence|api[_\- ]?key|secret|token|auth[_\- ]?token|access[_\- ]?key|private[_\- ]?key|routing[_\- ]?number)\s*[:=]\s*\S+",
    )?)
}

// Dates: MM/DD/YYYY, YYYY-MM-DD, DD.MM.YYYY, and similar with separators / - .
pub fn date_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"\b(?:\d{1,2}[/\-\.]\d{1,2}[/\-\.]\d{2,4}|\d{4}[/\-\.]\d{1,2}[/\-\.]\d{1,2})\b",
    )?)
}

// URLs containing authentication tokens, API keys, session IDs, etc.
pub fn url_with_token_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"(?i)\bhttps?://[^\s]+(?:token|apikey|api_key|api-key|auth|session|key|sig|signature|code|secret|password|access_token|refresh_token)=[^\s&]+[^\s]*",
    )?)
}

// Street addresses: number + street name + street type suffix.
pub fn address_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"(?im)\b\d{1,6}\s+[A-Za-z0-9][A-Za-z0-9.\- ]{2,40}\s(?:street|st|avenue|ave|road|rd|boulevard|blvd|drive|dr|lane|ln|way|court|ct|place|pl|circle|cir|terrace|ter|parkway|pkwy|highway|hwy)\b(?:[^\n,]{0,40})",
    )?)
}

// Captures first + optional last name after a greeting.
pub fn salutation_name_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"(?m)(?i:(?:dear|hello|hi|hey|good\s+(?:morning|afternoon|evening)))\s+(?P<name>[A-Z][a-z]{1,30}(?:\s+[A-Z][a-z]{1,30})?)\b",
    )?)
}

// Captures first + optional last name in a sign-off block.
pub fn signoff_name_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"(?m)(?i:(?:best|thanks|thank\s+you|regards|sincerely|cheers|warm\s+regards|kind\s+regards|yours\s+truly|respectfully))\s*,?\s*\r?\n\s*(?P<name>[A-Z][a-z]{1,30}(?:\s+[A-Z][a-z]{1,30})?)",
    )?)
}

// IPv4 addresses — uses word boundary to avoid matching inside longer numbers.
pub fn ip_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b",
    )?)
}

// International Bank Account Number (IBAN): 2 letter country code + 2 check digits + up to 30 alphanumerics.
// Matches compact form (DE89370400440532013000) and space-separated groups (GB29 NWBK 6016 1331 9268 19).
pub fn iban_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"\b[A-Z]{2}\d{2}[  ]?[A-Z0-9]{4}(?:[  ]?[A-Z0-9]{4}){1,7}(?:[  ]?[A-Z0-9]{1,4})?\b",
    )?)
}

// Passport numbers: contextual match requiring a "passport" label nearby.
pub fn passport_pattern() -> Result<Regex> {
    Ok(Regex::new(
        r"(?i)(?:passport\s*(?:no|number|#|num)?)\s*[:=]?\s*([A-Z0-9]{6,12})",
    )?)
}

pub struct PhonePatternSet {
    patterns: Vec<Regex>,
}

impl PhonePatternSet {
    pub fn new() -> Result<Self> {
        let patterns = vec![
            // International: +1 (555) 123-4567 or +44 20 7946 0958
            Regex::new(r"\+\d{1,3}[ \-]?\(?\d{2,4}\)?[ \-]?\d{3,4}[ \-]?\d{3,4}\b")?,
            // US/CA: (555) 123-4567
            Regex::new(r"\(\d{3}\)\s*\d{3}[ \-]?\d{4}\b")?,
            // US: 555-123-4567 or 555 123 4567
            Regex::new(r"\b\d{3}[ \-]\d{3}[ \-]\d{4}\b")?,
            // Dotted format: 555.123.4567
            Regex::new(r"\b\d{3}\.\d{3}\.\d{4}\b")?,
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
