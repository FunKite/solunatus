# Solunatus release procedure

Release from a reviewed PR merged into `main`. Keep `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, README, installation and feature guides, preview image, and `dist/RELEASE_NOTES.md` consistent. The dashboard title and CLI version use `CARGO_PKG_VERSION`; verify the built app displays the new version.

## Validate the release candidate

1. Check current GitHub and crates.io versions; choose an unused version.
2. Run `./scripts/safe_local_test.sh` (add `--allow-network` only if a dependency or advisory refresh is needed).
3. Run formatting, Clippy, all-feature and no-default-feature tests, and the MSRV checks in CI.
4. Build documentation with `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features`, and repeat with `--no-default-features`. Public library items must remain documented; do not suppress missing-documentation errors or hide public APIs to improve the percentage.
5. Regenerate the preview with `python3 scripts/render_night_demo.py --binary target/debug/solunatus` after building. Smoke-test `--night`, its `--tonight` alias, help, completions, and the man page.
6. Review package contents with `cargo package --locked --list`. Exclude local settings and old build artifacts.
7. Merge only after all PR checks and review findings are resolved, then fast-forward local `main` to the merged commit.

## Publish, then tag

From the clean committed release tree (substitute the new version below):

```bash
cargo publish --locked --dry-run
cargo package --locked
shasum -a 256 target/package/solunatus-0.7.0.crate
cargo publish --locked
git tag -a v0.7.0 -m "Solunatus 0.7.0"
git push origin v0.7.0
gh release create v0.7.0 --verify-tag \
  --title "Solunatus 0.7.0 — Plan any night" \
  --notes-file dist/RELEASE_NOTES.md
```

`cargo package --locked` retains the verified archive at the checksum path shown above; some Cargo versions keep publish dry-run archives under `target/package/tmp-crate/` instead.

The successful dry run must precede the real upload. Do not change source between them. Confirm the crates.io version and registry checksum match the tested archive before creating the tag. Never reuse a published version or move a published release tag.

After publishing, verify the GitHub tag points to the published commit and docs.rs builds the exact version with **100% public API documentation coverage**. Check the installed executable's path and `--version` if upgrading a local installation, since an older executable may shadow it in `PATH`.

## Release assets

The default release consists of the crate, tag, and curated notes. Attach binaries only when a release intentionally includes a validated packaging step. Build release binaries fresh for each version (outside git; `dist/*.tar.gz` and checksum files are ignored) and never attach binaries from an earlier version.
