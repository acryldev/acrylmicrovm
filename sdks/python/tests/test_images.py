# Modified by ACRYL, 2026
"""
Integration tests for runtime image operations in the async Python SDK.
"""

from __future__ import annotations

import pytest

import acrylmicrovm

pytestmark = [pytest.mark.integration, pytest.mark.asyncio]


async def test_images_pull_returns_metadata(shared_runtime: acrylmicrovm.AcrylMicrovm):
    result = await shared_runtime.images.pull("alpine:latest")

    assert isinstance(result, acrylmicrovm.ImagePullResult)
    assert result.reference == "alpine:latest"
    assert result.config_digest.startswith("sha256:")
    assert result.layer_count > 0


async def test_images_list_returns_cached_image(shared_runtime: acrylmicrovm.AcrylMicrovm):
    await shared_runtime.images.pull("alpine:latest")

    images = await shared_runtime.images.list()

    assert images
    assert all(isinstance(info, acrylmicrovm.ImageInfo) for info in images)

    alpine = next(
        (
            info
            for info in images
            if "alpine" in info.repository and info.tag == "latest"
        ),
        None,
    )
    assert alpine is not None
    assert alpine.id.startswith("sha256:")
    assert isinstance(alpine.cached_at, str)


async def test_cached_image_handle_rejects_after_runtime_shutdown(tmp_path):
    runtime = acrylmicrovm.AcrylMicrovm(acrylmicrovm.Options(home_dir=str(tmp_path)))
    images = runtime.images

    await runtime.shutdown()

    with pytest.raises(RuntimeError, match="shut down"):
        await images.pull("alpine:latest")
