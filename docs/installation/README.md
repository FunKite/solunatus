# Installation Guide

Choose the installation method that works best for you.

## Quick Start (5 minutes)

For the fastest way to get started:

```bash
# Clone and build from source
git clone https://github.com/FunKite/solunatus.git
cd solunatus
cargo build --release
./target/release/solunatus --help
```

See [Quick Start Guide](quick-start.md) for detailed steps.

## Installation Methods

### 1. Build from Source (Recommended for Developers)

**Requirements:**
- Latest stable Rust recommended
- Minimum supported Rust version: 1.91
- Cargo (included with Rust)
- Git

Solunatus targets the latest stable Rust release for active development. The current release line supports stable Rust versions compatible with `rust-version = "1.91"`, and that floor may rise in a minor release when security, dependency compatibility, or maintainability require it. If you need an older Rust toolchain, use an older Solunatus release that still supports it.

**Steps:**

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/FunKite/solunatus.git
cd solunatus

# Build release binary
cargo build --release

# Install system-wide (optional)
cargo install --path .

# Or run directly
./target/release/solunatus --city "New York"
```

### 2. Install from Crates.io (Recommended)

Install the CLI tool directly from crates.io:

```bash
cargo install solunatus
```

#### Using as a Library

If you want to use Solunatus as a library in your Rust project:

```toml
[dependencies]
solunatus = "0.2"
chrono = "0.4"
chrono-tz = "0.10"
```

See the [examples directory](../../examples/) for usage patterns.

### 3. Download Pre-built Binary (Linux)

Pre-built binaries for Linux are available on the [GitHub Releases](https://github.com/FunKite/solunatus/releases) page.

Check the releases page for available architectures and download instructions.

## Platform Support

### Tier 1 (Fully Tested)
- **macOS** (Intel and Apple Silicon) - Primary development platform
- **Linux** (x86_64) - Tested on Ubuntu 20.04+

### Tier 2 (Builds Successfully)
- **Windows** (x86_64 via WSL or native)
- **macOS ARM64** (Apple Silicon M1/M2/M3)

### Tier 3 (Community Supported)
- Other Linux distributions
- Other Unix-like systems

## Verification

After installation, verify it works:

```bash
# Show help
solunatus --help

# Get sunrise/sunset for your location
solunatus --city "New York"

# Output JSON
solunatus --city "Tokyo" --json

# View moon phases for a month
solunatus --city "Paris" --calendar --calendar-start 2025-12-01 --calendar-end 2025-12-31
```

## Troubleshooting

Having issues? See the [Troubleshooting Guide](troubleshooting.md).

## Next Steps

- **[Quick Start Guide](quick-start.md)** - Learn basic usage
- **[CLI Reference](../features/cli-reference.md)** - Complete command options
- **[Interactive Mode Guide](../features/interactive-mode.md)** - Master the TUI
- **[Contributing](../../CONTRIBUTING.md)** - Help improve Solunatus
