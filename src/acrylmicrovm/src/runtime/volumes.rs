// Modified by ACRYL, 2026
//! Named-volume operations handle.
//!
//! Provides [`VolumeHandle`] for managing named volumes (create, list, get,
//! remove). This mirrors [`ImageHandle`](crate::runtime::ImageHandle): volume
//! management is a distinct capability, surfaced via `AcrylMicrovmRuntime::volumes()`
//! and backed by either a local runtime or a REST runtime.
//!
//! The trait is `#[async_trait]` like the other capability backends
//! ([`ImageBackend`](crate::runtime::images::ImageBackend),
//! [`AuthBackend`](crate::runtime::auth::AuthBackend)) so REST backends can
//! perform network calls. The concrete backend is not implemented yet — every
//! operation currently returns `Unsupported`.

use std::sync::Arc;

use async_trait::async_trait;

use crate::AcrylMicrovmResult;
use crate::volumes::VolumeInfo;

/// Internal trait for named-volume management.
///
/// Implemented by both `LocalRuntime` and the REST runtime. Both return
/// `Unsupported` until a managed volume backend is wired up.
#[allow(
    clippy::double_must_use,
    reason = "async_trait generates a must-use boxed future"
)]
#[async_trait]
pub(crate) trait VolumeBackend: Send + Sync {
    /// Create a volume, returning its server-assigned metadata (including id).
    /// `name` is optional; the server names the volume after its id without one.
    async fn create_volume(&self, name: Option<&str>) -> AcrylMicrovmResult<VolumeInfo>;

    /// List all volumes.
    async fn list_volumes(&self) -> AcrylMicrovmResult<Vec<VolumeInfo>>;

    /// Get metadata for a single volume by id.
    async fn get_volume(&self, id: &str) -> AcrylMicrovmResult<VolumeInfo>;

    /// Remove a volume by id. `force` makes a missing volume a no-op.
    async fn remove_volume(&self, id: &str, force: bool) -> AcrylMicrovmResult<()>;
}

/// Handle for performing named-volume operations.
///
/// Obtained via [`AcrylMicrovmRuntime::volumes()`](crate::AcrylMicrovmRuntime::volumes).
#[derive(Clone)]
pub struct VolumeHandle {
    backend: Arc<dyn VolumeBackend>,
}

impl VolumeHandle {
    /// Internal constructor used by `AcrylMicrovmRuntime`.
    pub(crate) fn new(backend: Arc<dyn VolumeBackend>) -> Self {
        Self { backend }
    }

    /// Create a volume, returning its metadata (including the assigned id).
    ///
    /// A named volume can be mounted by that name instead of its id. Without
    /// a name the server uses the id, so the volume is still mountable.
    pub async fn create(&self, name: Option<&str>) -> AcrylMicrovmResult<VolumeInfo> {
        self.backend.create_volume(name).await
    }

    /// List all volumes.
    pub async fn list(&self) -> AcrylMicrovmResult<Vec<VolumeInfo>> {
        self.backend.list_volumes().await
    }

    /// Get metadata for a single volume by id.
    pub async fn get(&self, id: &str) -> AcrylMicrovmResult<VolumeInfo> {
        self.backend.get_volume(id).await
    }

    /// Remove a volume by id. With `force`, a missing volume is a no-op.
    pub async fn remove(&self, id: &str, force: bool) -> AcrylMicrovmResult<()> {
        self.backend.remove_volume(id, force).await
    }
}
