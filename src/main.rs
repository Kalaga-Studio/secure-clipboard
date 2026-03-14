use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use log::{error, info, warn};

use secure_clipboard::clipboard::ClipboardClient;
use secure_clipboard::config::AppConfig;
use secure_clipboard::hotkey::HotkeyController;
use secure_clipboard::notify;
use secure_clipboard::redaction::RedactionEngine;
use secure_clipboard::tray::{TrayAction, TrayController};

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = AppConfig::load_or_create_default()?;
    let enabled = Arc::new(AtomicBool::new(config.enabled));
    let mut clipboard = ClipboardClient::new(
        config.clipboard_retry_count,
        config.clipboard_retry_delay_ms,
    )?;
    let engine = RedactionEngine::new(config.redaction.clone())?;

    let hotkey = HotkeyController::new(&config.hotkey)?;
    let tray = TrayController::new(enabled.clone())?;

    info!("secure-clipboard started");

    loop {
        #[cfg(windows)]
        pump_windows_messages();

        if let Some(hotkey_event) = hotkey.poll_event() {
            if hotkey.is_activation_event(&hotkey_event) {
                info!("hotkey pressed; starting redaction cycle");
                run_redaction_cycle(
                    &config,
                    &engine,
                    &mut clipboard,
                    enabled.load(Ordering::Relaxed),
                );
            }
        }

        if let Some(action) = tray.poll_action() {
            match action {
                TrayAction::Enable => enabled.store(true, Ordering::Relaxed),
                TrayAction::Disable => enabled.store(false, Ordering::Relaxed),
                TrayAction::Quit => {
                    info!("secure-clipboard shutting down");
                    break;
                }
            }
        }

        thread::sleep(Duration::from_millis(25));
    }

    Ok(())
}

#[cfg(windows)]
fn pump_windows_messages() {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
    };

    // Drain the thread message queue so global hotkey/tray events get delivered.
    // (Some libraries require a Win32 message pump on the creating thread.)
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while PeekMessageW(&mut msg as *mut MSG, 0 as HWND, 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg as *const MSG);
            DispatchMessageW(&msg as *const MSG);
        }
    }
}

fn run_redaction_cycle(
    config: &AppConfig,
    engine: &RedactionEngine,
    clipboard: &mut ClipboardClient,
    enabled: bool,
) {
    if !enabled {
        notify::show_disabled_toast();
        return;
    }

    if config.hotkey.copy_before_redact {
        if let Err(err) = clipboard.send_ctrl_c() {
            warn!("failed to trigger copy shortcut: {err}");
        } else {
            thread::sleep(Duration::from_millis(config.hotkey.copy_settle_delay_ms));
        }
    }

    let text = match clipboard.read_text() {
        Ok(Some(text)) => {
            info!("clipboard read {} characters", text.chars().count());
            text
        }
        Ok(None) => {
            warn!("clipboard appears empty or non-text after copy");
            notify::show_error_toast("Clipboard is empty or non-text.");
            return;
        }
        Err(err) => {
            warn!("clipboard read failed: {err}");
            notify::show_error_toast("Failed to read clipboard.");
            return;
        }
    };

    let result = engine.redact(&text);
    if !result.changed {
        info!("no redaction changes detected; re-writing clipboard anyway");
    } else {
        info!("redaction changed text; writing redacted clipboard");
    }

    let out = if result.changed {
        &result.redacted_text
    } else {
        &text
    };
    if let Err(err) = clipboard.write_text(out) {
        error!("clipboard write failed: {err}");
        notify::show_error_toast("Failed to write to clipboard.");
        return;
    }
    info!("clipboard write succeeded");

    notify::show_redaction_toast(&result);

    if result.changed {
        info!(
            "redaction applied; entities_redacted={} total_matches={}",
            result
                .matches
                .iter()
                .map(|m| m.entity.as_str())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            result.matches.len()
        );
    }
}
