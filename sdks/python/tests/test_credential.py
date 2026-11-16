# Modified by ACRYL, 2026
"""Tests for the Credential abstraction.

Verifies: a `Credential` ABC with `ApiKeyCredential` virtual-registered,
`get_token()` returning a redacted `AccessToken`, and `from_env()`
discovery.

Marked `integration` because these exercise the native PyO3 classes
(`acrylmicrovm.ApiKeyCredential` / `AcrylMicrovmRestOptions`); the CI unit job
does not build the extension, so native-dependent tests run in the
integration job (same convention as test_options.py / test_images.py).
"""

from __future__ import annotations

import pytest

import acrylmicrovm

pytestmark = pytest.mark.integration


def test_api_key_credential_get_token():
    cred = acrylmicrovm.ApiKeyCredential("blk_live_secret")
    tok = cred.get_token()
    assert tok.token == "blk_live_secret"
    # API keys never expire — the core fetches once and caches.
    assert tok.expires_at is None


def test_api_key_credential_repr_is_masked():
    cred = acrylmicrovm.ApiKeyCredential("super-secret")
    assert "super-secret" not in repr(cred)
    assert repr(cred) == "ApiKeyCredential(***)"


def test_access_token_repr_is_masked():
    tok = acrylmicrovm.ApiKeyCredential("leak-me").get_token()
    assert "leak-me" not in repr(tok)


def test_is_instance_of_credential_abc():
    """The whole point of the abstraction: ApiKeyCredential is a
    nominal member of the Credential hierarchy via abc.register()."""
    cred = acrylmicrovm.ApiKeyCredential("k")
    assert isinstance(cred, acrylmicrovm.Credential)
    assert issubclass(acrylmicrovm.ApiKeyCredential, acrylmicrovm.Credential)


def test_from_env_unset_returns_none(monkeypatch):
    monkeypatch.delenv("ACRYLMICROVM_API_KEY", raising=False)
    assert acrylmicrovm.ApiKeyCredential.from_env() is None


def test_from_env_set_returns_credential(monkeypatch):
    monkeypatch.setenv("ACRYLMICROVM_API_KEY", "env-key")
    cred = acrylmicrovm.ApiKeyCredential.from_env()
    assert cred is not None
    assert cred.get_token().token == "env-key"
    assert isinstance(cred, acrylmicrovm.Credential)


def test_rest_options_takes_credential():
    opts = acrylmicrovm.AcrylMicrovmRestOptions(
        url="http://localhost:8100",
        credential=acrylmicrovm.ApiKeyCredential("k"),
    )
    assert opts.url == "http://localhost:8100"
    assert "***" in repr(opts)
    assert "k" not in repr(opts).replace("localhost", "")  # no secret leak


def test_rest_options_from_env(monkeypatch):
    monkeypatch.setenv("ACRYLMICROVM_REST_URL", "http://localhost:8100")
    monkeypatch.setenv("ACRYLMICROVM_API_KEY", "env-key")
    opts = acrylmicrovm.AcrylMicrovmRestOptions.from_env()
    assert opts.url == "http://localhost:8100"
    assert opts.credential is not None
    assert opts.credential.get_token().token == "env-key"


def test_rest_options_from_env_requires_url(monkeypatch):
    monkeypatch.delenv("ACRYLMICROVM_REST_URL", raising=False)
    with pytest.raises(RuntimeError):
        acrylmicrovm.AcrylMicrovmRestOptions.from_env()
