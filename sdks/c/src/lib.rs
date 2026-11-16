// Modified by ACRYL, 2026
//! C SDK for ACRYL microVMs
//!
//! This crate provides C FFI bindings for the ACRYL microVMs runtime,
//! building the C shared library and static library artifacts.

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::too_many_arguments)]

mod advanced_options;
mod agents;
mod archive;
mod box_handle;
mod checkpoints;
mod copy;
mod domains;
mod error;
mod event_queue;
mod exec;
mod images;
mod info;
mod metrics;
mod network;
mod options;
mod registries;
mod rest;
mod runtime;
mod ssh;
#[cfg(test)]
mod tests;
mod util;
mod volumes;

/// Test-only counter incremented every time `free_str` reclaims a
/// `CString::from_raw`'d inner pointer. Lets nested-leak reproducer
/// tests verify that `OwnedFfiPtr<T>::drop` for FFI payload types like
/// `CImagePullResult` actually traverses the struct's nested allocations
/// rather than only freeing the outer `Box`.
#[cfg(test)]
pub(crate) static FREE_STR_CALLS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// Serializes tests that observe `FREE_STR_CALLS` deltas. The counter is
/// process-global, so without this lock parallel cargo tests interleave
/// and produce false-positive failures (test A's `before` snapshot
/// includes test B's increments). Each leak-reproducer test must acquire
/// `FREE_STR_LOCK` before reading `FREE_STR_CALLS`.
#[cfg(test)]
pub(crate) static FREE_STR_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub type CAcrylMicrovmRuntime = runtime::RuntimeHandle;
pub type CCheckpointInfo = checkpoints::CCheckpointInfo;
pub type CCheckpointList = checkpoints::CCheckpointList;
pub type CBoxHandle = box_handle::BoxHandle;
pub type CBoxNetworkHandle = network::BoxNetworkHandle;
pub type CBoxTunnelHandle = network::BoxTunnelHandle;
pub type CTunnelForwarderHandle = network::TunnelForwarderHandle;
pub type CAcrylMicrovmImageHandle = images::ImageHandle;
pub type CAcrylMicrovmVolumeHandle = volumes::VolumeHandle;
pub type CAcrylMicrovmDomainHandle = domains::DomainHandle;
pub type CAcrylMicrovmRegistryHandle = registries::RegistryHandle;
pub type CAcrylMicrovmOptions = options::OptionsHandle;
pub type CAcrylMicrovmCredential = rest::CredentialHandle;
pub type CAcrylMicrovmRestOptions = rest::RestOptionsHandle;
pub type CAcrylMicrovmSimple = exec::BoxRunner;
pub type CAcrylMicrovmError = error::FFIError;
pub type CAcrylMicrovmExecResult = exec::ExecResult;
pub type CBoxInfo = info::CBoxInfo;
pub type CBoxInfoList = info::CBoxInfoList;
pub type CBoxMetrics = metrics::CBoxMetrics;
pub type CExecutionHandle = exec::ExecutionHandle;
pub type CExecutionInfo = exec::CExecutionInfo;
pub type CExecutionInfoList = exec::CExecutionInfoList;
pub type CImageInfoList = images::CImageInfoList;
pub type CImagePullResult = images::CImagePullResult;
pub type CImageDetail = images::CImageDetail;
pub type CImageUsage = images::CImageUsage;
pub type CVolumeInfo = volumes::CVolumeInfo;
pub type CVolumeInfoList = volumes::CVolumeInfoList;
pub type CDomain = domains::CDomain;
pub type CDomainList = domains::CDomainList;
pub type CDomainTarget = domains::CDomainTarget;
pub type CRegistryCredential = registries::CRegistryCredential;
pub type CRegistryCredentialList = registries::CRegistryCredentialList;
pub type CRuntimeMetrics = metrics::CRuntimeMetrics;
pub type AcrylMicrovmCommand = exec::AcrylMicrovmCommand;
pub type CAdvancedBoxOptions = advanced_options::AdvancedBoxOptionsHandle;

pub use advanced_options::*;
pub use agents::*;
pub use box_handle::*;
pub use checkpoints::*;
pub use copy::*;
pub use domains::*;
pub use error::*;
pub use event_queue::*;
pub use exec::*;
pub use images::*;
pub use info::*;
pub use metrics::*;
pub use network::*;
pub use options::*;
pub use registries::*;
pub use rest::*;
pub use runtime::*;
pub use ssh::*;
pub use util::*;
pub use volumes::*;
