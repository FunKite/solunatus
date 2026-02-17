/// CPU Feature Detection and Management
///
/// This module provides runtime and compile-time CPU feature detection
/// with automatic fallback to portable implementations for maximum compatibility.
///
/// Tier 1 Supported Platforms:
/// - x86_64: Baseline (SSE2), AVX2 when available
/// - aarch64: Baseline (NEON), SVE when available
/// - Darwin (Apple Silicon): ARM64 with NEON
///
/// Feature Detection Strategy:
/// 1. Compile-time: Use RUSTFLAGS or .cargo/config.toml
/// 2. Runtime: Detect available CPU features
/// 3. Graceful fallback: Use portable scalar implementations
///
/// CPU feature flags for compile-time detection
pub mod compile_time {
    /// Check if we're compiling for x86_64
    pub const IS_X86_64: bool = cfg!(target_arch = "x86_64");

    /// Check if we're compiling for aarch64 (ARM64)
    pub const IS_AARCH64: bool = cfg!(target_arch = "aarch64");

    /// Check if we're compiling for macOS
    pub const IS_MACOS: bool = cfg!(target_os = "macos");

    /// Check if we're compiling for Linux
    pub const IS_LINUX: bool = cfg!(target_os = "linux");

    /// Check if we're compiling for Windows
    pub const IS_WINDOWS: bool = cfg!(target_os = "windows");

    /// AVX2 available at compile time (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    pub const HAS_AVX2: bool = cfg!(target_feature = "avx2");

    #[cfg(not(target_arch = "x86_64"))]
    pub const HAS_AVX2: bool = false;

    /// AVX512F available at compile time (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    pub const HAS_AVX512F: bool = cfg!(target_feature = "avx512f");

    #[cfg(not(target_arch = "x86_64"))]
    pub const HAS_AVX512F: bool = false;

    /// SSE4.2 available at compile time (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    pub const HAS_SSE42: bool = cfg!(target_feature = "sse4.2");

    #[cfg(not(target_arch = "x86_64"))]
    pub const HAS_SSE42: bool = false;

    /// NEON available at compile time (aarch64 only)
    #[cfg(target_arch = "aarch64")]
    pub const HAS_NEON: bool = cfg!(target_feature = "neon");

    #[cfg(not(target_arch = "aarch64"))]
    pub const HAS_NEON: bool = false;

    /// SVE (Scalable Vector Extension) available (aarch64 only)
    #[cfg(target_arch = "aarch64")]
    pub const HAS_SVE: bool = cfg!(target_feature = "sve");

    #[cfg(not(target_arch = "aarch64"))]
    pub const HAS_SVE: bool = false;

    /// SIMD available at compile time
    pub const HAS_SIMD: bool = HAS_AVX2 || HAS_NEON || HAS_SVE;

    /// Get a human-readable CPU feature summary
    pub fn cpu_features_summary() -> String {
        let mut features = vec![];

        if IS_X86_64 {
            features.push("x86_64".to_string());
            if HAS_AVX512F {
                features.push("AVX-512F".to_string());
            }
            if HAS_AVX2 {
                features.push("AVX2".to_string());
            }
            if HAS_SSE42 {
                features.push("SSE4.2".to_string());
            }
        } else if IS_AARCH64 {
            features.push("ARM64".to_string());
            if HAS_SVE {
                features.push("SVE".to_string());
            }
            if HAS_NEON {
                features.push("NEON".to_string());
            }
        }

        if IS_MACOS {
            features.push("macOS".to_string());
        } else if IS_LINUX {
            features.push("Linux".to_string());
        } else if IS_WINDOWS {
            features.push("Windows".to_string());
        }

        format!("[{}]", features.join(", "))
    }
}

/// Runtime CPU feature detection
///
/// Note: This is primarily informational. For actual optimization,
/// use compile-time detection and RUSTFLAGS/cargo config.
pub mod runtime {
    /// Detect AVX2 support at runtime (x86_64 only)
    #[cfg(target_arch = "x86_64")]
    pub fn has_avx2() -> bool {
        // Use CPUID instruction via inline assembly or extern crate
        // For now, return compile-time value as fallback
        super::compile_time::HAS_AVX2
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn has_avx2() -> bool {
        false
    }

    /// Detect NEON support at runtime (aarch64 only)
    #[cfg(target_arch = "aarch64")]
    pub fn has_neon() -> bool {
        // ARM64 always has NEON support
        true
    }

    #[cfg(not(target_arch = "aarch64"))]
    pub fn has_neon() -> bool {
        false
    }

    /// Get CPU core count
    pub fn cpu_count() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }

    /// Get logical CPU core count (including hyperthreads)
    pub fn logical_cpu_count() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }

    /// Get physical CPU core count (excluding hyperthreads)
    /// Note: std library doesn't distinguish physical vs logical cores,
    /// so this returns the same as logical_cpu_count()
    pub fn physical_cpu_count() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
}

/// Optimization selection based on CPU features
#[derive(Debug, Clone)]
pub struct OptimizationProfile {
    pub name: &'static str,
    pub has_avx2: bool,
    pub has_neon: bool,
    pub has_sve: bool,
    pub has_avx512: bool,
    pub parallelism: usize,
    pub simd_width: usize,
}

impl OptimizationProfile {
    /// Select the best optimization profile for current CPU
    pub fn current() -> Self {
        let parallelism = runtime::physical_cpu_count();

        #[cfg(all(target_arch = "x86_64", target_feature = "avx512f"))]
        return Self {
            name: "x86_64-AVX-512",
            has_avx2: true,
            has_neon: false,
            has_sve: false,
            has_avx512: true,
            parallelism,
            simd_width: 16, // 512-bit / 32-bit = 16 floats
        };

        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            not(target_feature = "avx512f")
        ))]
        return Self {
            name: "x86_64-AVX2",
            has_avx2: true,
            has_neon: false,
            has_sve: false,
            has_avx512: false,
            parallelism,
            simd_width: 8, // 256-bit / 32-bit = 8 floats
        };

        #[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
        return Self {
            name: "x86_64-Portable",
            has_avx2: false,
            has_neon: false,
            has_sve: false,
            has_avx512: false,
            parallelism,
            simd_width: 1,
        };

        #[cfg(all(target_arch = "aarch64", target_feature = "sve"))]
        return Self {
            name: "ARM64-SVE",
            has_avx2: false,
            has_neon: true,
            has_sve: true,
            has_avx512: false,
            parallelism,
            simd_width: 4, // SVE can be wider, but 4 is conservative
        };

        #[cfg(all(target_arch = "aarch64", not(target_feature = "sve")))]
        return Self {
            name: "ARM64-NEON",
            has_avx2: false,
            has_neon: true,
            has_sve: false,
            has_avx512: false,
            parallelism,
            simd_width: 4, // 128-bit / 32-bit = 4 floats
        };

        // Fallback for unsupported architectures
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        Self {
            name: "Portable",
            has_avx2: false,
            has_neon: false,
            has_sve: false,
            has_avx512: false,
            parallelism,
            simd_width: 1,
        }
    }

    /// Check if SIMD optimization is available
    pub fn has_simd(&self) -> bool {
        self.has_avx512 || self.has_avx2 || self.has_neon || self.has_sve
    }

    /// Get profile description
    pub fn description(&self) -> String {
        format!(
            "Optimization Profile: {} (parallelism={}, SIMD={})",
            self.name,
            self.parallelism,
            if self.has_simd() {
                "enabled"
            } else {
                "disabled"
            }
        )
    }
}

/// Conditionally enable portable implementations
pub mod portable {
    /// Portable sine calculation (used when SIMD not available)
    #[inline]
    pub fn sin(x: f64) -> f64 {
        x.sin()
    }

    /// Portable cosine calculation (used when SIMD not available)
    #[inline]
    pub fn cos(x: f64) -> f64 {
        x.cos()
    }

    /// Portable atan2 calculation (used when SIMD not available)
    #[inline]
    pub fn atan2(y: f64, x: f64) -> f64 {
        y.atan2(x)
    }

    /// Portable sqrt calculation (used when SIMD not available)
    #[inline]
    pub fn sqrt(x: f64) -> f64 {
        x.sqrt()
    }
}

/// Get recommended build flags for current platform
pub fn recommended_build_flags() -> &'static str {
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    return "RUSTFLAGS='-C target-cpu=haswell -C llvm-args=-x86-asm-syntax=intel'";

    #[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
    return "RUSTFLAGS='-C target-cpu=x86-64'";

    #[cfg(target_arch = "aarch64")]
    return "RUSTFLAGS='-C target-cpu=apple-m1' # for Apple Silicon";

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    "Default settings (portable)"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_features_summary() {
        let summary = compile_time::cpu_features_summary();
        assert!(!summary.is_empty());
        println!("CPU Features: {}", summary);
    }

    #[test]
    fn test_optimization_profile() {
        let profile = OptimizationProfile::current();
        println!("Active Profile: {}", profile.description());
        assert!(profile.parallelism > 0);
        assert!(profile.simd_width > 0);
    }

    #[test]
    fn test_compile_time_detection() {
        let is_x86 = compile_time::IS_X86_64;
        let is_arm = compile_time::IS_AARCH64;
        assert!(is_x86 || is_arm); // At least one must be true
    }
}
