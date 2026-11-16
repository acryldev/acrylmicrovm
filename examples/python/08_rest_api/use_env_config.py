#!/usr/bin/env python3
# Modified by ACRYL, 2026
"""
Load REST connection config from environment variables.

Demonstrates:
- AcrylMicrovmRestOptions.from_env() for environment-based configuration
- API-key authentication via ACRYLMICROVM_API_KEY
- Useful for CI/CD pipelines and production deployments

Environment variables:
    ACRYLMICROVM_REST_URL          Server URL (required)
    ACRYLMICROVM_API_KEY           Long-lived API key
    ACRYLMICROVM_REST_PREFIX       API version prefix (default: v1)

Usage:
    ACRYLMICROVM_REST_URL=http://localhost:8100 \
    ACRYLMICROVM_API_KEY=your-api-key \
    python use_env_config.py

Prerequisites:
    make dev:python
    acrylmicrovm serve --port 8100
"""

import asyncio

from acrylmicrovm import AcrylMicrovm, AcrylMicrovmRestOptions


async def main():
    print("=" * 50)
    print("REST API: Environment-Based Configuration")
    print("=" * 50)

    # Load connection config from environment variables. from_env() reads
    # ACRYLMICROVM_REST_URL (required) and ACRYLMICROVM_API_KEY (wrapped in an
    # ApiKeyCredential automatically when set).
    try:
        opts = AcrylMicrovmRestOptions.from_env()
    except Exception as e:
        print(f"\n  Error: {e}")
        print("  Set ACRYLMICROVM_REST_URL and ACRYLMICROVM_API_KEY:")
        print("    ACRYLMICROVM_REST_URL=http://localhost:8100 \\")
        print("    ACRYLMICROVM_API_KEY=your-api-key \\")
        print("    python use_env_config.py")
        return

    print(f"\n  Loaded config: {opts}")

    rt = AcrylMicrovm.rest(opts)

    # Quick smoke test: list boxes
    boxes = await rt.list_info()
    print(f"  Boxes on server: {len(boxes)}")

    print("\n  Done")


if __name__ == "__main__":
    asyncio.run(main())
