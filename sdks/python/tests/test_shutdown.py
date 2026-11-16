# Modified by ACRYL, 2026
"""
Integration tests for runtime shutdown functionality.

These tests verify the shutdown() method on the AcrylMicrovm runtime.
Each test creates its own isolated runtime with a temporary ACRYLMICROVM_HOME
to avoid lock contention with the shared session-scoped runtime.

Uses a SyncAcrylMicrovm wrapper that borrows the shared runtime's event loop
and greenlet dispatcher, wrapping a fresh AcrylMicrovm instance for isolation.
"""

from __future__ import annotations

import tempfile

import pytest

import acrylmicrovm

pytestmark = pytest.mark.integration

# Try to import sync API - skip if greenlet not installed
try:
    from acrylmicrovm import SyncAcrylMicrovm

    SYNC_AVAILABLE = True
except ImportError:
    SYNC_AVAILABLE = False


@pytest.fixture
def isolated_sync_runtime(shared_sync_runtime):
    """Create an isolated sync runtime with its own ACRYLMICROVM_HOME.

    Borrows the greenlet dispatcher and event loop from shared_sync_runtime
    but wraps a fresh AcrylMicrovm instance with a separate home directory.
    This avoids both lock contention and event loop conflicts.
    """
    tmpdir = tempfile.mkdtemp(prefix="acrylmicrovm-test-shutdown-")
    inner = acrylmicrovm.AcrylMicrovm(acrylmicrovm.Options(home_dir=tmpdir))

    # Reuse the shared runtime's event loop and dispatcher
    rt = object.__new__(SyncAcrylMicrovm)
    rt._acrylmicrovm = inner
    rt._loop = shared_sync_runtime._loop
    rt._dispatcher_fiber = shared_sync_runtime._dispatcher_fiber
    rt._own_loop = False  # Don't close the shared loop
    rt._sync_helper = shared_sync_runtime._sync_helper.__class__(
        inner,
        shared_sync_runtime._loop,
        shared_sync_runtime._dispatcher_fiber,
    )
    rt._started = True

    yield rt
    # No cleanup needed - the shared runtime owns the loop


class TestShutdownSync:
    """Test runtime shutdown with sync API."""

    def test_shutdown_sync_default(self, isolated_sync_runtime):
        """Shutdown via sync API with default timeout."""
        runtime = isolated_sync_runtime
        runtime.create(acrylmicrovm.BoxOptions(image="alpine:latest"))
        runtime.shutdown()

    def test_shutdown_sync_custom_timeout(self, isolated_sync_runtime):
        """Shutdown via sync API with custom timeout."""
        runtime = isolated_sync_runtime
        runtime.create(acrylmicrovm.BoxOptions(image="alpine:latest"))
        runtime.shutdown(timeout=5)

    def test_shutdown_sync_idempotent(self, isolated_sync_runtime):
        """Sync shutdown can be called multiple times safely."""
        runtime = isolated_sync_runtime
        runtime.shutdown()
        runtime.shutdown()  # Should not fail

    def test_shutdown_stops_all_boxes(self, isolated_sync_runtime):
        """Shutdown gracefully stops multiple boxes."""
        runtime = isolated_sync_runtime

        # Create multiple boxes
        for _ in range(3):
            runtime.create(acrylmicrovm.BoxOptions(image="alpine:latest"))

        # Shutdown all
        runtime.shutdown(timeout=10)

    def test_operations_fail_after_shutdown(self, isolated_sync_runtime):
        """Operations fail after runtime shutdown."""
        runtime = isolated_sync_runtime
        runtime.shutdown()

        with pytest.raises(RuntimeError):
            runtime.create(acrylmicrovm.BoxOptions(image="alpine:latest"))


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
