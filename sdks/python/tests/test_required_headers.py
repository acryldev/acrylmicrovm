# Modified by ACRYL, 2026
"""Creating a secret requires an explicit header map."""

import importlib.util

import pytest

import acrylmicrovm

pytestmark = pytest.mark.skipif(
    importlib.util.find_spec("acrylmicrovm.acrylmicrovm") is None,
    reason="native extension not built",
)


def test_required_headers_constructor():
    with pytest.raises((TypeError, ValueError), match="headers"):
        acrylmicrovm.Secret(name="x", value="token", hosts=["api.test.com"])
