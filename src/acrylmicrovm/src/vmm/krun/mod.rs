// Modified by ACRYL, 2026
//! Libkrun-based engine implementation.

mod constants;
pub mod context;
pub mod engine;
pub mod factory;

use acrylmicrovm_shared::{AcrylMicrovmError, AcrylMicrovmResult};
pub use engine::Krun;
pub use factory::KrunFactory;

pub(crate) fn check_status(label: &str, status: i32) -> AcrylMicrovmResult<()> {
    if status < 0 {
        tracing::error!(function = label, status, "libkrun FFI call failed");
        if status == -22 {
            return Err(AcrylMicrovmError::Engine(format!(
                "libkrun function '{}' returned EINVAL (-22). Possible causes:\n\
                 - macOS: VM address space limit reached (kern.hv.max_address_spaces)\n\
                 - Invalid rootfs structure (missing kernel or initrd)\n\
                 Run `acrylmicrovm list` to check active boxes.",
                label
            )));
        }
        Err(AcrylMicrovmError::Engine(format!(
            "libkrun function '{}' failed with status {}",
            label, status
        )))
    } else {
        Ok(())
    }
}
