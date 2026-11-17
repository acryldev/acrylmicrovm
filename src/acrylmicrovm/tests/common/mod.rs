// Modified by ACRYL, 2026
//! Shared test infrastructure for acrylmicrovm integration tests.
//!
//! Runtime setup:
//! - [`PerTestBoxHome::new()`]: Per-test home with image cache (for VM tests).
//! - [`PerTestBoxHome::isolated()`]: Per-test home without cache (for non-VM tests).
//!
//! Helper functions:
//! - [`alpine_opts()`]: Default `BoxOptions` with `alpine:latest`, `auto_delete=0`
//! - [`alpine_opts_auto()`]: Same but `auto_delete>0`

#![allow(dead_code)]

// Re-export shared infrastructure from acrylmicrovm-test-utils.
pub use acrylmicrovm_test_utils::*;

use acrylmicrovm::runtime::options::{AcrylMicrovmOptions, BoxOptions, RootfsSpec};

/// Non-VM suites use the existing test constructor on hosted runners. VM
/// integration builds retain the public constructor and its host validation.
pub fn non_vm_runtime(
    options: AcrylMicrovmOptions,
) -> acrylmicrovm::AcrylMicrovmResult<TestRuntime> {
    if std::env::var("ACRYLMICROVM_TEST_ENGINE").as_deref() == Ok("native") {
        return TestRuntime::new(options);
    }
    #[cfg(feature = "test-support")]
    {
        acrylmicrovm::AcrylMicrovmRuntime::new_for_test(options).map(TestRuntime::from_runtime)
    }
    #[cfg(not(feature = "test-support"))]
    {
        TestRuntime::new(options)
    }
}

// ============================================================================
// BOX OPTIONS HELPERS
// ============================================================================

/// Default test box options: `alpine:latest`, `auto_delete=0`.
pub fn alpine_opts() -> BoxOptions {
    BoxOptions {
        rootfs: RootfsSpec::Image("alpine:latest".into()),
        auto_delete: Some(0),
        ..Default::default()
    }
}

/// Alpine box with `auto_delete>0` (cleaned up on stop).
pub fn alpine_opts_auto() -> BoxOptions {
    BoxOptions {
        rootfs: RootfsSpec::Image("alpine:latest".into()),
        auto_delete: Some(1),
        ..Default::default()
    }
}

mod engine;
pub use engine::TestRuntime;
