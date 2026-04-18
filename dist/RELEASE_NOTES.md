# Solunatus v0.4.0 Release Notes

Release date: 2026-04-18

## Summary

This release promotes the current unreleased work into the `0.4.0` line. It includes CLI behavior fixes, USNO validation hardening, dependency and security updates, and release-policy documentation cleanup.

## Highlights

- Non-watch runs now honor saved time-sync settings and persist explicit `--city` / `--lat` / `--lon` changes back to the user config.
- USNO validation now reuses the primary day fetch and fails surrounding-day retries faster during API outages.
- The Rust support contract is now explicit: latest stable remains the active development target and the release line supports stable Rust `1.91+`.
- Security and dependency updates include the current `rand`, `rustls-webpki`, `quinn-proto`, `chrono`, `clap`, and `anyhow` bumps captured in the changelog.

## Install Or Upgrade

```bash
cargo install solunatus --force
```

If you use Solunatus as a library:

```toml
[dependencies]
solunatus = "0.4.0"
chrono = "0.4"
chrono-tz = "0.10"
```

## Release Notes Source

The canonical per-release details should match the `CHANGELOG.md` entry for `0.4.0`. GitHub Releases for the default flow are tags plus notes only; do not imply binary attachments unless the release explicitly includes them.
