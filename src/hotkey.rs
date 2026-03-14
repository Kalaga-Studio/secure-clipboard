use anyhow::{anyhow, Result};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

use crate::config::HotkeyConfig;

pub struct HotkeyController {
    _manager: GlobalHotKeyManager,
    hotkey_id: u32,
}

impl HotkeyController {
    pub fn new(config: &HotkeyConfig) -> Result<Self> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| anyhow!("failed to create hotkey manager: {e}"))?;
        let hotkey = parse_hotkey(config)?;
        let hotkey_id = hotkey.id();
        manager
            .register(hotkey)
            .map_err(|e| anyhow!("failed to register hotkey: {e}"))?;

        Ok(Self {
            _manager: manager,
            hotkey_id,
        })
    }

    pub fn hotkey_id(&self) -> u32 {
        self.hotkey_id
    }

    pub fn poll_event(&self) -> Option<GlobalHotKeyEvent> {
        GlobalHotKeyEvent::receiver().try_recv().ok()
    }

    pub fn is_activation_event(&self, event: &GlobalHotKeyEvent) -> bool {
        event.id == self.hotkey_id && event.state == HotKeyState::Pressed
    }
}

fn parse_hotkey(config: &HotkeyConfig) -> Result<HotKey> {
    let mut modifiers = Modifiers::empty();

    for token in config.modifiers.iter().map(|m| m.to_ascii_lowercase()) {
        match token.as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "super" | "win" | "windows" => modifiers |= Modifiers::SUPER,
            _ => return Err(anyhow!("unsupported hotkey modifier: {token}")),
        }
    }

    let key = match config.key.to_ascii_uppercase().as_str() {
        "A" => Code::KeyA,
        "B" => Code::KeyB,
        "C" => Code::KeyC,
        "D" => Code::KeyD,
        "E" => Code::KeyE,
        "F" => Code::KeyF,
        "G" => Code::KeyG,
        "H" => Code::KeyH,
        "I" => Code::KeyI,
        "J" => Code::KeyJ,
        "K" => Code::KeyK,
        "L" => Code::KeyL,
        "M" => Code::KeyM,
        "N" => Code::KeyN,
        "O" => Code::KeyO,
        "P" => Code::KeyP,
        "Q" => Code::KeyQ,
        "R" => Code::KeyR,
        "S" => Code::KeyS,
        "T" => Code::KeyT,
        "U" => Code::KeyU,
        "V" => Code::KeyV,
        "W" => Code::KeyW,
        "X" => Code::KeyX,
        "Y" => Code::KeyY,
        "Z" => Code::KeyZ,
        "F1" => Code::F1,
        "F2" => Code::F2,
        "F3" => Code::F3,
        "F4" => Code::F4,
        "F5" => Code::F5,
        "F6" => Code::F6,
        "F7" => Code::F7,
        "F8" => Code::F8,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        other => return Err(anyhow!("unsupported hotkey key: {other}")),
    };

    Ok(HotKey::new(Some(modifiers), key))
}
