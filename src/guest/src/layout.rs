// Modified by ACRYL, 2026
//! Guest filesystem layout.
//!
//! Defines the directory structure under /run/acrylmicrovm/ where the guest
//! sets up mounts, containers, and runtime state.

use acrylmicrovm_shared::layout::{dirs, SharedGuestLayout, GUEST_BASE};
use std::path::{Path, PathBuf};

// ============================================================================
// CONTAINER LAYOUT (per-container runtime directory)
// ============================================================================

/// Per-container runtime directory layout.
///
/// Represents the OCI bundle directory for a single container:
/// ```text
/// /run/acrylmicrovm/containers/{cid}/
/// ├── rootfs/            # Container rootfs (bind → shared/containers/{cid}/rootfs)
/// └── state/             # libcontainer state directory
/// ```
#[derive(Clone, Debug)]
pub struct ContainerLayout {
    root: PathBuf,
}

#[allow(dead_code)]
impl ContainerLayout {
    /// Create a container layout with the given root path.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Root directory: /run/acrylmicrovm/containers/{cid}
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Rootfs directory: /run/acrylmicrovm/containers/{cid}/rootfs
    ///
    /// Bind mount target for shared/containers/{cid}/rootfs/.
    pub fn rootfs_dir(&self) -> PathBuf {
        self.root.join(dirs::ROOTFS)
    }

    /// Prepare container directory.
    pub fn prepare(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.rootfs_dir())
    }
}

// ============================================================================
// GUEST LAYOUT (guest runtime root)
// ============================================================================

/// Guest filesystem layout.
///
/// Provides paths for the guest's runtime directories:
/// - shared/ (virtiofs mount from host)
/// - containers/ (OCI bundles + libcontainer state)
///
/// # Structure
///
/// ```text
/// /run/acrylmicrovm/                       # base
/// ├── shared/                         # SharedGuestLayout (virtio-fs mount)
/// │   └── containers/{cid}/
/// │       ├── overlayfs/{upper,work}  # overlayfs writable layer
/// │       └── rootfs/                 # merged rootfs mount point
/// └── containers/                     # OCI containers
///     └── {cid}/
///         ├── config.json             # OCI bundle config
///         ├── rootfs/                 # bind mount to shared/.../rootfs
///         └── state/                  # libcontainer state
/// ```
#[derive(Clone, Debug)]
pub struct GuestLayout {
    base: PathBuf,
    shared: SharedGuestLayout,
}

impl Default for GuestLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl GuestLayout {
    /// Create guest layout with default base path: /run/acrylmicrovm
    pub fn new() -> Self {
        Self::with_base(GUEST_BASE)
    }

    /// Create guest layout with custom base path (for testing).
    pub fn with_base(base: impl Into<PathBuf>) -> Self {
        let base = base.into();
        let shared = SharedGuestLayout::new(base.join(dirs::SHARED));
        Self { base, shared }
    }

    // ========================================================================
    // BASE DIRECTORIES
    // ========================================================================

    /// Base directory: /run/acrylmicrovm
    pub fn base(&self) -> &Path {
        &self.base
    }

    /// Shared layout: /run/acrylmicrovm/shared
    ///
    /// Returns the SharedGuestLayout for accessing shared container directories.
    pub fn shared(&self) -> &SharedGuestLayout {
        &self.shared
    }

    /// Shared directory path: /run/acrylmicrovm/shared
    pub fn shared_dir(&self) -> &Path {
        self.shared.base()
    }

    // ========================================================================
    // CONTAINER LAYOUT
    // ========================================================================

    /// Containers directory: /run/acrylmicrovm/containers
    ///
    /// Each container has its own directory at /run/acrylmicrovm/containers/{cid}/
    /// containing OCI bundle (config.json, rootfs/) and libcontainer state.
    pub fn containers_dir(&self) -> PathBuf {
        self.base.join(dirs::CONTAINERS)
    }

    /// Get the bundle directory for a specific container.
    ///
    /// Returns /run/acrylmicrovm/containers/{cid}/ which contains:
    /// - config.json (OCI bundle config)
    /// - rootfs/ (bind mount to shared/.../rootfs)
    /// - state/ (libcontainer state)
    pub fn container_bundle_dir(&self, container_id: &str) -> PathBuf {
        self.containers_dir().join(container_id)
    }

    /// Get the state directory for a specific container.
    ///
    /// Returns /run/acrylmicrovm/containers/{cid}/state/ for libcontainer.
    pub fn container_state_dir(&self, container_id: &str) -> PathBuf {
        self.container_bundle_dir(container_id).join("state")
    }

    /// Get layout for a specific container's runtime directory.
    ///
    /// Returns ContainerLayout for /run/acrylmicrovm/containers/{cid}/.
    #[allow(dead_code)]
    pub fn container(&self, container_id: &str) -> ContainerLayout {
        ContainerLayout::new(self.container_bundle_dir(container_id))
    }

    // ========================================================================
    // PREPARATION
    // ========================================================================

    /// Prepare base directories.
    ///
    /// Called early in guest init before mounting virtio-fs.
    pub fn prepare_base(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.shared_dir())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // ContainerLayout tests
    // ========================================================================

    #[test]
    fn test_container_layout_paths() {
        let container = ContainerLayout::new("/run/acrylmicrovm/containers/main");

        assert_eq!(
            container.root().to_str().unwrap(),
            "/run/acrylmicrovm/containers/main"
        );
        assert_eq!(
            container.rootfs_dir().to_str().unwrap(),
            "/run/acrylmicrovm/containers/main/rootfs"
        );
    }

    // ========================================================================
    // GuestLayout tests
    // ========================================================================

    #[test]
    fn test_guest_layout_default() {
        let layout = GuestLayout::new();
        assert_eq!(layout.base().to_str().unwrap(), "/run/acrylmicrovm");
        assert_eq!(
            layout.shared_dir().to_str().unwrap(),
            "/run/acrylmicrovm/shared"
        );
    }

    #[test]
    fn test_guest_layout_custom_base() {
        let layout = GuestLayout::with_base("/tmp/test");
        assert_eq!(layout.base().to_str().unwrap(), "/tmp/test");
        assert_eq!(layout.shared_dir().to_str().unwrap(), "/tmp/test/shared");
    }

    #[test]
    fn test_guest_layout_container() {
        let layout = GuestLayout::new();
        let container = layout.container("main");

        assert_eq!(
            container.root().to_str().unwrap(),
            "/run/acrylmicrovm/containers/main"
        );
        assert_eq!(
            container.rootfs_dir().to_str().unwrap(),
            "/run/acrylmicrovm/containers/main/rootfs"
        );
    }

    #[test]
    fn test_guest_layout_containers_dir() {
        let layout = GuestLayout::new();
        assert_eq!(
            layout.containers_dir().to_str().unwrap(),
            "/run/acrylmicrovm/containers"
        );
    }

    #[test]
    fn test_guest_layout_container_bundle_dir() {
        let layout = GuestLayout::new();
        assert_eq!(
            layout.container_bundle_dir("main").to_str().unwrap(),
            "/run/acrylmicrovm/containers/main"
        );
    }

    #[test]
    fn test_guest_layout_container_state_dir() {
        let layout = GuestLayout::new();
        assert_eq!(
            layout.container_state_dir("main").to_str().unwrap(),
            "/run/acrylmicrovm/containers/main/state"
        );
    }

    #[test]
    fn test_guest_layout_shared_access() {
        let layout = GuestLayout::new();

        // Access shared layout through guest layout
        let shared_container = layout.shared().container("main");
        assert_eq!(
            shared_container.overlayfs_dir().to_str().unwrap(),
            "/run/acrylmicrovm/shared/containers/main/overlayfs"
        );
        assert_eq!(
            shared_container.rootfs_dir().to_str().unwrap(),
            "/run/acrylmicrovm/shared/containers/main/rootfs"
        );
    }
}
