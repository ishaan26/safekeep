# AGENTS.md

Guidance for coding agents working in this repository.

## Project overview

`safekeep` is a Rust 2024 backup library. The root crate exposes:

- `BackupOptions`: builder-style backup configuration and execution.
- `BackupType`: trait for values that can serialize themselves for backup.
- `BackupError`: shared error type.
- `#[derive(BackupType)]`: re-exported from the local proc-macro crate `safekeep-derive`.

The derive macro supports `#[safekeep(format = "json" | "yaml" | "toml")]` and defaults to JSON.

## Repository layout

- `Cargo.toml` / `Cargo.lock`: root `safekeep` crate.
- `src/lib.rs`: public API, standard `BackupType` impls, `BackupOptions`, unit tests.
- `src/error.rs`: `BackupError` definitions and conversions.
- `safekeep-derive/`: local proc-macro crate for `#[derive(BackupType)]`.
- `examples/`: small usage examples.
- `README.md`, `CHANGELOG.md`, `LICENSE`: package metadata/docs.

## Common commands

Run from the repository root unless noted.

```bash
cargo test
cargo test --all-features
cargo fmt --all
cargo clippy --all-targets --all-features
cargo run --example backup_options
cargo run --example derive_backup_type
```

Format/check the proc-macro crate directly when editing it:

```bash
(cd safekeep-derive && cargo fmt --all)
(cd safekeep-derive && cargo test)
(cd safekeep-derive && cargo clippy --all-targets)
```

## Coding guidelines

- Keep the public API small and documented; this is a library crate intended for publication.
- Preserve the builder-style API on `BackupOptions` unless intentionally redesigning it.
- When adding serialization formats, update all relevant places:
  - root `Cargo.toml` features/dependencies,
  - `safekeep-derive/src/lib.rs`,
  - trait docs in `src/lib.rs`,
  - examples/tests/README as appropriate.
- Optional format dependencies should remain feature-gated. Avoid introducing unconditional references to optional crates unless they are always enabled by the selected feature set.
- `BackupType` implementors should return bytes plus a stable file extension/name.
- Keep generated backup artifacts out of version control (for example `test_files/` or `test.yaml` from examples).

## Testing expectations

Before finishing changes, run at least:

```bash
cargo test
cargo fmt --all --check
```

If you changed the derive crate, also run its tests/formatting from `safekeep-derive/`. If you changed feature flags or serialization logic, run `cargo test --all-features`.
