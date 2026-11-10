# Solunatus release procedure

Release from a reviewed PR merged into `main`. Keep `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, README, installation and feature guides, preview image, and `dist/RELEASE_NOTES.md` consistent. The dashboard title and CLI version use `CARGO_PKG_VERSION`; verify the built app displays the new version.

## Validate the release candidate

1. Check current GitHub and crates.io versions; choose an unused version.
2. Run `./scripts/safe_local_test.sh` (add `--allow-network` only if a dependency or advisory refresh is needed).
3. Run formatting, Clippy, all-feature and no-default-feature tests, and the MSRV checks in CI.
4. Build documentation with `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --all-features`, and repeat with `--no-default-features`. Public library items must remain documented; do not suppress missing-documentation errors or hide public APIs to improve the percentage.
5. Measure actual public-item documentation coverage with `cargo +nightly rustdoc --locked --lib --all-features -- -Z unstable-options --show-coverage` and record the documented/total count. It must be 100%; do not infer the metric from a successful build.
6. Regenerate the preview with `python3 scripts/render_night_demo.py --binary target/debug/solunatus` after building. Smoke-test `--night`, its `--tonight` alias, help, completions, and the man page.
7. Review package contents with `cargo package --locked --list`. Exclude local settings and old build artifacts.
8. Merge only after all PR checks and review findings are resolved, then fast-forward local `main` to the merged commit.

## Publish from GitHub (Trusted Publishing)

Releases publish from `.github/workflows/release.yml` when the owner pushes a version tag. crates.io trusts that workflow through [Trusted Publishing](https://crates.io/docs/trusted-publishing), so no crates.io token is stored anywhere. The workflow checks that the tag matches the `Cargo.toml` version, that `dist/RELEASE_NOTES.md` and `CHANGELOG.md` cover it and that the tagged commit is on `main`, then runs `cargo publish --locked --dry-run`, publishes, and creates the GitHub release from `dist/RELEASE_NOTES.md`.

One-time setup on crates.io (owner only): open the `solunatus` crate's **Settings**, then **Trusted Publishing**, add a GitHub publisher with owner `FunKite`, repository `solunatus`, workflow `release.yml` and environment `release`. Optionally add required reviewers to the `release` environment in the repository settings so each publish waits for approval.

After the release PR is merged and local `main` is up to date (substitute the new version):

```bash
git tag -a v0.8.1 -m "Solunatus 0.8.1"
git push origin v0.8.1
```

Watch the **Release** workflow run. If the version check or dry run fails, nothing is published; delete the tag (`git push origin :refs/tags/v0.8.1`), fix the release PR and tag again. Once crates.io has the version, never move or reuse the tag.

### Manual fallback

From a machine logged in to crates.io, on the clean committed release tree:

```bash
cargo publish --locked --dry-run
cargo package --locked
shasum -a 256 target/package/solunatus-0.8.1.crate
cargo publish --locked
git tag -a v0.8.1 -m "Solunatus 0.8.1"
git push origin v0.8.1
```

Pushing the tag also starts the Release workflow, whose publish job then fails because the version already exists; create the GitHub release by hand instead:

```bash
gh release create v0.8.1 --verify-tag \
  --title "Solunatus 0.8.1 — Accuracy and reliability fixes" \
  --notes-file dist/RELEASE_NOTES.md
```

`cargo package --locked` retains the verified archive at the checksum path shown above; some Cargo versions keep publish dry-run archives under `target/package/tmp-crate/` instead.

After publishing either way, verify the GitHub tag points to the published commit and docs.rs builds the exact version with **100% public API documentation coverage**. Check the installed executable's path and `--version` if upgrading a local installation, since an older executable may shadow it in `PATH`.

## Release assets

The default release consists of the crate, tag, and curated notes. Attach binaries only when a release intentionally includes a validated packaging step. Build release binaries fresh for each version (outside git; `dist/*.tar.gz` and checksum files are ignored) and never attach binaries from an earlier version.
