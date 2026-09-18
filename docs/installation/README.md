# Install Solunatus

## Install the published release

With [Rust and Cargo](https://www.rust-lang.org/tools/install) installed:

```bash
cargo install --locked solunatus
solunatus --version
solunatus --city "Tucson"
```

Use the latest stable Rust toolchain when possible. The current minimum is Rust 1.91; it may rise in a future minor release for security or compatibility.

To reinstall or upgrade:

```bash
cargo install --locked solunatus --force
```

## Historical Linux downloads

v0.7.0 is distributed through Cargo; no prebuilt binaries are attached. The older [v0.6.1](https://github.com/FunKite/solunatus/releases/tag/v0.6.1) provides two prebuilt archives, which do not include `--night`:

- `solunatus-v0.6.1-linux-x86_64.tar.gz`
- `solunatus-v0.6.1-linux-aarch64.tar.gz`

Download the archive for your CPU and `solunatus-v0.6.1-SHA256SUMS.txt` from that release. In the download directory, verify the matching archive on Linux:

```bash
sha256sum --check --ignore-missing solunatus-v0.6.1-SHA256SUMS.txt
```

Proceed only if your downloaded archive is listed as `OK`. Extract the archive and run its `solunatus` executable. See the release notes for the supported assets; unsigned macOS and Windows binaries are not provided in this release, so use Cargo there.

## Build the current source

Use this method to work with the current development source:

```bash
git clone https://github.com/FunKite/solunatus.git
cd solunatus
cargo install --locked --path .
solunatus --city "Tucson" --night
```

To build without installing:

```bash
cargo build --release --locked
./target/release/solunatus --city "Tucson" --night
```

## Optional integrations

USNO validation and AI insights are included by default. The dashboard and core astronomy need neither integration. To omit them:

```bash
# Published release
cargo install --locked solunatus --no-default-features

# Current source
cargo install --locked --path . --no-default-features
```

The default time check is independent of those features. Set `SOLUNATUS_SKIP_TIME_SYNC=1` for an offline dashboard; `--night` and `--next` are already offline.

## Use the library

```toml
[dependencies]
solunatus = "0.7.0"
chrono = "0.4"
chrono-tz = "0.10"
```

See the [API docs](https://docs.rs/solunatus) and [examples](../../examples/).

## Help

- [Quick start](quick-start.md)
- [Observing guide](../features/observing.md)
- [Troubleshooting](troubleshooting.md)
