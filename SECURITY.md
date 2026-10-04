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
cargo install --locked solunatus

# Alternative: Install from the official GitHub repository
cargo install --locked --git https://github.com/FunKite/solunatus.git --tag v0.7.0

# Or download prebuilt archives from official GitHub releases
# https://github.com/FunKite/solunatus/releases
```

**Warning**: Never install from unofficial mirrors, forks, or third-party package repositories unless you've verified the source code yourself.

### Verify Checksums

The v0.7.0 release is distributed through Cargo; no prebuilt binaries are attached. Cargo verifies downloaded registry packages against the SHA-256 checksums in the [registry index](https://doc.rust-lang.org/cargo/reference/registry-index.html), not publisher signatures. The older [v0.6.1](https://github.com/FunKite/solunatus/releases/tag/v0.6.1) release still provides prebuilt Linux archives (no `--night` support), which can be checksum-verified before use:

```bash
# Download the archive and its checksum file from the v0.6.1 release
curl -LO https://github.com/FunKite/solunatus/releases/download/v0.6.1/solunatus-v0.6.1-linux-x86_64.tar.gz
curl -LO https://github.com/FunKite/solunatus/releases/download/v0.6.1/solunatus-v0.6.1-SHA256SUMS.txt

# Verify checksum on Linux
sha256sum --check --ignore-missing solunatus-v0.6.1-SHA256SUMS.txt

# On macOS, use:
shasum -a 256 --check --ignore-missing solunatus-v0.6.1-SHA256SUMS.txt
```

Proceed only if the archive is listed as `OK`. Unsigned macOS and Windows binaries are not provided; use Cargo on those platforms. See the [installation guide](docs/installation/README.md) for current details.

### Verify Source Code Before Building

When building from source, verify the repository and review changes:

```bash
# Clone from official repository
git clone https://github.com/FunKite/solunatus.git
cd solunatus

# Verify you're on the official repository
git remote -v

# Checkout a specific release tag
git checkout v0.7.0

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

Solunatus stores your location preferences in `~/.solunatus.json`. This file contains:
- Latitude/longitude coordinates
- Timezone information
- City name (if selected)
- AI insights server/model settings, if configured

**Privacy note**: The configuration file is stored locally. On Unix systems each save writes a new file with `0600` permissions and atomically replaces the old file, so a reader holding an old descriptor cannot see newly saved settings. Existing files from older versions are replaced on their next successful save. Optional network features can transmit location data as described below.

### Network Requests

Solunatus makes network requests for:

- **NTP time synchronization**: Queries time.google.com or pool.ntp.org
  - Purpose: Detect system clock drift
  - Frequency: Cached for 30 minutes
  - Data sent: Standard SNTP request (no personal information)

- **USNO validation** (optional): Queries aa.usno.navy.mil
  - Only when using `--validate` or pressing 'r' in watch mode
  - Purpose: Accuracy verification against U.S. Naval Observatory data
  - Data sent: Latitude, longitude, and date

- **AI insights** (optional): Sends requests to the configured Ollama server
  - Only when AI insights are enabled
  - Default server: `http://localhost:11434`; a configured remote server receives the same data
  - Data sent: Latitude, longitude, optional city name, and astronomical context

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
- ✅ Configuration file restricted to owner-only permissions (`0600`) on Unix

### Future Enhancements

We're considering these additional security measures:

#### Short-term (Next Release)
- Windows/macOS ACL hardening for the configuration file (currently `0600`-restricted on Unix only)
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

**Last Updated**: 2026-09-29
**Maintainer**: @FunKite
