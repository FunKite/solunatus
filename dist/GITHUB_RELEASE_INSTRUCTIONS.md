# GitHub Release Instructions

This file now documents the current Solunatus GitHub release flow.

## Current Policy

- Publish the crate to crates.io first.
- Create and push the matching Git tag after a successful publish.
- Create a GitHub Release from that tag with curated release notes.
- Do not attach binary artifacts unless that specific release intentionally includes a packaging step.

## Release Sequence

1. Confirm `Cargo.toml`, `CHANGELOG.md`, and release-facing docs are finalized.
2. Run local validation and `cargo publish --dry-run`.
3. Run `cargo publish`.
4. Create and push the release tag, for example `v0.4.0`.
5. Create the GitHub Release with notes derived from the finalized changelog entry.

## GitHub Web Flow

1. Go to [GitHub Releases](https://github.com/FunKite/solunatus/releases).
2. Click **Draft a new release**.
3. Select the pushed release tag.
4. Use the version number as the release title, for example `v0.4.0`.
5. Paste or adapt the release notes from `dist/RELEASE_NOTES.md`.
6. Publish the release without attaching binaries unless that release explicitly includes them.

## GitHub CLI Flow

```bash
gh release create v0.4.0 \
  --title "v0.4.0" \
  --notes-file dist/RELEASE_NOTES.md
```

## Historical Assets

The `dist/` directory still contains older packaging artifacts from early binary-distribution experiments. Treat those files as historical only; they are not part of the default modern release process.
