# Modified by ACRYL, 2026
"""
SyncAcrylMicrovm - Synchronous wrapper for AcrylMicrovm runtime.

Provides sync API using greenlet fiber switching. This is the main entry point
for the synchronous AcrylMicrovm API.
"""

import asyncio
import logging
from typing import TYPE_CHECKING, Optional

from greenlet import greenlet

if TYPE_CHECKING:
    from ..acrylmicrovm import (
        AcrylMicrovm,
        AcrylMicrovmRestOptions,
        BoxOptions,
        Options,
        RuntimeMetrics,
    )
    from ._box import SyncBox
    from ._checkpoints import SyncCheckpointHandle
    from ._domains import SyncDomainHandle
    from ._images import SyncImageHandle

logger = logging.getLogger("acrylmicrovm.sync_acrylmicrovm")

__all__ = ["SyncAcrylMicrovm"]


class SyncAcrylMicrovm:
    """
    Synchronous wrapper for AcrylMicrovm runtime.

    This class handles both the dispatcher fiber lifecycle AND provides the
    sync-capable runtime operations. Metadata methods remain async-only and are
    not exposed by this wrapper.

    Usage (default runtime - preferred):
        with SyncAcrylMicrovm.default() as runtime:
            box = runtime.create(BoxOptions(image="alpine:latest"))
            execution = box.exec("echo", ["Hello"])
            for line in execution.stdout():
                print(line)
            box.stop()

    Usage (with custom options):
        with SyncAcrylMicrovm(Options(home_dir="/custom/path")) as runtime:
            box = runtime.create(BoxOptions(image="alpine:latest"))
            # Data stored in /custom/path instead of ~/.acryl/microvm
            box.stop()

    Usage (manual start/stop - for REPL, test fixtures):
        runtime = SyncAcrylMicrovm.default().start()
        box = runtime.create(BoxOptions(image="alpine:latest"))
        # ... use box ...
        runtime.stop()

    Architecture:
        - Creates a dispatcher greenlet fiber that runs the event loop
        - User code runs in the main fiber
        - When user calls a sync method, it switches to dispatcher
        - Dispatcher processes the async task
        - When task completes, callback switches back to user fiber
    """

    def __init__(self, options: "Options") -> None:
        """Create a new SyncAcrylMicrovm instance.

        Args:
            options: Runtime options (e.g., custom home_dir).
                     Use SyncAcrylMicrovm.default() for default runtime.
        """
        from ..acrylmicrovm import AcrylMicrovm

        self._adopt(AcrylMicrovm(options))

    def _adopt(self, acrylmicrovm: "AcrylMicrovm") -> None:
        """Wrap an async runtime; the dispatcher starts on __enter__/start()."""
        self._acrylmicrovm = acrylmicrovm
        self._loop: asyncio.AbstractEventLoop = None
        self._dispatcher_fiber: greenlet = None
        self._own_loop = False
        self._sync_helper = None
        self._started = False

    def __enter__(self) -> "SyncAcrylMicrovm":  # noqa: PYI034 - typing.Self needs 3.11+; project supports 3.10+
        """
        Start the sync runtime and enter context.

        Returns:
            Self, with dispatcher fiber running and runtime ready.

        Raises:
            RuntimeError: If called from within an async context.
        """
        # 1. Create event loop
        try:
            self._loop = asyncio.get_running_loop()
        except RuntimeError:
            self._loop = asyncio.new_event_loop()
            self._own_loop = True

        # 2. Check not in async context (loop shouldn't be running)
        if self._loop.is_running():
            raise RuntimeError(
                "Cannot use SyncAcrylMicrovm inside an asyncio loop. "
                "Use the async API (CodeBox, SimpleBox) instead."
            )

        # 3. Create dispatcher fiber
        def greenlet_main() -> None:
            """
            Dispatcher fiber entry point.

            Runs the event loop indefinitely until stop() is called.
            The event loop efficiently waits for I/O events (via OS-level
            epoll/kqueue) and processes tasks scheduled by _sync() calls.
            """
            self._loop.run_forever()

        self._dispatcher_fiber = greenlet(greenlet_main)

        from ._sync_base import SyncBase

        self._sync_helper = SyncBase(self._acrylmicrovm, self._loop, self._dispatcher_fiber)

        # 5. Start dispatcher fiber
        g_self = greenlet.getcurrent()

        def on_ready():
            """Callback to switch back to user fiber after dispatcher starts."""
            g_self.switch()

        self._loop.call_soon(on_ready)
        self._dispatcher_fiber.switch()
        # Control returns here after dispatcher calls on_ready()

        self._started = True
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        """
        Stop the sync runtime and clean up.

        This stops the dispatcher fiber and closes the event loop if we own it.
        """
        self._started = False

        # Signal the event loop to stop, then switch to let it finish cleanly
        self._loop.call_soon(self._loop.stop)
        self._dispatcher_fiber.switch()

        if self._own_loop:
            # Cancel pending tasks
            try:
                tasks = asyncio.all_tasks(self._loop)
                for t in [t for t in tasks if not (t.done() or t.cancelled())]:
                    t.cancel()
                self._loop.run_until_complete(self._loop.shutdown_asyncgens())
            except Exception as e:  # noqa: BLE001 - loop is shutting down regardless; must not raise
                logger.debug(f"Error during event loop cleanup: {e}")
            finally:
                self._loop.close()

    def start(self) -> "SyncAcrylMicrovm":
        """
        Start the sync runtime (non-context-manager usage).

        This is an alternative to using `with SyncAcrylMicrovm.default() as runtime:`.
        Useful for REPL sessions, test fixtures, or class-based lifecycle.

        Returns:
            Self, with dispatcher fiber running and runtime ready.

        Example:
            runtime = SyncAcrylMicrovm.default().start()
            box = runtime.create(BoxOptions(image="alpine:latest"))
            # ... use box ...
            runtime.stop()  # Don't forget to stop!
        """
        return self.__enter__()

    def stop(self) -> None:
        """
        Stop the sync runtime (non-context-manager usage).

        This cleans up the dispatcher fiber and event loop.
        Must be called if you used start() instead of the context manager.

        Example:
            runtime = SyncAcrylMicrovm.default().start()
            try:
                box = runtime.create(BoxOptions(image="alpine:latest"))
                # ... use box ...
            finally:
                runtime.stop()
        """
        self.__exit__(None, None, None)

    @staticmethod
    def init_default(options: "Options") -> None:
        """
        Initialize the global default runtime with custom options.

        This must be called before any SyncAcrylMicrovm.default() usage if you want
        to customize the default runtime (e.g., custom home_dir).

        Args:
            options: Runtime options to use for the default runtime.

        Example:
            SyncAcrylMicrovm.init_default(Options(home_dir="/custom/path"))
            with SyncAcrylMicrovm.default() as runtime:  # Now uses /custom/path
                ...
        """
        from ..acrylmicrovm import AcrylMicrovm

        AcrylMicrovm.init_default(options)

    @staticmethod
    def default() -> "SyncAcrylMicrovm":
        """
        Create a SyncAcrylMicrovm with default runtime options.

        This is the recommended way to create a SyncAcrylMicrovm instance.
        Mirrors async AcrylMicrovm.default().

        Returns:
            A new SyncAcrylMicrovm instance using the default runtime (~/.acryl/microvm).

        Example:
            with SyncAcrylMicrovm.default() as runtime:
                box = runtime.create(BoxOptions(image="alpine:latest"))
                ...
        """
        from ..acrylmicrovm import AcrylMicrovm

        return SyncAcrylMicrovm._around(AcrylMicrovm.default())

    @staticmethod
    def rest(options: "AcrylMicrovmRestOptions") -> "SyncAcrylMicrovm":
        """
        Create a SyncAcrylMicrovm on a remote AcrylMicrovm server.

        Mirrors async AcrylMicrovm.rest(): boxes and images are the server's.

        Example:
            with SyncAcrylMicrovm.rest(AcrylMicrovmRestOptions.from_env()) as runtime:
                for image in runtime.images.list():
                    ...
        """
        from ..acrylmicrovm import AcrylMicrovm

        return SyncAcrylMicrovm._around(AcrylMicrovm.rest(options))

    @staticmethod
    def _around(acrylmicrovm: "AcrylMicrovm") -> "SyncAcrylMicrovm":
        """A SyncAcrylMicrovm over an existing async runtime, not yet started."""
        instance = object.__new__(SyncAcrylMicrovm)
        instance._adopt(acrylmicrovm)
        return instance

    def _require_started(self) -> None:
        """Raise RuntimeError if runtime not started."""
        if not self._started:
            raise RuntimeError(
                "SyncAcrylMicrovm not started. Use 'with SyncAcrylMicrovm(...) as runtime:' "
                "or call 'SyncAcrylMicrovm.start()' first."
            )

    def _sync(self, coro):
        """Run async operation synchronously."""
        self._require_started()
        return self._sync_helper._sync(coro)

    # ─────────────────────────────────────────────────────────────────────────
    # Sync-capable runtime API
    # ─────────────────────────────────────────────────────────────────────────

    def create(
        self,
        options: "BoxOptions",
        name: str | None = None,
    ) -> "SyncBox":
        """
        Create a new box.

        Args:
            options: BoxOptions specifying image, resources, etc. General
                options are ignored when a box is reused; the local runtime
                additionally refuses to reuse a box whose capability policy
                differs from the requested one.
            name: Optional unique name for the box.

        Returns:
            SyncBox handle for the created box.

        Example:
            with SyncAcrylMicrovm.default() as runtime:
                box = runtime.create(BoxOptions(image="alpine:latest"))
        """
        self._require_started()
        from ._box import SyncBox

        native_box = self._sync(self._acrylmicrovm.create(options, name=name))
        return SyncBox(self, native_box)

    def get_or_create(
        self,
        options: "BoxOptions",
        name: str | None = None,
    ) -> tuple["SyncBox", bool]:
        """
        Get an existing box by name, or create a new one.

        Args:
            options: BoxOptions specifying image, resources, etc.
            name: Optional name to look up or assign. If None, always creates.

        Returns:
            Tuple of (SyncBox, created) where created is True if newly created.
        """
        self._require_started()
        from ._box import SyncBox

        native_box, created = self._sync(
            self._acrylmicrovm.get_or_create(options, name=name)
        )
        return SyncBox(self, native_box), created

    def get(self, id_or_name: str) -> Optional["SyncBox"]:
        """
        Get an existing box by ID or name.

        Args:
            id_or_name: Box ID or name to look up.

        Returns:
            SyncBox if found, None otherwise.
        """
        self._require_started()
        from ._box import SyncBox

        native_box = self._sync(self._acrylmicrovm.get(id_or_name))
        if native_box is None:
            return None
        return SyncBox(self, native_box)

    def metrics(self) -> "RuntimeMetrics":
        """
        Get runtime metrics.

        Returns:
            RuntimeMetrics with aggregate statistics.
        """
        self._require_started()
        return self._sync(self._acrylmicrovm.metrics())

    @property
    def images(self) -> "SyncImageHandle":
        """Get the runtime image handle."""
        self._require_started()
        from ._images import SyncImageHandle

        return SyncImageHandle(self, self._acrylmicrovm.images)

    @property
    def checkpoints(self) -> "SyncCheckpointHandle":
        """Manage REST checkpoints; local operations raise an unsupported error."""
        self._require_started()
        from ._checkpoints import SyncCheckpointHandle

        return SyncCheckpointHandle(self, self._acrylmicrovm.checkpoints)

    @property
    def domains(self) -> "SyncDomainHandle":
        """Get the organization's domain handle (REST runtimes only)."""
        self._require_started()
        from ._domains import SyncDomainHandle

        return SyncDomainHandle(self, self._acrylmicrovm.domains)

    def remove(self, id_or_name: str, force: bool = False) -> None:
        """
        Remove a box.

        Args:
            id_or_name: Box ID or name to remove.
            force: Force removal even if box is running.
        """
        self._sync(self._acrylmicrovm.remove(id_or_name, force))

    def shutdown(self, timeout: int | None = None) -> None:
        """
        Gracefully shutdown all boxes in this runtime.

        This method stops all running boxes, waiting up to `timeout` seconds
        for each box to stop gracefully before force-killing it.

        After calling this method, the runtime is permanently shut down and
        will return errors for any new operations (like `create()`).

        Args:
            timeout: Seconds to wait before force-killing each box:
                - None (default) - Use default timeout (10 seconds)
                - Positive integer - Wait that many seconds
                - -1 - Wait indefinitely (no timeout)
        """
        self._sync(self._acrylmicrovm.shutdown(timeout))

    # ─────────────────────────────────────────────────────────────────────────
    # Properties for internal use by SyncBox/SyncExecution
    # ─────────────────────────────────────────────────────────────────────────

    @property
    def loop(self) -> asyncio.AbstractEventLoop:
        """Get the event loop used by this runtime."""
        return self._loop

    @property
    def dispatcher_fiber(self) -> greenlet:
        """Get the dispatcher greenlet fiber."""
        return self._dispatcher_fiber

    @property
    def runtime(self) -> "AcrylMicrovm":
        """Get the underlying native AcrylMicrovm runtime (for internal/advanced use)."""
        return self._acrylmicrovm

    def __repr__(self) -> str:
        return f"SyncAcrylMicrovm({self._acrylmicrovm})"
