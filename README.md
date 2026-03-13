# Secure Clipboard (Rust)

Secure Clipboard is a Windows-first smart clipboard utility that keeps normal clipboard workflows and adds on-demand local redaction for sensitive data before pasting into AI tools.

## Why this exists

When you copy email threads, transaction notes, or support messages into AI models, personal data can leak accidentally. This tool masks common sensitive patterns while preserving readability.

## Features (MVP)

- On-demand global hotkey redaction (`Ctrl + Shift + C` by default).
- Local-only processing (no cloud calls, no network dependency).
- Sensitive pattern masking:
  - email addresses
  - phone numbers
  - account-like number strings
  - URL query tokens (`token`, `api_key`, `session`, etc.)
  - basic street-address patterns
  - names in salutations/sign-offs
- Tray controls:
  - enable redaction
  - disable redaction
  - quit app
- Configurable rules through AppData config file.

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

## How it works

1. Press the configured global hotkey.
2. App triggers `Ctrl+C` to copy the current selection.
3. Clipboard text is read in-memory.
4. Redaction engine applies deterministic masking rules.
5. Redacted text is written back to clipboard.
6. Paste normally (`Ctrl+V`) into your target app.

## Configuration

At first launch, config is created at:

`%APPDATA%/secure-clipboard/secure-clipboard/config/config.toml`

See defaults in `config/default.toml`.

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

## Packaging (MSI)

This repo includes a WiX template at `wix/main.wxs` and ships a pinned WiX 3.14 toolset under `tools/wix314/` so you can build an installer without installing WiX globally.

Recommended packaging flow:

```bash
cargo install cargo-wix
cargo wix -b tools/wix314
```

The installer is generated in `target/wix/` as `secure-clipboard-<version>-x86_64.msi`.

## Open-source notes

- Language: Rust
- License: MIT
- Local-only redaction by default
- No telemetry included in MVP
