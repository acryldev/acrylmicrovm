# Modified by ACRYL, 2026
"""Synchronous access to independent managed checkpoints."""

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from ..acrylmicrovm import CheckpointHandle, CheckpointInfo
    from ._acrylmicrovm import SyncAcrylMicrovm


class SyncCheckpointHandle:
    def __init__(self, runtime: "SyncAcrylMicrovm", handle: "CheckpointHandle") -> None:
        from ._sync_base import SyncBase

        self._runtime = runtime
        self._handle = handle
        self._bridge = SyncBase(handle, runtime.loop, runtime.dispatcher_fiber)

    def create(self, source: str, name: str | None = None) -> "CheckpointInfo":
        return self._bridge._sync(self._handle.create(source, name))

    def list(self) -> list["CheckpointInfo"]:
        return self._bridge._sync(self._handle.list())

    def get(self, key: str) -> "CheckpointInfo":
        return self._bridge._sync(self._handle.get(key))

    def remove(self, key: str, force: bool = False) -> None:
        self._bridge._sync(self._handle.remove(key, force))
