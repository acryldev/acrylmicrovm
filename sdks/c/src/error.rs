// Modified by ACRYL, 2026
//! Error handling utilities for ACRYL microVMs FFI
//!
//! Provides error codes and conversion functions for C API error handling.

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

use acrylmicrovm::AcrylMicrovmError;

// ============================================================================
// Error Code Enum - Maps to AcrylMicrovmError variants
// ============================================================================

/// Error codes returned by ACRYL microVMs C API functions.
///
/// These codes map directly to Rust's AcrylMicrovmError variants,
/// allowing programmatic error handling in C.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcrylMicrovmErrorCode {
    /// Operation succeeded
    Ok = 0,
    /// Internal error
    Internal = 1,
    /// Resource not found
    NotFound = 2,
    /// Resource already exists
    AlreadyExists = 3,
    /// Invalid state for operation
    InvalidState = 4,
    /// Invalid argument provided
    InvalidArgument = 5,
    /// Configuration error
    Config = 6,
    /// Storage error
    Storage = 7,
    /// Image error
    Image = 8,
    /// Network error
    Network = 9,
    /// Execution error
    Execution = 10,
    /// Resource stopped
    Stopped = 11,
    /// Engine error
    Engine = 12,
    /// Unsupported operation
    Unsupported = 13,
    /// Database error
    Database = 14,
    /// Portal/communication error
    Portal = 15,
    /// RPC error
    Rpc = 16,
    /// RPC transport error
    RpcTransport = 17,
    /// Metadata error
    Metadata = 18,
    /// Unsupported engine error
    UnsupportedEngine = 19,
    /// System resource limit reached
    ResourceExhausted = 20,
    /// Interactive execution session was reaped server-side after disconnect.
    /// Reattach is no longer possible — start a new exec.
    SessionReaped = 21,
}

/// Extended error information for C API.
///
/// Contains both an error code (for programmatic handling)
/// and an optional detailed message (for debugging).
#[repr(C)]
pub struct FFIError {
    /// Error code
    pub code: AcrylMicrovmErrorCode,
    /// Detailed error message (NULL if none, caller must free with acrylmicrovm_error_free)
    pub message: *mut c_char,
}

impl Default for FFIError {
    fn default() -> Self {
        FFIError {
            code: AcrylMicrovmErrorCode::Ok,
            message: ptr::null_mut(),
        }
    }
}

/// Map AcrylMicrovmError to AcrylMicrovmErrorCode
pub fn error_to_code(err: &AcrylMicrovmError) -> AcrylMicrovmErrorCode {
    match err {
        AcrylMicrovmError::Internal(_) => AcrylMicrovmErrorCode::Internal,
        AcrylMicrovmError::NotFound(_) => AcrylMicrovmErrorCode::NotFound,
        AcrylMicrovmError::AlreadyExists(_) => AcrylMicrovmErrorCode::AlreadyExists,
        AcrylMicrovmError::InvalidState(_) => AcrylMicrovmErrorCode::InvalidState,
        AcrylMicrovmError::InvalidArgument(_) => AcrylMicrovmErrorCode::InvalidArgument,
        AcrylMicrovmError::Config(_) => AcrylMicrovmErrorCode::Config,
        AcrylMicrovmError::Storage(_) => AcrylMicrovmErrorCode::Storage,
        AcrylMicrovmError::Image(_) => AcrylMicrovmErrorCode::Image,
        AcrylMicrovmError::Network(_) => AcrylMicrovmErrorCode::Network,
        AcrylMicrovmError::Execution(_) => AcrylMicrovmErrorCode::Execution,
        AcrylMicrovmError::Stopped(_) => AcrylMicrovmErrorCode::Stopped,
        AcrylMicrovmError::Engine(_) => AcrylMicrovmErrorCode::Engine,
        AcrylMicrovmError::Unsupported(_) => AcrylMicrovmErrorCode::Unsupported,
        AcrylMicrovmError::UnsupportedEngine => AcrylMicrovmErrorCode::UnsupportedEngine,
        AcrylMicrovmError::Database(_) => AcrylMicrovmErrorCode::Database,
        AcrylMicrovmError::Portal(_) => AcrylMicrovmErrorCode::Portal,
        AcrylMicrovmError::Rpc(_) => AcrylMicrovmErrorCode::Rpc,
        AcrylMicrovmError::RpcTransport(_) => AcrylMicrovmErrorCode::RpcTransport,
        AcrylMicrovmError::MetadataError(_) => AcrylMicrovmErrorCode::Metadata,
        AcrylMicrovmError::ResourceExhausted(_) => AcrylMicrovmErrorCode::ResourceExhausted,
        AcrylMicrovmError::SessionReaped(_) => AcrylMicrovmErrorCode::SessionReaped,
    }
}

/// Convert Rust error to C string (caller must free)
pub fn error_to_c_string(err: &AcrylMicrovmError) -> *mut c_char {
    let msg = format!("{}", err);
    match CString::new(msg) {
        Ok(s) => s.into_raw(),
        Err(_) => {
            let fallback = CString::new("Failed to format error message").unwrap();
            fallback.into_raw()
        }
    }
}

/// Convert Rust error to C error struct
pub fn error_to_c_error(err: AcrylMicrovmError) -> FFIError {
    let code = error_to_code(&err);
    let message = error_to_c_string(&err);
    FFIError { code, message }
}

/// Write error to output parameter (if not NULL)
///
/// # Safety
/// out_error must be null or a valid pointer to FFIError
pub unsafe fn write_error(out_error: *mut FFIError, err: AcrylMicrovmError) {
    unsafe {
        if !out_error.is_null() {
            *out_error = error_to_c_error(err);
        }
    }
}

/// Helper to create InvalidArgument error for NULL pointers
pub fn null_pointer_error(param_name: &str) -> AcrylMicrovmError {
    AcrylMicrovmError::InvalidArgument(format!("{} is null", param_name))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn acrylmicrovm_error_free(error: *mut crate::CAcrylMicrovmError) {
    if !error.is_null() {
        let err = &mut *error;
        if !err.message.is_null() {
            drop(CString::from_raw(err.message));
            err.message = ptr::null_mut();
        }
        err.code = AcrylMicrovmErrorCode::Ok;
    }
}
