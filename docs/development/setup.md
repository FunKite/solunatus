# Development Setup

Get your development environment ready for building Solunatus from source.

## Prerequisites

- **Latest stable Rust recommended** - [Install Rust](https://www.rust-lang.org/tools/install)
- **Minimum supported Rust version: 1.91**
- **Git** - Version control
- **Text editor or IDE** - VS Code recommended
- **macOS, Linux, or Windows** with development tools

Solunatus targets the latest stable Rust release for active development. The current release line supports stable Rust versions compatible with `rust-version = "1.91"`, and that floor may rise in a minor release when security, dependency compatibility, or maintainability require it. If you need an older Rust toolchain, use an older Solunatus release that still supports it.

## Installing Rust

```bash
# Official Rust installer (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then activate:
source "$HOME/.cargo/env"

# Verify installation
rustc --version
cargo --version
```

## Platform-Specific Setup

### macOS

```bash
# Install Xcode command line tools
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Linux (Ubuntu/Debian)

```bash
# Install build tools
sudo apt-get update
sudo apt-get install build-essential

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Linux (Fedora/RHEL)

```bash
# Install build tools
sudo dnf install gcc make

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Windows

1. Download [rustup-init.exe](https://win.rustup.rs/)
2. Run the installer and follow prompts
3. Ensure you have a C++ build chain (Visual Studio Community recommended)

## Clone Repository

```bash
# Clone the repository
git clone https://github.com/FunKite/solunatus.git
cd solunatus
```

## Build the Project

```bash
# Development build (faster to compile, slower to run)
cargo build

# Release build (slower to compile, optimized runtime)
cargo build --release

# Watch mode (auto-rebuild on file changes)
cargo watch

# Check without building (fastest)
cargo check
```

## Verify Installation

```bash
# Run CLI
cargo run --release -- --help

# Run with arguments
cargo run --release -- --city "New York" --no-prompt

# Run tests
cargo test

# Generate documentation
cargo doc --open
```

## IDE Setup

### VS Code (Recommended)

1. **Install Rust Analyzer:**
   - Command Palette (Cmd/Ctrl+Shift+P)
   - Search "Extensions: Install Extensions"
   - Search for "Rust Analyzer"
   - Install and reload

2. **Optional extensions:**
   - `Even Better TOML` - For Cargo.toml editing
   - `crates` - Dependency version checking
   - `CodeLLDB` - Debugger support

3. **Settings (.vscode/settings.json):**
   ```json
   {
     "[rust]": {
       "editor.formatOnSave": true,
       "editor.defaultFormatter": "rust-lang.rust-analyzer"
     },
     "rust-analyzer.checkOnSave.command": "clippy"
   }
   ```

### Other Editors

- **Vim/Neovim** - Use rust.vim plugin
- **Emacs** - Use rust-mode
- **IntelliJ IDEA** - Use Rust plugin
- **Sublime Text** - Use Rust Enhanced

## Code Quality Tools

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

```bash
# Run clippy (Rust linter)
cargo clippy -- -D warnings

# Fix clippy warnings automatically (when possible)
cargo clippy --fix --allow-dirty
```

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench
```

## Common Development Tasks

### Build for different architectures

```bash
# Current architecture (default)
cargo build --release

# Apple Silicon M1/M2/M3
cargo build --release --profile release-m1-max

# Intel x86_64 with AVX2
RUSTFLAGS='-C target-feature=+avx2' cargo build --release
```

### Check compilation without building

```bash
cargo check
```

### Generate documentation

```bash
# Open in browser
cargo doc --open

# Without opening
cargo doc
```

### Debug with println

```rust
// In your code
println!("Debug: {:?}", my_variable);
dbg!(my_variable);  // More convenient
```

### Run specific example

```bash
cargo run --example basic_usage
cargo run --example city_search
```

## Project Structure Reference

```
src/
├── lib.rs              # Library public API
├── main.rs             # CLI entry point
├── astro/              # Astronomical calculations
│   ├── mod.rs          # Common types
│   ├── sun.rs          # Solar calculations
│   └── moon.rs         # Lunar calculations
├── tui/                # Terminal UI
├── cli.rs              # CLI parsing
├── city.rs             # City database
├── config.rs           # Configuration
├── output.rs           # JSON output
└── time_sync.rs        # Clock checking

examples/               # Library examples
data/                   # Embedded data
docs/                   # User documentation
tests/                  # Integration tests
```

## Building for Distribution

```bash
# Build optimized release
cargo build --release

# Strip debug symbols for smaller binary
strip target/release/solunatus

# Install to system
sudo cp target/release/solunatus /usr/local/bin/
```

## Running Tests

```bash
# All tests
cargo test

# Specific module
cargo test astro::sun

# With output
cargo test -- --nocapture

# Benchmark performance
cargo bench
```

## Troubleshooting

### "error: linker `cc` not found"
Install C compiler and build tools (see platform-specific setup above).

### "cargo: command not found"
Rust not installed or PATH not updated. Run: `source "$HOME/.cargo/env"`

### Tests fail on first run
Some tests may require network access for NTP time sync or USNO validation. Use:
```bash
SOLUNATUS_SKIP_TIME_SYNC=1 cargo test
```

### Out of memory during build
Use incremental compilation:
```bash
cargo build -j 1
```

## Next Steps

- **[Architecture Guide](architecture.md)** - Code structure overview
- **[Accuracy Testing](accuracy.md)** - Verify calculation accuracy
- **[API Documentation](https://docs.rs/solunatus)** - Auto-generated docs

## Getting Help

- **[GitHub Issues](https://github.com/FunKite/solunatus/issues)** - Report bugs or request features
- **[Rust Book](https://doc.rust-lang.org/book/)** - Learn Rust
- **[Cargo Guide](https://doc.rust-lang.org/cargo/)** - Learn Cargo

Happy coding! 🚀
