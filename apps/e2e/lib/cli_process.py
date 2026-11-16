# Modified by ACRYL, 2026
"""Run E2E CLI commands with separate output and visible failure diagnostics."""
from __future__ import annotations

import os
import subprocess


CLI_PROFILE = os.environ.get("ACRYLMICROVM_E2E_PROFILE", "p1")


def run(cli, *args, timeout: int = 60, stdin: str | None = None,
        check: bool = True) -> subprocess.CompletedProcess:
    result = subprocess.run(
        [cli, *args],
        timeout=timeout,
        input=stdin,
        text=True,
        capture_output=True,
        check=False,
        env={**os.environ, "ACRYLMICROVM_PROFILE": CLI_PROFILE},
    )
    if check and result.returncode != 0:
        raise AssertionError(
            f"CLI exited with status {result.returncode}\n"
            f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    return result
