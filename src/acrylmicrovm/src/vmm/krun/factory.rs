// Modified by ACRYL, 2026
//! Krun engine factory implementation.

use crate::vmm::{
    VmmConfig, VmmKind, factory::VmmFactory, krun::Krun, registry::EngineFactoryRegistration,
};
use acrylmicrovm_shared::errors::AcrylMicrovmResult;

pub struct KrunFactory;

impl VmmFactory for KrunFactory {
    type Engine = Krun;

    fn create(options: VmmConfig) -> AcrylMicrovmResult<Self::Engine> {
        Krun::new(options)
    }
}

// Auto-register this factory with the global registry at compile time
inventory::submit! {
    EngineFactoryRegistration {
        kind: VmmKind::Libkrun,
        factory: |options| {
            Ok(Box::new(KrunFactory::create(options)?))
        }
    }
}
