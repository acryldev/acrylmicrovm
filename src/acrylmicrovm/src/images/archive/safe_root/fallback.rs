// Modified by ACRYL, 2026
//! Non-Linux: delegates to the shared walk algorithm.

use super::path;
use acrylmicrovm_shared::errors::AcrylMicrovmResult;
use std::path::{Path, PathBuf};

pub(super) struct Backend {
    root: PathBuf,
}

impl Backend {
    pub(super) fn open(root: &Path) -> AcrylMicrovmResult<Self> {
        Ok(Self {
            root: root.to_path_buf(),
        })
    }

    pub(super) fn resolve(&self, rel: &Path) -> AcrylMicrovmResult<PathBuf> {
        path::resolve_walk(&self.root, rel)
    }
}
