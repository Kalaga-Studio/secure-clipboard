# Contributing

Thanks for contributing to Secure Clipboard.

## Development workflow

1. Fork and create a feature branch.
2. Add or update tests for behavior changes.
3. Run `cargo test` and `cargo clippy` before submitting a PR.
4. Keep privacy guarantees intact (no network calls for redaction logic).

## Scope guidance

- Prefer deterministic local rules over heavy dependencies.
- Minimize CPU overhead in the hotkey path.
- Never log raw clipboard contents.
