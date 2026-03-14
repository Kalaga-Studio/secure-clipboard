use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use arboard::Clipboard;

pub struct ClipboardClient {
    clipboard: Clipboard,
    retry_count: u32,
    retry_delay_ms: u64,
}

impl ClipboardClient {
    pub fn new(retry_count: u32, retry_delay_ms: u64) -> Result<Self> {
        let clipboard = Clipboard::new().map_err(|e| anyhow!("failed to open clipboard: {e}"))?;
        Ok(Self {
            clipboard,
            retry_count,
            retry_delay_ms,
        })
    }

    pub fn read_text(&mut self) -> Result<Option<String>> {
        self.with_retry(|clipboard| match clipboard.get_text() {
            Ok(text) => Ok(Some(text)),
            Err(_) => Ok(None),
        })
    }

    // On Windows: bypass arboard's delayed-rendering path and write directly
    // via Win32 SetClipboardData so any app can paste immediately.
    #[cfg(windows)]
    pub fn write_text(&mut self, text: &str) -> Result<()> {
        Self::win32_write_with_retry(text, self.retry_count, self.retry_delay_ms)
    }

    #[cfg(not(windows))]
    pub fn write_text(&mut self, text: &str) -> Result<()> {
        self.with_retry(|clipboard| {
            clipboard
                .set_text(text.to_owned())
                .map_err(|e| anyhow!("failed to set clipboard text: {e}"))
        })
    }

    fn with_retry<T, F>(&mut self, mut f: F) -> Result<T>
    where
        F: FnMut(&mut Clipboard) -> Result<T>,
    {
        let mut last_error: Option<anyhow::Error> = None;
        let retry_count = self.retry_count;
        let retry_delay_ms = self.retry_delay_ms;

        for _ in 0..=retry_count {
            match f(&mut self.clipboard) {
                Ok(result) => return Ok(result),
                Err(err) => {
                    last_error = Some(err);
                    thread::sleep(Duration::from_millis(retry_delay_ms));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("clipboard operation failed")))
    }

    #[cfg(windows)]
    fn win32_write_with_retry(text: &str, retry_count: u32, retry_delay_ms: u64) -> Result<()> {
        let mut last_err: Option<anyhow::Error> = None;
        for _ in 0..=retry_count {
            match Self::win32_write_once(text) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_err = Some(e);
                    thread::sleep(Duration::from_millis(retry_delay_ms));
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow!("Win32 clipboard write failed after retries")))
    }

    // Direct Win32 clipboard write: GlobalAlloc -> copy UTF-16 -> SetClipboardData.
    // Unlike arboard's delayed-rendering approach, this stores the data immediately
    // so any target app (Notepad, browser, etc.) can paste without a WM_RENDERFORMAT
    // round-trip back to us.
    #[cfg(windows)]
    fn win32_write_once(text: &str) -> Result<()> {
        use windows_sys::Win32::System::DataExchange::{
            CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
        };
        use windows_sys::Win32::System::Memory::{
            GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
        };

        const CF_UNICODETEXT: u32 = 13;

        // Encode as UTF-16 with null terminator.
        let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0u16)).collect();
        let byte_size = utf16.len() * std::mem::size_of::<u16>();

        unsafe {
            let h_mem = GlobalAlloc(GMEM_MOVEABLE, byte_size);
            if h_mem.is_null() {
                return Err(anyhow!("GlobalAlloc failed"));
            }

            let dst = GlobalLock(h_mem) as *mut u16;
            if dst.is_null() {
                return Err(anyhow!("GlobalLock failed"));
            }
            std::ptr::copy_nonoverlapping(utf16.as_ptr(), dst, utf16.len());
            GlobalUnlock(h_mem);

            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return Err(anyhow!("OpenClipboard failed"));
            }
            EmptyClipboard();
            let result = SetClipboardData(CF_UNICODETEXT, h_mem);
            CloseClipboard();

            if result.is_null() {
                return Err(anyhow!("SetClipboardData failed"));
            }
        }
        Ok(())
    }

    #[cfg(windows)]
    pub fn send_ctrl_c(&self) -> Result<()> {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL,
        };

        unsafe fn key_input(vk: u16, flags: u32) -> INPUT {
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            }
        }

        let mut inputs = vec![
            unsafe { key_input(VK_CONTROL, 0) },
            unsafe { key_input(b'C' as u16, 0) },
            unsafe { key_input(b'C' as u16, KEYEVENTF_KEYUP) },
            unsafe { key_input(VK_CONTROL, KEYEVENTF_KEYUP) },
        ];

        let sent = unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_mut_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            )
        };
        if sent == 0 {
            return Err(anyhow!("SendInput failed when dispatching Ctrl+C"));
        }
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn send_ctrl_c(&self) -> Result<()> {
        Err(anyhow!(
            "copy shortcut simulation is only supported on Windows"
        ))
    }
}
