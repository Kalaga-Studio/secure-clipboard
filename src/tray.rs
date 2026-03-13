use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Enable,
    Disable,
    Quit,
}

#[cfg(windows)]
mod imp {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    use super::{Result, TrayAction};

    use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
    use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

    pub struct TrayController {
        _tray: TrayIcon,
        enable_id: MenuId,
        disable_id: MenuId,
        quit_id: MenuId,
        _enabled_state: Arc<AtomicBool>,
    }

    impl TrayController {
        pub fn new(enabled_state: Arc<AtomicBool>) -> Result<Self> {
            let menu = Menu::new();
            let enable_item = MenuItem::new("Enable Redaction", true, None);
            let disable_item = MenuItem::new("Disable Redaction", true, None);
            let quit_item = MenuItem::new("Quit", true, None);
            let _ = menu.append_items(&[&enable_item, &disable_item, &quit_item]);

            let tray = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip(if enabled_state.load(Ordering::Relaxed) {
                    "Secure Clipboard: enabled"
                } else {
                    "Secure Clipboard: disabled"
                })
                .with_icon(default_icon()?)
                .build()?;

            Ok(Self {
                _tray: tray,
                enable_id: enable_item.id().clone(),
                disable_id: disable_item.id().clone(),
                quit_id: quit_item.id().clone(),
                _enabled_state: enabled_state,
            })
        }

        pub fn poll_action(&self) -> Option<TrayAction> {
            let event = MenuEvent::receiver().try_recv().ok()?;
            if event.id == self.enable_id {
                return Some(TrayAction::Enable);
            }
            if event.id == self.disable_id {
                return Some(TrayAction::Disable);
            }
            if event.id == self.quit_id {
                return Some(TrayAction::Quit);
            }
            None
        }
    }

    fn default_icon() -> Result<Icon> {
        // Simple 16x16 icon generated at runtime to avoid bundling assets for MVP.
        let width = 16;
        let height = 16;
        let mut rgba = vec![0u8; width * height * 4];
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                let border = x == 0 || y == 0 || x == width - 1 || y == height - 1;
                let (r, g, b, a) = if border {
                    (0x11, 0x11, 0x11, 0xFF)
                } else if x > 3 && x < 12 && y > 3 && y < 12 {
                    (0x2E, 0x7D, 0x32, 0xFF)
                } else {
                    (0xFA, 0xFA, 0xFA, 0xFF)
                };
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = a;
            }
        }
        Ok(Icon::from_rgba(rgba, width as u32, height as u32)?)
    }
}

#[cfg(not(windows))]
mod imp {
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    use super::{Result, TrayAction};

    pub struct TrayController {
        _enabled_state: Arc<AtomicBool>,
    }

    impl TrayController {
        pub fn new(enabled_state: Arc<AtomicBool>) -> Result<Self> {
            Ok(Self {
                _enabled_state: enabled_state,
            })
        }

        pub fn poll_action(&self) -> Option<TrayAction> {
            None
        }
    }
}

pub use imp::TrayController;
