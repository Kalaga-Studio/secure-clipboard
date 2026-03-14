# Secure Clipboard (Rust)

Secure Clipboard is a Windows-first smart clipboard utility that keeps normal clipboard workflows and adds on-demand local redaction for sensitive data before pasting into AI tools.

## Why this exists

When you copy email threads, transaction notes, or support messages into AI models, personal data can leak accidentally. This tool masks common sensitive patterns while preserving readability.

## Features

- On-demand global hotkey redaction (`Ctrl + Shift + C` by default).
- Local-only processing (no cloud calls, no network dependency).
- Toast notifications after each redaction cycle showing what was masked.
- Sensitive pattern masking:
  - Email addresses
  - Phone numbers (US, international, dotted formats)
  - SSNs (dash and space separated)
  - Account-like number strings (credit cards, bank accounts)
  - URL query tokens (`token`, `api_key`, `session`, `access_token`, `secret`, etc.)
  - Street addresses (with expanded street type support)
  - Names in salutations and sign-offs
  - Labeled fields (`password:`, `cvv:`, `api_key:`, `secret:`, `token=`, `pin:`, etc.)
  - Dates (US, ISO, and European dot formats)
  - IPv4 addresses
  - IBANs (compact and space-separated)
  - Passport numbers (context-aware, requires "passport" label)
- System tray controls (enable / disable / quit).
- Configurable rules through TOML config file — each entity type is individually toggleable.
- Allowlist to preserve specific tokens from redaction.
- Custom name list for redacting project-specific names.

## Example

Input:

```text
Dear Fresno,

I am reaching out to share my contact info for easier communication. Here's my phone +1 771-322-0123. Looking forward to meeting you.

Best,
Alberto
```

Output:

```text
Dear XXXXXX,

I am reaching out to share my contact info for easier communication. Here's my phone +X XXX-XXX-XXXX. Looking forward to meeting you.

Best,
XXXXXXX
```

A toast notification will appear: **"Masked 3 item(s): 1 name, 1 phone"**

## How it works

1. Press the configured global hotkey.
2. (Optional) App triggers `Ctrl+C` to copy the current selection.
3. Clipboard text is read in-memory.
4. Redaction engine applies deterministic masking rules in priority order.
5. Redacted text is written back to clipboard.
6. A Windows toast notification summarizes what was redacted.
7. Paste normally (`Ctrl+V`) into your target app.

## Configuration

At first launch, config is created at:

`%APPDATA%/secure-clipboard/secure-clipboard/config/config.toml`

See defaults in `config/default.toml`.

### Redaction toggles

All entity types can be individually enabled or disabled:

```toml
[redaction]
redact_email = true
redact_phone = true
redact_address = true
redact_urls_with_tokens = true
redact_account_numbers = true
redact_ssn = true
redact_labeled_fields = true
redact_dates = true
redact_names_in_salutations = true
redact_ip_addresses = true
redact_iban = true
redact_passport = true
custom_names = []
allowlist_tokens = []
```

## Development

### Prerequisites

- Rust stable toolchain
- Windows 10/11

### Build

```bash
cargo build --release
```

### Test

```bash
cargo test
```

The test suite includes **76 tests** covering:

- Unit tests for all masking functions (edge cases, unicode, empty inputs)
- Pattern matching tests for every entity type (positive and negative cases)
- Config serialization round-trip and partial config with defaults
- Integration tests for full redaction cycles

### Lint

```bash
cargo clippy --all-targets -- -D warnings
```

### Format

```bash
cargo fmt --all -- --check
```

## CI/CD

This project uses GitHub Actions for continuous integration. On every push and pull request to `main`, the pipeline runs:

1. **Check** — `cargo check --all-targets`
2. **Rustfmt** — formatting verification
3. **Clippy** — lint-free build with `-D warnings`
4. **Test** — full test suite
5. **Build Release** — produces `secure-clipboard.exe` as a downloadable artifact

See `.github/workflows/ci.yml` for the full configuration.

## Packaging (MSI)

This repo includes a WiX template at `wix/main.wxs` and an optional helper script that downloads a pinned WiX 3.14 toolset into `tools/wix314/` (which is ignored by Git).

Recommended packaging flow (from the repo root, in PowerShell):

```powershell
.\scripts\setup-wix-and-build.ps1
```

This will:

- download WiX 3.14 binaries into `tools/wix314/` if they are not present
- ensure `cargo-wix` is installed
- run `cargo wix -b tools/wix314`

The installer is generated in `target/wix/` as `secure-clipboard-<version>-x86_64.msi`.

## Architecture

```
User presses Ctrl+Shift+C
        │
        ▼
  HotkeyController (global-hotkey)
        │
        ▼
  main.rs event loop
        │
        ├──► (optional) ClipboardClient.send_ctrl_c()
        │
        ▼
  ClipboardClient.read_text()  ← with retry logic
        │
        ▼
  RedactionEngine.redact()
        │  ├─ labeled fields   → mask_labeled_field
        │  ├─ emails           → mask_email
        │  ├─ phones           → mask_phone
        │  ├─ SSNs             → mask_account_like
        │  ├─ URL tokens       → mask_url_keep_host
        │  ├─ accounts         → mask_account_like
        │  ├─ dates            → mask_account_like
        │  ├─ addresses        → mask_address
        │  ├─ IP addresses     → mask_account_like
        │  ├─ IBANs            → mask_alphanumeric
        │  ├─ passports        → mask_alphanumeric
        │  ├─ names            → mask_name
        │  └─ allowlist restore
        │
        ▼
  ClipboardClient.write_text()  ← Win32 SetClipboardData (UTF-16)
        │
        ▼
  Toast notification (summary of what was redacted)
        │
        ▼
  User pastes normally (Ctrl+V)
```

## Open-source notes

- Language: Rust
- License: MIT
- Local-only redaction by default
- No telemetry
- No network calls
