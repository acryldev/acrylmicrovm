// Modified by ACRYL, 2026
//! VMM Factory pattern for dependency management and engine creation.

use crate::vmm::{Vmm, VmmConfig};
use acrylmicrovm_shared::errors::AcrylMicrovmResult;

/// Factory trait for creating VMM engines.
pub trait VmmFactory {
    type Engine: Vmm;

    /// Create engine instance with the provided options
    fn create(options: VmmConfig) -> AcrylMicrovmResult<Self::Engine>;
}
