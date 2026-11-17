// Modified by ACRYL, 2026
//! Error types used across the ACRYL microVMs runtime.

use thiserror::Error;

/// Result type for ACRYL microVMs operations.
pub type AcrylMicrovmResult<T> = Result<T, AcrylMicrovmError>;

#[derive(Debug, Error)]
pub enum AcrylMicrovmError {
    #[error("unsupported engine kind")]
    UnsupportedEngine,

    #[error("engine reported an error: {0}")]
    Engine(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("images error: {0}")]
    Image(String),

    #[error("portal error: {0}")]
    Portal(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("gRPC/tonic error: {0}")]
    Rpc(String),

    #[error("gRPC transport error: {0}")]
    RpcTransport(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("unsupported: {0}")]
    Unsupported(String),

    /// The named resource — a box, image, volume or snapshot — does not exist.
    #[error("not found: {0}")]
    NotFound(String),

    /// Box or resource already exists.
    #[error("already exists: {0}")]
    AlreadyExists(String),

    /// Box is in wrong state for the requested operation.
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// Database operation failed.
    #[error("database error: {0}")]
    Database(String),

    /// Metadata corruption or parsing error.
    #[error("metadata error: {0}")]
    MetadataError(String),

    /// Invalid argument provided.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// Resource (box or runtime) has been stopped/shutdown.
    #[error("stopped: {0}")]
    Stopped(String),

    /// System resource limit reached (e.g., VM address spaces exhausted).
    #[error("resource exhausted: {0}")]
    ResourceExhausted(String),

    /// An interactive execution session was reaped server-side after the
    /// client disconnected and did not reconnect within the grace window
    /// (default 5 min idle → SIGHUP, escalating to SIGKILL; 24h hard cap).
    /// Reattach via the same `execution_id` is no longer possible — start
    /// a new exec instead.
    #[error("session reaped: {0}")]
    SessionReaped(String),
}

// Implement From for common error types to enable `?` operator
impl From<std::io::Error> for AcrylMicrovmError {
    fn from(err: std::io::Error) -> Self {
        AcrylMicrovmError::Internal(format!("I/O error: {}", err))
    }
}

impl From<serde_json::Error> for AcrylMicrovmError {
    fn from(err: serde_json::Error) -> Self {
        AcrylMicrovmError::Internal(format!("JSON error: {}", err))
    }
}

impl From<String> for AcrylMicrovmError {
    fn from(err: String) -> Self {
        AcrylMicrovmError::Internal(err)
    }
}

impl From<&str> for AcrylMicrovmError {
    fn from(err: &str) -> Self {
        AcrylMicrovmError::Internal(err.to_string())
    }
}

impl From<tonic::Status> for AcrylMicrovmError {
    fn from(err: tonic::Status) -> Self {
        AcrylMicrovmError::Rpc(err.to_string())
    }
}

impl From<tonic::transport::Error> for AcrylMicrovmError {
    fn from(err: tonic::transport::Error) -> Self {
        AcrylMicrovmError::RpcTransport(err.to_string())
    }
}

/// Canonical mapping from a `AcrylMicrovmError` variant to its HTTP response
/// shape: `(status, error_type, code)` where:
///
/// - `status` is the HTTP status code as a `u16`. Callers convert to
///   their concrete `StatusCode` type (axum, reqwest, http) at the
///   boundary.
/// - `error_type` is the stable PascalCase identifier matching the
///   `error.type` field on the wire (K8s `Status.reason` style).
/// - `code` is the stable snake_case machine identifier matching the
///   `error.code` field on the wire (Stripe `code` style).
///
/// Both `error_type` and `code` are part of the public API contract;
/// changing them is a breaking change to SDK clients that pattern-match
/// on them. The single source of truth lives here so adding a
/// `AcrylMicrovmError` variant becomes a compile error elsewhere.
///
/// Mapping rationale per row:
/// - Status semantics follow RFC 9110 (§15.5 — 422 specifically) and
///   Google's `google.rpc.Code` ↔ HTTP translation, except
///   `FAILED_PRECONDITION` (Google → 400, acrylmicrovm → 409 per
///   Docker / K8s / Stripe public-API consensus).
/// - `Image` and `Execution` are 422 because the *shape* of the
///   request was valid; the semantic content (`alpine:lastest`,
///   `/nonexistent/binary`) was wrong — RFC 9110 §15.5.21.
/// - `Network`, `Portal`, `Rpc`, `RpcTransport`, `Engine` are 503
///   because they signal an internal dep is unavailable, not that the
///   server itself failed (gRPC `UNAVAILABLE`).
/// - `Storage`, `Database`, `MetadataError`, `Config`, `Internal` are
///   500 because they indicate a server-side bug or data-plane
///   corruption — not a recoverable condition.
impl AcrylMicrovmError {
    /// Convert an error returned by request validation into a gRPC status.
    ///
    /// Validators use [`AcrylMicrovmError::InvalidArgument`] for caller mistakes;
    /// other variants indicate that validation itself could not be completed.
    pub fn into_validation_status(self) -> tonic::Status {
        match self {
            AcrylMicrovmError::InvalidArgument(message) => tonic::Status::invalid_argument(message),
            error => tonic::Status::internal(error.to_string()),
        }
    }

    pub fn http(&self) -> (u16, &'static str, &'static str) {
        match self {
            AcrylMicrovmError::InvalidArgument(_) => {
                (400, "InvalidArgumentError", "invalid_argument")
            }
            AcrylMicrovmError::Unsupported(_) | AcrylMicrovmError::UnsupportedEngine => {
                (400, "UnsupportedError", "unsupported")
            }
            AcrylMicrovmError::NotFound(_) => (404, "NotFoundError", "not_found"),
            AcrylMicrovmError::SessionReaped(_) => (410, "SessionReapedError", "session_reaped"),
            AcrylMicrovmError::AlreadyExists(_) => (409, "AlreadyExistsError", "already_exists"),
            AcrylMicrovmError::InvalidState(_) => (409, "InvalidStateError", "invalid_state"),
            AcrylMicrovmError::Stopped(_) => (409, "StoppedError", "stopped"),
            AcrylMicrovmError::Image(_) => (422, "ImageError", "image_pull_failed"),
            AcrylMicrovmError::Execution(_) => (422, "ExecutionError", "execution_failed"),
            AcrylMicrovmError::ResourceExhausted(_) => {
                (429, "ResourceExhaustedError", "resource_exhausted")
            }
            AcrylMicrovmError::Network(_) => (503, "NetworkError", "network_unavailable"),
            AcrylMicrovmError::Portal(_)
            | AcrylMicrovmError::Rpc(_)
            | AcrylMicrovmError::RpcTransport(_) => {
                (503, "UpstreamUnavailableError", "upstream_unavailable")
            }
            AcrylMicrovmError::Engine(_) => (503, "EngineError", "engine_unavailable"),
            AcrylMicrovmError::Storage(_) => (500, "StorageError", "storage_error"),
            AcrylMicrovmError::Database(_) => (500, "DatabaseError", "database_error"),
            AcrylMicrovmError::MetadataError(_) => (500, "MetadataError", "metadata_error"),
            AcrylMicrovmError::Config(_) => (500, "ConfigError", "config_error"),
            AcrylMicrovmError::Internal(_) => (500, "InternalError", "internal"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_status_distinguishes_caller_input_from_internal_failure() {
        let invalid = AcrylMicrovmError::InvalidArgument("unknown capability".into())
            .into_validation_status();
        assert_eq!(invalid.code(), tonic::Code::InvalidArgument);
        assert_eq!(invalid.message(), "unknown capability");

        let internal = AcrylMicrovmError::Internal("cannot read capability ceiling".into())
            .into_validation_status();
        assert_eq!(internal.code(), tonic::Code::Internal);
        assert_eq!(
            internal.message(),
            "internal error: cannot read capability ceiling"
        );
    }

    /// Canonical `AcrylMicrovmError → (status, error_type, code)` table.
    ///
    /// Each row is asserted via [`AcrylMicrovmError::http`]. Adding a
    /// `AcrylMicrovmError` variant without extending this table OR the
    /// `http()` match will produce a compile error in the impl
    /// (exhaustive match) — the assertion here is the second wall:
    /// it pins the *exact* status / type / code so a silent re-map
    /// later (e.g. changing 422 → 400) trips the test.
    #[test]
    fn http_mapping_matches_canonical_table() {
        let cases: &[(AcrylMicrovmError, u16, &str, &str)] = &[
            (
                AcrylMicrovmError::InvalidArgument("cpus must be 1..32".into()),
                400,
                "InvalidArgumentError",
                "invalid_argument",
            ),
            (
                AcrylMicrovmError::Unsupported("feature x not built".into()),
                400,
                "UnsupportedError",
                "unsupported",
            ),
            (
                AcrylMicrovmError::UnsupportedEngine,
                400,
                "UnsupportedError",
                "unsupported",
            ),
            (
                AcrylMicrovmError::NotFound("box abc".into()),
                404,
                "NotFoundError",
                "not_found",
            ),
            (
                AcrylMicrovmError::SessionReaped("exec 123".into()),
                410,
                "SessionReapedError",
                "session_reaped",
            ),
            (
                AcrylMicrovmError::AlreadyExists("box abc".into()),
                409,
                "AlreadyExistsError",
                "already_exists",
            ),
            (
                AcrylMicrovmError::InvalidState("box not running".into()),
                409,
                "InvalidStateError",
                "invalid_state",
            ),
            (
                AcrylMicrovmError::Stopped("runtime shut down".into()),
                409,
                "StoppedError",
                "stopped",
            ),
            (
                AcrylMicrovmError::Image("manifest unknown for alpine:lastest".into()),
                422,
                "ImageError",
                "image_pull_failed",
            ),
            (
                AcrylMicrovmError::Execution("/nonexistent: no such file".into()),
                422,
                "ExecutionError",
                "execution_failed",
            ),
            (
                AcrylMicrovmError::ResourceExhausted("disk full".into()),
                429,
                "ResourceExhaustedError",
                "resource_exhausted",
            ),
            (
                AcrylMicrovmError::Network("tap setup failed".into()),
                503,
                "NetworkError",
                "network_unavailable",
            ),
            (
                AcrylMicrovmError::Portal("portal not ready".into()),
                503,
                "UpstreamUnavailableError",
                "upstream_unavailable",
            ),
            (
                AcrylMicrovmError::Rpc("tonic status: …".into()),
                503,
                "UpstreamUnavailableError",
                "upstream_unavailable",
            ),
            (
                AcrylMicrovmError::RpcTransport("connection refused".into()),
                503,
                "UpstreamUnavailableError",
                "upstream_unavailable",
            ),
            (
                AcrylMicrovmError::Engine("krun init failed".into()),
                503,
                "EngineError",
                "engine_unavailable",
            ),
            (
                AcrylMicrovmError::Storage("qcow2 corrupt".into()),
                500,
                "StorageError",
                "storage_error",
            ),
            (
                AcrylMicrovmError::Database("sqlite locked".into()),
                500,
                "DatabaseError",
                "database_error",
            ),
            (
                AcrylMicrovmError::MetadataError("box.toml parse failed".into()),
                500,
                "MetadataError",
                "metadata_error",
            ),
            (
                AcrylMicrovmError::Config("startup misconfig".into()),
                500,
                "ConfigError",
                "config_error",
            ),
            (
                AcrylMicrovmError::Internal("unreachable".into()),
                500,
                "InternalError",
                "internal",
            ),
        ];

        for (err, want_status, want_type, want_code) in cases {
            let (status, etype, ecode) = err.http();
            assert_eq!(
                (status, etype, ecode),
                (*want_status, *want_type, *want_code),
                "variant {:?} mapped incorrectly",
                err
            );
        }
    }

    /// Spot-check: the snake_code identifier set is unique. Two
    /// variants sharing a code would break round-tripping on the
    /// client (which dispatches on `error.code`).
    #[test]
    fn http_code_strings_are_unique_per_logical_status() {
        // Variants that intentionally share a code (multi-variant
        // rows in the canonical table).
        let shared_ok: &[&str] = &["unsupported", "upstream_unavailable"];

        let all: Vec<&'static str> = [
            AcrylMicrovmError::InvalidArgument(String::new()),
            AcrylMicrovmError::Unsupported(String::new()),
            AcrylMicrovmError::UnsupportedEngine,
            AcrylMicrovmError::NotFound(String::new()),
            AcrylMicrovmError::SessionReaped(String::new()),
            AcrylMicrovmError::AlreadyExists(String::new()),
            AcrylMicrovmError::InvalidState(String::new()),
            AcrylMicrovmError::Stopped(String::new()),
            AcrylMicrovmError::Image(String::new()),
            AcrylMicrovmError::Execution(String::new()),
            AcrylMicrovmError::ResourceExhausted(String::new()),
            AcrylMicrovmError::Network(String::new()),
            AcrylMicrovmError::Portal(String::new()),
            AcrylMicrovmError::Rpc(String::new()),
            AcrylMicrovmError::RpcTransport(String::new()),
            AcrylMicrovmError::Engine(String::new()),
            AcrylMicrovmError::Storage(String::new()),
            AcrylMicrovmError::Database(String::new()),
            AcrylMicrovmError::MetadataError(String::new()),
            AcrylMicrovmError::Config(String::new()),
            AcrylMicrovmError::Internal(String::new()),
        ]
        .iter()
        .map(|e| e.http().2)
        .collect();

        // Build a histogram; codes seen >1 time must be in shared_ok.
        let mut counts: std::collections::HashMap<&'static str, usize> =
            std::collections::HashMap::new();
        for c in &all {
            *counts.entry(c).or_default() += 1;
        }
        for (code, n) in counts {
            if n > 1 {
                assert!(
                    shared_ok.contains(&code),
                    "code {:?} mapped by {} variants but not in shared_ok",
                    code,
                    n
                );
            }
        }
    }
}
