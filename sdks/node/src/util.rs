// Modified by ACRYL, 2026
use acrylmicrovm_shared::errors::AcrylMicrovmError;
use napi::Error as NapiError;

/// Map AcrylMicrovmError to napi Error
/// This is the single source of truth for error conversion (DRY principle)
pub(crate) fn map_err(err: AcrylMicrovmError) -> NapiError {
    NapiError::from_reason(format!("{}", err))
}
