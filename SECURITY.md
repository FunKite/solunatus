# Security Policy

## Supported Versions

**Only the latest release receives security patches.** We recommend always running the most recent version.

| Version | Supported          |
| ------- | ------------------ |
| Latest  | :white_check_mark: |
| Older   | :x:                |

To check for updates:
```bash
cargo install solunatus --force
```

## Reporting a Vulnerability

We take the security of Solunatus seriously. If you discover a security vulnerability, please report it responsibly.

### How to Report

**For security vulnerabilities, please DO NOT open a public issue.**

Instead, please report security issues using GitHub's private security advisory feature:

1. Go to https://github.com/FunKite/solunatus/security/advisories
2. Click "New draft security advisory"
3. Fill in the details:
   - A description of the vulnerability
   - Steps to reproduce the issue
   - Potential impact
   - Any suggested fixes (optional)

### What to Expect

- **Response time**: You should receive an acknowledgment within 48 hours
- **Updates**: We'll keep you informed about the progress of fixing the vulnerability
- **Credit**: If you'd like, we'll credit you in the release notes for responsible disclosure
- **Timeline**: We aim to release patches for confirmed vulnerabilities within 7-14 days

### Security Update Process

Once a vulnerability is confirmed:

1. We'll develop and test a fix
2. Create a new patch release (e.g., 0.1.2)
3. Publish the fix to:
   - GitHub releases
   - crates.io
4. Publish a security advisory with:
   - Description of the vulnerability
   - Affected versions
   - Upgrade instructions
   - Credit to the reporter (if desired)

## Security Best Practices for Users

### Installation

Always install from trusted, official sources:

```bash
# Recommended: Install from crates.io
cargo install solunatus

# Alternative: Install from official GitHub repository
cargo install --git https://github.com/FunKite/solunatus.git --tag v0.2.2

# Or download pre-built binaries from official GitHub releases
# https://github.com/FunKite/solunatus/releases
```

**Warning**: Never install from unofficial mirrors, forks, or third-party package repositories unless you've verified the source code yourself.

### Verify Checksums

When downloading binaries from GitHub releases, always verify checksums to ensure integrity:

```bash
# Download the binary and checksum file
curl -LO https://github.com/FunKite/solunatus/releases/download/v0.2.2/solunatus-v0.2.2-linux-x86_64.tar.gz
curl -LO https://github.com/FunKite/solunatus/releases/download/v0.2.2/solunatus-v0.2.2-linux-x86_64.tar.gz.sha256

# Verify checksum (Linux/macOS)
sha256sum -c solunatus-v0.2.2-linux-x86_64.tar.gz.sha256

# On macOS, you can also use:
shasum -a 256 -c solunatus-v0.2.2-macos-universal.tar.gz.sha256

# Windows PowerShell
Get-FileHash solunatus-v0.2.2-windows-x86_64.zip -Algorithm SHA256
```

If the checksum doesn't match, **do not run the binary** and report it immediately.

### Verify Source Code Before Building

When building from source, verify the repository and review changes:

```bash
# Clone from official repository
git clone https://github.com/FunKite/solunatus.git
cd solunatus

# Verify you're on the official repository
git remote -v

# Checkout a specific release tag
git checkout v0.2.2

# Review the source code before building
# Especially check build.rs and any procedural macros
```

### Keep Your Installation Updated

Security patches are released promptly. Stay updated:

```bash
# Check your current version
solunatus --version

# Update to the latest version
cargo install solunatus --force

# Subscribe to security advisories
# Visit: https://github.com/FunKite/solunatus/security/advisories
```

### Sandboxing and Isolation

For additional security, consider running Solunatus in isolated environments:

```bash
# Run in Docker (if you create your own Dockerfile)
docker run --rm -it solunatus-container solunatus --city "New York"

# Use firejail for sandboxing (Linux)
firejail --net=none solunatus --city "Boston" --no-prompt

# macOS sandbox (for downloaded binaries)
# System will automatically prompt for permissions
```

### Least Privilege Principle

Solunatus doesn't require elevated privileges:

```bash
# Never run with sudo/root (unnecessary and dangerous)
# BAD: sudo solunatus
# GOOD: solunatus --city "Tokyo"
```

## Known Security Considerations

### Configuration File

Solunatus stores your location preferences in `~/.astro_times.json`. This file contains:
- Latitude/longitude coordinates
- Timezone information
- City name (if selected)

**Privacy note**: This information is stored locally and never transmitted over the network.

### Network Requests

Solunatus makes network requests for:

- **NTP time synchronization**: Queries time.google.com or pool.ntp.org
  - Purpose: Detect system clock drift
  - Frequency: Cached for 30 minutes
  - Data sent: Standard SNTP request (no personal information)

- **USNO validation** (optional): Queries aa.usno.navy.mil
  - Only when using `--validate` or pressing 'r' in watch mode
  - Purpose: Accuracy verification against U.S. Naval Observatory data

**No location data or personal information is ever transmitted.**

### Dependencies

We regularly update dependencies to address security vulnerabilities. To check for vulnerabilities in the current version:

```bash
# Install cargo-audit
cargo install cargo-audit

# Check for vulnerabilities
cargo audit
```

## Dependency Management

### Automated Updates

- **Dependabot**: Automatically monitors dependencies for security vulnerabilities
- **Dependabot Security Updates**: Automatically creates PRs for security patches

### Manual Review

All dependency updates are reviewed before merging, with special attention to:
- Breaking changes
- New permissions or capabilities
- Upstream security track record

## Scope

This security policy covers:
- ✅ The Solunatus CLI application
- ✅ The Solunatus Rust library (published on crates.io)
- ✅ Build scripts and release binaries
- ✅ Dependencies with known vulnerabilities

This policy does NOT cover:
- ❌ Third-party dependencies (report directly to their maintainers)
- ❌ Issues with Rust toolchain or cargo (report to rust-lang)
- ❌ Operating system or terminal emulator issues

## Security Features

### Current Protections

#### Core Security
- **No remote code execution**: All astronomical calculations are performed locally using pure Rust
- **No data collection**: Zero telemetry, analytics, or usage tracking of any kind
- **Minimal attack surface**: Single-purpose CLI tool with no web server, network listening, or background services
- **Memory safety**: Written in Rust for guaranteed memory safety (no buffer overflows, use-after-free, etc.)
- **No unsafe code in calculations**: Critical astronomical algorithms use only safe Rust
- **Dependency scanning**: Automated vulnerability detection via Dependabot and `cargo audit`

#### Network Security
- **TLS verification**: All HTTPS connections enforce certificate validation
- **Minimal network use**: Only optional NTP sync and USNO validation
- **No automatic updates**: User controls when to update
- **Transparent network requests**: All network activity is documented and optional

#### Data Privacy
- **Local-only storage**: Configuration stored in user's home directory
- **No cloud sync**: All data remains on your device
- **No user tracking**: No unique identifiers, session IDs, or analytics
- **No external APIs**: Except explicitly requested NTP/USNO validation

#### Build Security
- **Reproducible builds**: Same source + toolchain = same binary (work in progress)
- **No build-time code generation**: No procedural macros that execute arbitrary code
- **Minimal build dependencies**: Reduces supply chain attack surface
- **Version pinning**: Dependencies locked to specific versions in Cargo.lock

### Verified Security Practices

- ✅ No `unsafe` code in critical paths (solar/lunar calculations)
- ✅ Input validation on all user-provided data
- ✅ Bounds checking on array access
- ✅ Safe string handling (no buffer overflows)
- ✅ Secure HTTP client configuration
- ✅ Error handling without information leakage
- ✅ No shell command execution with user input
- ✅ Configuration file permissions checked

### Future Enhancements

We're considering these additional security measures:

#### Short-term (Next Release)
- Enhanced configuration file permission warnings
- Optional strict mode for paranoid users
- Additional input validation hardening

#### Medium-term
- Code signing for macOS binaries (to avoid Gatekeeper warnings)
- Notarization for macOS releases
- Windows binary signing with Authenticode
- Reproducible builds for complete verification

#### Long-term
- Supply chain security with cargo-vet
- SBOM (Software Bill of Materials) generation
- Security audit by third-party firm
- Formal verification of critical algorithms

## Questions?

If you have questions about security that don't involve reporting a vulnerability, feel free to:
- Open a discussion on GitHub
- Open a regular issue with the "question" label

---

**Last Updated**: 2025-01-14
**Maintainer**: @FunKite
