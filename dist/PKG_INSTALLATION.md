# PKG Installer Notes

This file is retained for historical context only.

## Current Release Policy

Solunatus does not currently treat macOS PKG installers as part of the default release flow. The standard modern release path is:

1. Publish the crate on crates.io.
2. Tag the matching version in GitHub.
3. Create a GitHub Release with notes only unless a release explicitly includes packaged artifacts.

## Recommended Installation Paths

Install or upgrade from crates.io:

```bash
cargo install solunatus --force
```

Or build from source:

```bash
git clone https://github.com/FunKite/solunatus.git
cd solunatus
cargo install --path .
```

## Historical Artifacts

Older PKG and tarball assets under `dist/` were produced for earlier experiments and should not be treated as the supported install path for current releases.
