// Modified by ACRYL, 2026
//! Init duties normally supplied by libkrun. Run before hardening and the zygote.

use acrylmicrovm_shared::{AcrylMicrovmError, AcrylMicrovmResult};
use nix::sys::resource::{setrlimit, Resource};
use nix::unistd::{getpid, sethostname};

pub(crate) fn prepare() -> AcrylMicrovmResult<()> {
    if getpid().as_raw() != 1 {
        return Ok(());
    }

    crate::mounts::mount_init_filesystems()?;
    let hostname = std::env::var_os("HOSTNAME").unwrap_or_else(|| "localhost".into());
    sethostname(&hostname)
        .map_err(|e| AcrylMicrovmError::Internal(format!("set guest hostname: {e}")))?;

    // Match the limits passed to init.krun by the libkrun engine. Workload
    // limits remain owned by the OCI spec, independently of these agent limits.
    for (resource, soft, hard) in [
        (Resource::RLIMIT_NPROC, 4096, 8192),
        (Resource::RLIMIT_NOFILE, 1_048_576, 1_048_576),
    ] {
        setrlimit(resource, soft, hard)
            .map_err(|e| AcrylMicrovmError::Internal(format!("set guest {resource:?}: {e}")))?;
    }
    eprintln!(
        "[guest] T+{}ms: init bootstrap completed",
        crate::boot_elapsed_ms()
    );
    Ok(())
}
