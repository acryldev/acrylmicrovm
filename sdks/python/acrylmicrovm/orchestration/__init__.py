# Modified by ACRYL, 2026
"""
Multi-box orchestration for AcrylMicrovm.

Example:
    from acrylmicrovm.orchestration import BoxRuntime

    async with BoxRuntime() as runtime:
        worker = await runtime.create_box(name="worker")

        @worker.on_message
        def handle(sender, data):
            return {"result": data["value"] * 2}

        await worker.run()
        await runtime.wait_all()
"""

from .box_runtime import BoxRuntime, ManagedBox

__all__ = ["BoxRuntime", "ManagedBox"]
