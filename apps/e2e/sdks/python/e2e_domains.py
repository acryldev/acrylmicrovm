# Modified by ACRYL, 2026
"""One domain operation through either public Python API."""

import asyncio
import json
import os
import sys

import acrylmicrovm


def domain(value):
    assert not hasattr(value, "name"), "obsolete short name field is present"
    target = value.target
    return {
        "hostname": value.hostname,
        "url": value.url,
        "target": None
        if target is None
        else {
            "type": target.type,
            "box": target.box,
            "port": target.port,
        },
    }


def normalize(value, action):
    if action == "unbind":
        assert value == (), f"unbind returned {value!r}, expected ()"
        return {}
    if action == "list":
        return {
            "domains": [domain(d) for d in value.domains],
            "limit": value.limit,
            "used": value.used,
            "remaining": value.remaining,
            "min_length": value.min_length,
        }
    return domain(value)


def main():
    mode, action, *args = sys.argv[1:]
    options = acrylmicrovm.AcrylMicrovmRestOptions(
        url=os.environ["ACRYLMICROVM_E2E_URL"],
        credential=acrylmicrovm.ApiKeyCredential(os.environ["ACRYLMICROVM_E2E_API_KEY"]),
        path_prefix=os.environ["ACRYLMICROVM_E2E_PREFIX"],
    )

    def operation(domains):
        if action == "list":
            return domains.list()
        if action == "unbind":
            return domains.unbind(args[0])
        assert action == "bind", "unknown operation"
        return domains.bind(
            args[0],
            acrylmicrovm.DomainTarget(
                type="box",
                box=os.environ["ACRYLMICROVM_E2E_BOX"],
                port=int(args[1]),
            ),
        )

    async def run_async():
        runtime = acrylmicrovm.AcrylMicrovm.rest(options)
        try:
            return normalize(await operation(runtime.domains), action)
        finally:
            runtime.close()

    if mode == "sync":
        with acrylmicrovm.SyncAcrylMicrovm.rest(options) as runtime:
            result = normalize(operation(runtime.domains), action)
    else:
        result = asyncio.run(run_async())
    print(json.dumps(result))


if __name__ == "__main__":
    try:
        main()
    except acrylmicrovm.AcrylMicrovmError as exc:
        print(f"{type(exc).__name__}: {exc}", file=sys.stderr)
        sys.exit(1)
