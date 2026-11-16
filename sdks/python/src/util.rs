// Modified by ACRYL, 2026
use acrylmicrovm::AcrylMicrovmError;
use pyo3::{exceptions::PyRuntimeError, prelude::*};

type Raise = fn(String) -> PyErr;

/// Imports each `acrylmicrovm.errors` class (lazily, on the first failure raised as
/// it) and keys it by its name, which is the name `AcrylMicrovmError::http()` gives
/// the class — so the grouping of variants into classes is decided there.
macro_rules! python_classes {
    ($($class:ident),+ $(,)?) => {
        mod classes {
            $(pyo3::import_exception!(acrylmicrovm.errors, $class);)+
        }

        const CLASSES: &[(&str, Raise)] =
            &[$((stringify!($class), classes::$class::new_err::<String>)),+];
    };
}

python_classes!(
    InvalidArgumentError,
    UnsupportedError,
    NotFoundError,
    SessionReapedError,
    AlreadyExistsError,
    InvalidStateError,
    StoppedError,
    ImageError,
    ExecutionError,
    ResourceExhaustedError,
    NetworkError,
    UpstreamUnavailableError,
    EngineError,
    StorageError,
    DatabaseError,
    MetadataError,
    ConfigError,
    InternalError,
);

/// Raise a runtime failure as its `acrylmicrovm.errors` class, message unchanged.
pub(crate) fn map_err(err: AcrylMicrovmError) -> PyErr {
    let (_, class, _) = err.http();
    let message = err.to_string();
    match CLASSES.iter().find(|(name, _)| *name == class) {
        Some((_, raise)) => raise(message),
        None => PyRuntimeError::new_err(message),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One of every variant. The match has no wildcard, so a new variant stops
    /// this compiling until it is listed here too.
    fn every_variant() -> Vec<AcrylMicrovmError> {
        let all = vec![
            AcrylMicrovmError::UnsupportedEngine,
            AcrylMicrovmError::Engine(String::new()),
            AcrylMicrovmError::Config(String::new()),
            AcrylMicrovmError::Storage(String::new()),
            AcrylMicrovmError::Image(String::new()),
            AcrylMicrovmError::Portal(String::new()),
            AcrylMicrovmError::Network(String::new()),
            AcrylMicrovmError::Rpc(String::new()),
            AcrylMicrovmError::RpcTransport(String::new()),
            AcrylMicrovmError::Internal(String::new()),
            AcrylMicrovmError::Execution(String::new()),
            AcrylMicrovmError::Unsupported(String::new()),
            AcrylMicrovmError::NotFound(String::new()),
            AcrylMicrovmError::AlreadyExists(String::new()),
            AcrylMicrovmError::InvalidState(String::new()),
            AcrylMicrovmError::Database(String::new()),
            AcrylMicrovmError::MetadataError(String::new()),
            AcrylMicrovmError::InvalidArgument(String::new()),
            AcrylMicrovmError::Stopped(String::new()),
            AcrylMicrovmError::ResourceExhausted(String::new()),
            AcrylMicrovmError::SessionReaped(String::new()),
        ];
        for err in &all {
            match err {
                AcrylMicrovmError::UnsupportedEngine
                | AcrylMicrovmError::Engine(_)
                | AcrylMicrovmError::Config(_)
                | AcrylMicrovmError::Storage(_)
                | AcrylMicrovmError::Image(_)
                | AcrylMicrovmError::Portal(_)
                | AcrylMicrovmError::Network(_)
                | AcrylMicrovmError::Rpc(_)
                | AcrylMicrovmError::RpcTransport(_)
                | AcrylMicrovmError::Internal(_)
                | AcrylMicrovmError::Execution(_)
                | AcrylMicrovmError::Unsupported(_)
                | AcrylMicrovmError::NotFound(_)
                | AcrylMicrovmError::AlreadyExists(_)
                | AcrylMicrovmError::InvalidState(_)
                | AcrylMicrovmError::Database(_)
                | AcrylMicrovmError::MetadataError(_)
                | AcrylMicrovmError::InvalidArgument(_)
                | AcrylMicrovmError::Stopped(_)
                | AcrylMicrovmError::ResourceExhausted(_)
                | AcrylMicrovmError::SessionReaped(_) => {}
            }
        }
        all
    }

    #[test]
    fn every_variant_is_raised_as_a_class_of_its_own() {
        for err in every_variant() {
            let (_, class, _) = err.http();
            assert!(
                CLASSES.iter().any(|(name, _)| *name == class),
                "{err:?} would be raised as a bare RuntimeError: no class for {class}"
            );
        }
    }

    /// `map_err` imports its classes from the `acrylmicrovm` package beside this
    /// crate, as the extension does from its installed package.
    fn put_the_package_on_the_import_path(py: Python<'_>) {
        let path = py.import("sys").unwrap().getattr("path").unwrap();
        let package_root = env!("CARGO_MANIFEST_DIR");
        if !path.contains(package_root).unwrap() {
            path.call_method1("insert", (0, package_root)).unwrap();
        }
    }

    /// Existing `except RuntimeError` handlers must keep catching every
    /// failure, and the text a caller logs must not change.
    #[test]
    fn every_variant_is_raised_as_its_class_with_the_message_unchanged() {
        Python::attach(|py| {
            put_the_package_on_the_import_path(py);
            for err in every_variant() {
                let (_, class, _) = err.http();
                let message = err.to_string();

                let raised = map_err(err);

                let raised_class = raised.get_type(py);
                assert_eq!(raised_class.module().unwrap(), "acrylmicrovm.errors");
                assert_eq!(raised_class.name().unwrap(), class);
                assert!(
                    raised.is_instance_of::<PyRuntimeError>(py),
                    "{class} is not a RuntimeError"
                );
                assert_eq!(raised.value(py).to_string(), message);
            }
        });
    }

    /// The Python classes are written out by hand in `errors.py`; this reads
    /// that file so a class missing there, or carrying another code, fails
    /// here rather than when a user first meets that failure.
    #[test]
    fn every_class_is_declared_in_python_with_its_code() {
        let source = include_str!("../acrylmicrovm/errors.py");
        for err in every_variant() {
            let (_, class, code) = err.http();
            let start = source
                .find(&format!("\nclass {class}(_NativeError):"))
                .unwrap_or_else(|| panic!("errors.py declares no {class}"));
            let body = &source[start + 1..];
            let body = &body[..body.find("\nclass ").unwrap_or(body.len())];
            assert!(
                body.contains(&format!("code = \"{code}\"")),
                "errors.py gives {class} another code than {code}"
            );
        }
    }
}
