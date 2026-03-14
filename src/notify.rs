use crate::redaction::RedactionResult;

#[cfg(windows)]
pub fn show_redaction_toast(result: &RedactionResult) {
    use std::collections::HashMap;
    use win_toast_notify::{Audio, Duration, WinToastNotify};

    let (title, message) = if !result.changed {
        (
            "Secure Clipboard",
            "No sensitive data detected. Clipboard unchanged.".to_string(),
        )
    } else {
        let mut entity_counts: HashMap<&str, usize> = HashMap::new();
        for m in &result.matches {
            *entity_counts.entry(m.entity.as_str()).or_insert(0) += 1;
        }

        let mut summary_parts: Vec<String> = entity_counts
            .iter()
            .map(|(entity, count)| format!("{count} {entity}"))
            .collect();
        summary_parts.sort();

        let summary = summary_parts.join(", ");
        (
            "Secure Clipboard — Redacted",
            format!("Masked {} item(s): {}", result.matches.len(), summary),
        )
    };

    let toast = WinToastNotify::new()
        .set_title(title)
        .set_messages(vec![&message])
        .set_duration(Duration::Short)
        .set_audio(Audio::Silent, win_toast_notify::Loop::False);

    if let Err(e) = toast.show() {
        log::warn!("toast notification failed: {e}");
    }
}

#[cfg(not(windows))]
pub fn show_redaction_toast(result: &RedactionResult) {
    if result.changed {
        log::info!(
            "[notify stub] redaction applied, {} matches",
            result.matches.len()
        );
    } else {
        log::info!("[notify stub] no sensitive data detected");
    }
}

pub fn show_disabled_toast() {
    #[cfg(windows)]
    {
        use win_toast_notify::{Audio, Duration, WinToastNotify};

        let toast = WinToastNotify::new()
            .set_title("Secure Clipboard")
            .set_messages(vec!["Redaction is currently disabled."])
            .set_duration(Duration::Short)
            .set_audio(Audio::Silent, win_toast_notify::Loop::False);

        if let Err(e) = toast.show() {
            log::warn!("toast notification failed: {e}");
        }
    }

    #[cfg(not(windows))]
    {
        log::info!("[notify stub] redaction disabled");
    }
}

pub fn show_error_toast(context: &str) {
    #[cfg(windows)]
    {
        use win_toast_notify::{Audio, Duration, WinToastNotify};

        let toast = WinToastNotify::new()
            .set_title("Secure Clipboard — Error")
            .set_messages(vec![context])
            .set_duration(Duration::Short)
            .set_audio(Audio::WinDefault, win_toast_notify::Loop::False);

        if let Err(e) = toast.show() {
            log::warn!("toast notification failed: {e}");
        }
    }

    #[cfg(not(windows))]
    {
        log::warn!("[notify stub] error: {context}");
    }
}
