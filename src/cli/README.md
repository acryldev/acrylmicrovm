# ACRYL microVMs CLI

Command-line interface for ACRYL microVMs — use ACRYL microVMs without writing code, with a familiar Docker/Podman-like experience.


For CLI development (build, test, adding commands), see [CLI Development Guide](../../docs/contributing/development/cli.md).


**Platforms:** macOS (Apple Silicon), Linux (x86_64, ARM64)

**Upgrade notice:** existing local boxes with secrets must be recreated with explicit
`headers`, including boxes managed by `acrylmicrovm serve`. See the
[required-headers migration notes](../../docs/reference/configuration.md#secrets).

## Overview

The ACRYL microVMs CLI (`acrylmicrovm`) lets you create, run, and manage ACRYL microVMs boxes from the terminal. It targets quick testing, shell scripting and automation, debugging, and demos.


### Key Features

- **Run** — Create a box from an image or prepared rootfs and run a command (interactive, TTY, or detached); supports `-p` (publish ports) and `-v` (volumes)
- **Create** — Create a box from an image or prepared rootfs without running; supports `-p` and `-v`
- **Lifecycle** — Start, stop, restart, remove boxes
- **Inspect** — Show detailed box info (JSON, YAML, or Go template)
- **Exec** — Run commands in a running or safely resumable box
- **Images** — Pull and list OCI images

- **Executions** — List live guest processes with `acrylmicrovm top BOX`; reattach with `acrylmicrovm attach BOX EXECUTION_ID`, or recover control with `acrylmicrovm kill BOX EXECUTION_ID` / `acrylmicrovm signal BOX EXECUTION_ID SIGNAL`.
- **Copy** — Copy files between host and box (`acrylmicrovm cp`)
- **Output formats** — Table, JSON, or YAML for list/images
- **Shell completion** — Bash, Zsh, Fish

The `acrylmicrovm agent` namespace groups nine commands for agents and turns,
including steering. BOX is positional; creation uses `--agent IMPLEMENTATION`.
Other commands use `--agent ID` and `--turn TURN` to select existing work.
See the [agent CLI reference](../../docs/reference/cli/README.md#acrylmicrovm-agent)
for commands, grouped options and error behavior.

## GitHub CI connect (in development)

`acrylmicrovm --context dev ci connect GITHUB_ACCOUNT` binds a GitHub App installation
using the selected remote ACRYL microVMs context and a browser device authorization.
Public repositories are enabled by default. Use `--allow-private-repositories` to include private repositories, and `--no-browser`
to print the authorization URL without opening it. The account may be a personal
GitHub account or organization.

The command requires ACRYL microVMs organization ownership and GitHub installation ownership.
An API key needs `write:boxes` and `delete:boxes`. Current and future App-selected
repositories follow the binding. A conflicting binding is rejected rather than replaced.

The authorization poll is a POST without a body, so the CLI declares `Content-Length: 0` on it.
Some gateways answer a POST without a length with HTTP 411 before it reaches the API.

After connecting, use `runs-on: acrylmicrovm` for the configured worker defaults or
`runs-on: acrylmicrovm-2vcpu-4gbmem-10gbdisk` for 2 vCPU, 4 GiB RAM and 10 GiB disk.
Both hosted and standalone execution enforce the backend's resource and quota
checks.

The current API reports `provisioning` after binding; runner capacity is not ready yet.
The API deployment also needs the CI Login OAuth App credentials for authenticated GitHub account-type lookup; user authorization still uses the matching Personal or Organization CI App.

CI logs and results remain in GitHub
Actions; running jobs use ordinary Box management commands.

## Installation

### One-line install (Linux & macOS Apple Silicon)

```bash
curl -fsSL https://sh.acryl.dev | sh
```

Installs to `$HOME/.local/bin/acrylmicrovm`. For version pinning, a custom
install dir, and release-artifact verification (sigstore, `SHA256SUMS`,
`gh attestation verify`), see the [CLI Reference's Installation & Verification section](../../docs/reference/cli/README.md#installation--verification).

### cargo install (from source)

```bash
cargo install acrylmicrovm-cli
```

### cargo binstall (prebuilt binary)

```bash
cargo binstall acrylmicrovm-cli
```

### Homebrew

Coming soon

### Build from Source

```bash
# From repository root
git clone https://github.com/acryldev/acrylmicrovm.git
cd acrylmicrovm

# Initialize submodules (required)
git submodule update --init --recursive

# Build the CLI
cargo build --release -p acrylmicrovm-cli

# Binary: target/release/acrylmicrovm
```

### System Requirements

| Platform       | Architecture          | Status           |
|----------------|-----------------------|------------------|
| macOS          | Apple Silicon (ARM64) | ✅ Supported     |
| Linux          | x86_64                | ✅ Supported     |
| Linux          | ARM64                 | ✅ Supported     |
| Windows (WSL2) | x86_64                | ✅ Supported     |
| macOS          | Intel (x86_64)        | ❌ Not supported |


## Quick Start

### Run a one-off command

```bash
acrylmicrovm run python:slim python -c 'print("Hello from AcrylMicrovm!")'
```

### Run interactively with a TTY

```bash
acrylmicrovm run -it alpine:latest /bin/sh
```

### Create a box and run in the background

```bash
# Create and start (prints box ID)
acrylmicrovm run -d --name mybox alpine:latest sleep 3600

# Run a command in the box
acrylmicrovm exec mybox echo "Hello"

# List boxes
acrylmicrovm list -a

# Stop and remove
acrylmicrovm stop mybox
acrylmicrovm rm mybox
```

### Pull an image and list images

```bash
acrylmicrovm pull alpine:latest
acrylmicrovm images
```

## Checkpoint management

Only `--name` may accompany `create --checkpoint` among creation flags.
Explicit defaults such as `--auto-delete 0` and `--network enabled` also reject;
global connection options remain available.

Checkpoints are managed by the selected REST server. Local runtimes return
an unsupported-operation error. Each checkpoint captures a stopped box's disk
and execution configuration, survives source deletion, and can create multiple
boxes. It does not capture memory or running processes.

```bash
# Use a REST context, or pass --url with the selected server.
acrylmicrovm --url https://api.example.com stop source
acrylmicrovm --url https://api.example.com checkpoint create source --name dev-base
acrylmicrovm --url https://api.example.com checkpoint list
acrylmicrovm --url https://api.example.com checkpoint get dev-base

# Saved execution settings are inherited; only --name may accompany --checkpoint.
acrylmicrovm --url https://api.example.com create --checkpoint dev-base --name worker
acrylmicrovm --url https://api.example.com checkpoint delete dev-base
```

A capture can initially report `creating`; wait for `ready` before use.
Capture never automatically stops a Box. `create --checkpoint` refuses an image,
`--rootfs` and every other create option before sending a request. Deleting a
checkpoint leaves boxes created from it intact; `--force` tolerates absence and
never bypasses active use.

A checkpoint is a rootfs source for `create`, beside an image or `--rootfs`, so
the `checkpoint` subcommands only manage checkpoints. The CLI and SDKs do not
replace an existing box from a checkpoint; remove the box and create a new one.

Storage and capture coordination belong to the server.
The SDK's `CheckpointHandle` only validates inputs and sends REST requests.
The separate `import ARCHIVE` command keeps the native archive workflow unchanged.
Local `serve` reports `checkpoints_enabled: false` and exposes no checkpoint
management routes. Existing binary archive export/import remains available.
The cloud runner verifies the downloaded artifact SHA before ordinary native
import, which owns archive validation; this creates no local
checkpoint catalog or recovery records.

## Connecting to a remote server

To target a remote ACRYL microVMs REST server instead of the local runtime, sign in
with `acrylmicrovm auth login`. Three login methods are supported:

| Method      | When to use                                            | Token type           |
|-------------|--------------------------------------------------------|----------------------|
| `api-key`   | CI / automation, or a server-minted long-lived key     | Opaque `blk_…` key   |
| `browser`   | Local developer machine with a desktop browser         | OIDC access token    |
| `device`    | SSH session / headless container with no browser       | OIDC access token    |

Create a named context with `auth login`, then select it once with
`acrylmicrovm context use NAME`. Subsequent commands use that context automatically.
`ACRYLMICROVM_API_KEY` overrides the selected context's bearer credential; its URL and
tenant path remain in effect.

```bash
# Browser OIDC against a control plane (default for `auth login` on a TTY).
# Requires the IdP admin to have registered `http://127.0.0.1:5555/callback`
# in the SPA application's Allowed Callback URLs.
acrylmicrovm --context cloud auth login --url https://<your-control-plane>/api
acrylmicrovm context use cloud
acrylmicrovm list

# Same target, headless (SSH / no browser): prints a code + URL to type into
# any browser on another device.
acrylmicrovm --context cloud auth login --url https://<your-control-plane>/api --no-browser

# Local acrylmicrovm serve with a static API key (unchanged from the original behavior).
acrylmicrovm --context local auth login --url http://localhost:8100  # interactive paste
echo "$KEY" | acrylmicrovm --context local auth login --url http://localhost:8100 --api-key-stdin

# CI via env vars only — no `auth login` call needed.
ACRYLMICROVM_API_KEY=$KEY ACRYLMICROVM_REST_URL=https://<your-server> acrylmicrovm list
```

Credentials are stored at `<ACRYLMICROVM_HOME>/credentials.toml` (default
`~/.acryl/microvm/credentials.toml`, perms `0600`). OIDC
sessions auto-refresh on use when within 5 minutes of expiry; if the refresh
token is rejected (`invalid_grant`) the CLI prompts you to re-run `auth login`.

`acrylmicrovm context ls` marks the effective context with `*`; `acrylmicrovm context show`
prints its name. `--context NAME` or `ACRYLMICROVM_CONTEXT=NAME` overrides the saved
selection without changing it. The old `--profile` and `ACRYLMICROVM_PROFILE` spellings
remain supported. Login saves credentials but does not change the current context.

`acrylmicrovm context use default` restores the pre-context configuration: a URL
override or existing `default` credentials select REST; otherwise commands use
the embedded runtime. `acrylmicrovm context use embedded` always selects the embedded
runtime, including after a `default` login; `auth login` cannot store
credentials under that name. Context selection is stored separately in
`<ACRYLMICROVM_HOME>/cli.toml`. See [context selection and precedence](../../docs/reference/cli/README.md#acrylmicrovm-context)
for URL overrides and missing contexts.

## Commands Reference

> For an exhaustive man-page-style reference (shared flag groups, volume/port grammar,
> exit codes, configuration file format), see the
> [CLI Reference](../../docs/reference/cli/README.md).

### Global flags

Place these options before the top-level command or after the complete command
path. A parent command accepts only options shared by all of its children, so
write `acrylmicrovm auth status --url URL` (or `acrylmicrovm --url URL auth status`), not
`acrylmicrovm auth --url URL status`. Each command's help shows only options it can
use. Executing invocations reject unsupported options; Clap's `--help` and
`--version` display actions remain terminal.

| Flag | Scope | Description |
|------|-------|-------------|
| `--debug` | all commands except `completion` | Enable debug output. Precedence: `--debug` > `RUST_LOG` env > default (`warn`). |
| `--home PATH` | local runtime + credentials + contexts | Absolute ACRYL microVMs home directory (default: `~/.acryl/microvm`). `ACRYLMICROVM_HOME` is the env spelling. |
| `--registry REGISTRY` | `run`, `create`, `pull`, `serve` | Image registry (repeatable; prepended to config). |
| `--config PATH` | local-capable commands | JSON config file path (e.g. for `home_dir` and `image_registries`). |
| `--url URL` | REST-capable commands | Connect to a remote ACRYL microVMs REST server instead of the local runtime. Env: `ACRYLMICROVM_REST_URL`. |
| `--context NAME` | REST-capable commands + `auth` + `context` | Override the saved current context. Env: `ACRYLMICROVM_CONTEXT`. Legacy aliases: `--profile`, `ACRYLMICROVM_PROFILE`. |
| `--path-prefix VALUE` | REST box/volume commands | Routing-slot value for `/v1/<prefix>/...`; overrides the selected context. `--path-prefix=''` clears it. Env: `ACRYLMICROVM_REST_PATH_PREFIX` (empty also clears). |

### Network policy updates

```bash
acrylmicrovm network inbound mybox enabled
acrylmicrovm network outbound mybox enabled --allow-net '*.example.com'
acrylmicrovm network outbound mybox enabled  # default empty list: unrestricted
acrylmicrovm network outbound mybox disabled
acrylmicrovm inspect mybox
```

The box and mode are required. Repeat `--allow-net` for multiple outbound rules.
Omit `--allow-net` for the default empty list; `''` is rejected, as at create.
Inbound accepts no list; disabled outbound accepts no `--allow-net`. Each command replaces
one direction without retaining earlier rules. See [networking](../../docs/guides/networking.md#updating-a-running-box)
for runtime support, connection lifetime and persistence.

### Guest SSH

```bash
acrylmicrovm run -d --name dev alpine:latest tail -f /dev/null
acrylmicrovm ssh dev
```

With no operation flag, `acrylmicrovm ssh dev` prepares CLI-managed Ed25519 keys and
logs in as `box` using system OpenSSH. Both `ssh` and `ssh-keygen` must be on PATH.
Saved keys are reused; healthy SSH with a matching host identity is not
reconfigured. Changed host identities are rejected rather than overwritten.
Choose one operation with `--install-ssh-config`, `--configure`, `--status`,
`--enable`, `--disable`, `--tunnel`, or `--proxy`; all support local and remote boxes.
`--configure` retains full configuration control and prints status JSON.

To connect later with plain OpenSSH, install an alias once:

```bash
acrylmicrovm ssh dev --install-ssh-config
ssh dev
acrylmicrovm --context cloud ssh dev --install-ssh-config --alias dev-cloud
ssh dev-cloud
```

Installation prepares SSH and exits without logging in. The default alias is the
resolved box name, or its full ID for unnamed boxes. Reinstalling the same target
updates its entry; a different target requires `--alias NAME` or `--overwrite`.
Overwrite replaces only ACRYL microVMs-managed entries, never handwritten Host blocks.
The CLI adds one Include to `~/.ssh/config` and keeps its entries separately in
`~/.acryl/microvm/ssh/config`. Installed aliases pin home, backend, endpoint, context and
routing, while credentials remain in the existing store. User and system SSH
settings still participate, including personal identities and Match conditions;
installation does not guarantee complete configuration isolation. See the
[installation guide](../../docs/guides/ssh.md#managed-openssh-aliases) for safety
checks and lifecycle behavior.

`--proxy` forwards only stdin/stdout; `--tunnel` remains available for a local TCP
listener. Both require enabled SSH. `--enable` restores saved SSH configuration;
`--disable` stops SSH while retaining credentials. Both require a running box.
All SSH operations accept cloud contexts, for example
`acrylmicrovm --context cloud ssh dev`. Default login uses the resolved context,
authentication, endpoint and routing prefix for its entire session.
It pins the local managed host key in a private `known_hosts` file and ignores
user OpenSSH configuration without modifying it.
Cloud login requires SSH control, tunnel support and inbound access;
`acrylmicrovm serve` supports explicit controls but has no tunnel endpoint.
A local login, proxy or tunnel holds the runtime home lock until it ends.
The default command returns the SSH exit code; non-detached boxes remain alive
until the session ends, then stop when the CLI exits.
See [`acrylmicrovm ssh`](../../docs/reference/cli/README.md#acrylmicrovm-ssh) for flags and the
[SSH guide](../../docs/guides/ssh.md#cli) for lifecycle rules.

### `acrylmicrovm auth login`

Log in to a ACRYL microVMs REST server. Supports three flows that all save to
`<ACRYLMICROVM_HOME>/credentials.toml` (default `~/.acryl/microvm/credentials.toml`,
perms `0600`):

- **API key** — paste or stdin. Long-lived, org-scoped. Good for CI, SDK
  integrations, `acrylmicrovm serve` setups.
- **Browser OIDC** — Authorization Code + PKCE against the IdP that
  the control plane is configured for (Auth0, Dex, Okta, etc.). Opens the system
  browser; the CLI listens on `127.0.0.1:5555` for the callback. Mints an
  access token + refresh token; the latter is used to silently refresh
  within 5 minutes of expiry.
- **Device code OIDC** — RFC 8628. Headless / SSH-friendly: prints a short
  code + URL to enter on any browser on another device. Same token type
  and refresh behavior as the browser flow.

On a ACRYL microVMs Auth0 deployment, an existing unverified database account must
complete the hosted verification Form through browser login:

```bash
acrylmicrovm --context cloud auth login --url https://<your-control-plane>/api --method browser
```

Auth0 Forms cannot run inside refresh-token, device-code, or other
non-interactive exchanges. An SSH user should finish verification through the
ACRYL microVMs dashboard in a separate browser, then retry the device login.

When `--method` is unset, the CLI infers it: piped stdin → `api-key` (CI-safe),
while a TTY opens the interactive picker. If Browser is selected from that
picker in a detected headless/SSH environment, the CLI falls back to `device`.

**Usage:** `acrylmicrovm auth login [OPTIONS]`

| Option | Description |
|--------|-------------|
| `--url URL` | Server URL (precedence: flag, `ACRYLMICROVM_REST_URL`, selected context URL, then `https://api.webboxes.ai`). Self-hosted deployments may require a base path such as `/api`. |
| `--method <api-key\|browser\|device>` | Explicit flow choice. Overrides inference. |
| `--api-key-stdin` | Read the API key from stdin (one line). The flag takes no value, so the secret never appears on argv. Selects API-key login and conflicts with explicit browser/device methods. |
| `--no-browser` | Use device code when no method is explicit; conflicts with API-key stdin and explicit API-key/browser methods. |
| `--callback-port <PORT>` | Local port for the browser-flow callback (default `5555`). **Must match an entry in the IdP's allow-list byte-for-byte** — a different port produces "Callback URL mismatch" exactly like no entry at all. |
| `--issuer URL` | OIDC issuer URL. Overrides what `GET /api/config` returns. Useful for self-hosted Dex tenants where the discovery is wrong. |
| `--client-id ID` | OIDC client_id. Overrides `/api/config`. |
| `--audience VAL` | OIDC audience. Auth0 requires it; Dex tolerates `None`. |

`--context NAME` selects the context to create or update; when omitted, login
uses the effective current context. Login accepts both `--context` and `--url`
because the URL configures that context. `--home PATH` selects the credential and
context preference directory. Login does not switch the saved current context.

**Examples:**

```bash
# Cloud control plane via browser (most common; opens system browser).
acrylmicrovm --context cloud auth login --url https://<your-control-plane>/api

# Same target, headless: prints a code + URL.
acrylmicrovm --context cloud auth login --url https://<your-control-plane>/api --no-browser

# Local acrylmicrovm serve with paste-API-key (interactive).
acrylmicrovm --context local auth login --url http://localhost:8100

# CI: API key from stdin (nothing on argv).
echo "$KEY" | acrylmicrovm --context local auth login --url http://localhost:8100 --api-key-stdin
```

**Deployment-side setup (one-time, by the IdP admin):**

Browser PKCE login needs the loopback callback `http://127.0.0.1:5555/callback`
registered in the client's **Allowed Callback URLs**. Device login does not use
that callback. It uses the provider's discovered `device_authorization_endpoint`;
older Dex deployments without this field retain the `/device/code` fallback.

For Auth0, configure a public **Native** CLI application with the **Device Code**
grant enabled. Enable **Refresh Token** for the client and **Allow Offline Access**
for the target API to issue renewable sessions. Pass its public ID with
`--client-id` if `/api/config` advertises the dashboard SPA application. Flags
win over server values; `OIDC_CLIENT_ID` is a fallback, not an override for
`/api/config`. Do not provide a client secret. An `unauthorized_client` error
can mean an unknown/wrong client or a disabled grant: check the CLI client ID
and issuer, then ask the IdP administrator to verify the Device Code grant.
For Dex, enable `oauth2.deviceFlow: {}` in `apps/dex/config.yaml`.

Device login prints the verification URL and one-time user code on stderr.
Complete authorization in a browser while the CLI polls. Ctrl-C cancels polling;
denied, expired, and cancelled attempts do not save a partial session. Retry
`auth login` to obtain a new code. Server response descriptions and bodies are
excluded from device-login errors to avoid exposing credentials.

### `acrylmicrovm auth logout`

Remove the selected context from `<ACRYLMICROVM_HOME>/credentials.toml` (default
`~/.acryl/microvm/credentials.toml`). Prompts for confirmation unless `--yes` is given.
The current context stays selected; commands fail if its credentials are missing.
Log in again or run `acrylmicrovm context use default` to switch back explicitly.

**Usage:** `acrylmicrovm auth logout [OPTIONS]`

| Option | Short | Description |
|--------|-------|-------------|
| `--yes` | `-y` | Skip the confirmation prompt |

### `acrylmicrovm auth status`

Print the current authentication state: the logged-in URL, the source
(stored file vs env var), the credential type (API key vs OIDC), and for
OIDC sessions the access token's expiry. Offline — no network calls,
no secret material printed. With only `ACRYLMICROVM_API_KEY` set, it reports
`https://api.webboxes.ai`.

**Usage:** `acrylmicrovm auth status [--url URL | --context NAME]`

**Example output (API key):**

```
Context:         local (from saved context)
Logged in to:    http://localhost:8100
Credential:      API key (from <ACRYLMICROVM_HOME>/credentials.toml [local])
```

**Example output (OIDC session):**

```
Context:         cloud (from saved context)
Logged in to:    https://api.webboxes.ai
Credential:      OIDC bearer token (from <ACRYLMICROVM_HOME>/credentials.toml [cloud])
Expires:         2026-05-21T15:42:00+00:00
```

### `acrylmicrovm auth whoami`

Confirm the active credential's identity by making one authenticated
request to `GET /v1/me`. Unlike `auth status` (offline, only reports where
the credential came from), `whoami` shows the server-resolved principal,
path prefix, and scopes. Triggers a silent OIDC refresh if the access
token is within 5 minutes of expiry.

**Usage:** `acrylmicrovm auth whoami [--url URL | --context NAME]`

**Example output:**

```
Logged in as:    dev@acme.test
Name:            Dev McAcme
Principal:       auth0|abc123 (user)
Path prefix:     acme
Server:          https://api.webboxes.ai
Scopes:          box:read, box:write, box:exec, image:read, snapshot:read
```

### `acrylmicrovm run`

Create a box from an image (or a prepared rootfs via `--rootfs`) and run a
command, with docker's semantics: `COMMAND` replaces the image's `CMD`, the
image's `ENTRYPOINT` is prepended, and the result **is** the container's init
(PID 1). Omit it and the image's own default runs.

The box lives exactly as long as that command. When it exits, the box stops and
takes its exit code — `acrylmicrovm inspect -f '{{.State.ExitCode}}' NAME` reads it
back. `exec` against a box whose command has finished is refused rather than
silently restarting it, because restarting would run the command a second time.

If the init command is missing or non-executable, startup fails: the CLI prints
`Error: ...` to stderr and exits `1`. Only `exec` reports these startup refusals
as completed executions with status `126` or `127`.

**Usage:**

- `acrylmicrovm run [OPTIONS] IMAGE [COMMAND]...`
- `acrylmicrovm run [OPTIONS] --rootfs PATH [COMMAND]...`

| Option | Short | Description |
|--------|-------|-------------|
| `--rootfs PATH` | | Use a prepared rootfs path instead of pulling/resolving an image |
| `--interactive` | `-i` | Keep STDIN open; conflicts with `--detach` |
| `--tty` | `-t` | Allocate a pseudo-TTY; requires TTY-attached stdin, even with `--detach` |
| `--env KEY[=VALUE]` | `-e` | Set environment variables; a bare key inherits its host value (repeatable) |
| `--workdir PATH` | `-w` | Working directory in the box |
| `--user NAME[:GROUP]` | `-u` | Run as a name/uid and optional group/gid |
| `--entrypoint EXEC` | | Override the image entrypoint |
| `--publish PORT` | `-p` | Publish a TCP box port locally (`80` = automatic host port, `8080:80` = fixed) |
| `--volume VOLUME` | `-v` | Mount a volume: `name:/box` for a managed volume, `./path:/box` for a host bind, `/box` for anonymous |
| `--cpus N` | | CPU allocation, e.g. `2` or `0.5`; fractional values require a Linux execution host |
| `--memory MiB` | | Memory limit (MiB) |
| `--disk-size GB` | | Sparse rootfs disk size; smaller values than the base image are ignored |
| `--cap-add CAPABILITY` | | Add a Linux capability (repeatable; accepts `CAP_` prefix or `ALL`) |
| `--cap-drop CAPABILITY` | | Drop a Linux capability (repeatable; accepts `CAP_` prefix or `ALL`) |
| `--network <enabled\|disabled>` | | Outbound network mode (default `enabled`) |
| `--allow-net HOST` | | Restrict egress to an exact host, wildcard domain, IPv4, or IPv4 CIDR; repeatable and implies enabled networking |
| `--inbound <enabled\|disabled>` | | Inbound network mode (default `disabled`) |
| `--net-tx-kbps KBPS` | | Cap what the box sends, in kilobits/sec; `0` or unset is uncapped. Local runtime only; verified on Linux |
| `--net-rx-kbps KBPS` | | Cap what reaches the box, in kilobits/sec; `0` or unset is uncapped. Local runtime only; verified on Linux |
| `--name NAME` | | Name the box |
| `--detach` | `-d` | Run in background, print box ID |
| `--rm` | | Remove the box when it stops; conflicts with lifecycle deadlines and is ignored with `--detach` |
| `--auto-stop DURATION` | | Stop the box after this much inactivity; `0` disables |
| `--auto-delete DURATION` | | Delete the box this long after it stops; `0` disables |
| `--no-auto-resume` | | Ask a REST server to refuse implicit resume after a box has run; first boot is unaffected, and the embedded runtime records but does not enforce it |
| `--security <enable\|disable>` | | Embedded-runtime sandbox security (default `enable`; env `ACRYLMICROVM_SECURITY`); REST servers own this policy |

`-p` is explicit local publication. Remote REST contexts reject it and direct
the caller to `acrylmicrovm network tunnel`.

`DURATION` is seconds when bare, or suffixed: `30s`, `15m`, `2h`, `7d`. The two
deadlines are swept by a server — `acrylmicrovm serve` or the cloud — so against the
embedded runtime they are refused rather than silently reinterpreted. See the
[CLI reference](../../docs/reference/cli/README.md) for the sweep's granularity
and how `--rm` differs between the two.

**Examples:**

```bash
acrylmicrovm run alpine:latest echo "Hello"
acrylmicrovm run -it --rm alpine:latest /bin/sh
acrylmicrovm run -d --name openclaw -p 18789:18789 ghcr.io/openclaw/openclaw:main
acrylmicrovm run -v /host/data:/app/data alpine:latest cat /app/data/hello.txt
acrylmicrovm run --rootfs /path/to/rootfs /bin/sh
acrylmicrovm run --cap-drop ALL --cap-add NET_BIND_SERVICE nginx:alpine
```

### `acrylmicrovm create`

Create a box without starting it.

`COMMAND` is stored, not run — it becomes the box's main command when the box is
next started, exactly as under `run`. A box created *with* one is a job, and
`exec`/`cp` will not start it implicitly (that would run the command); start it
deliberately with `acrylmicrovm start`. A box created *without* one boots the image's
default, and `exec` still starts it on demand.

**Usage:**

- `acrylmicrovm create [OPTIONS] IMAGE [COMMAND]...`
- `acrylmicrovm create [OPTIONS] --rootfs PATH [COMMAND]...`

| Option | Short | Description |
|--------|-------|-------------|
| `--rootfs PATH` | | Use a prepared rootfs path instead of pulling/resolving an image |
| `--name NAME` | | Name the box |
| `--env KEY[=VALUE]` | `-e` | Environment variables; a bare key inherits its host value |
| `--workdir PATH` | `-w` | Working directory |
| `--entrypoint EXEC` | | Override the image entrypoint |
| `--publish PORT` | `-p` | Publish a TCP box port locally (`80` = automatic host port, `8080:80` = fixed) |
| `--volume VOLUME` | `-v` | Mount a volume: `name:/box` for a managed volume, `./path:/box` for a host bind, `/box` for anonymous |
| `--cpus N` | | CPU allocation, e.g. `2` or `0.5`; fractional values require a Linux execution host |
| `--memory MiB` | | Memory limit (MiB) |
| `--disk-size GB` | | Sparse rootfs disk size; smaller values than the base image are ignored |
| `--cap-add CAPABILITY` | | Add a Linux capability (repeatable; accepts `CAP_` prefix or `ALL`) |
| `--cap-drop CAPABILITY` | | Drop a Linux capability (repeatable; accepts `CAP_` prefix or `ALL`) |
| `--network <enabled\|disabled>` | | Outbound network mode (default `enabled`) |
| `--allow-net HOST` | | Restrict egress to an exact host, wildcard domain, IPv4, or IPv4 CIDR; repeatable and implies enabled networking |
| `--inbound <enabled\|disabled>` | | Inbound network mode (default `disabled`) |
| `--net-tx-kbps KBPS` | | Cap what the box sends, in kilobits/sec; `0` or unset is uncapped. Local runtime only; verified on Linux |
| `--net-rx-kbps KBPS` | | Cap what reaches the box, in kilobits/sec; `0` or unset is uncapped. Local runtime only; verified on Linux |
| `--auto-stop DURATION` | | Stop the box after this much inactivity; `0` disables; requires a REST server |
| `--auto-delete DURATION` | | Delete the box this long after it stops; `0` disables; requires a REST server |
| `--no-auto-resume` | | Ask a REST server to refuse implicit resume after a box has run; first boot is unaffected, and the embedded runtime records but does not enforce it |
| `--security <enable\|disable>` | | Embedded-runtime sandbox security (default `enable`; env `ACRYLMICROVM_SECURITY`); REST servers own this policy |

`-p` is explicit local publication. Remote REST contexts reject it and direct
the caller to `acrylmicrovm network tunnel`.

**Examples:**

```bash
acrylmicrovm create --name mybox alpine:latest
acrylmicrovm create -p 18789:18789 -v /data:/app/data --name openclaw ghcr.io/openclaw/openclaw:main
acrylmicrovm create --rootfs /path/to/rootfs --name local-rootfs
acrylmicrovm create --cap-drop NET_RAW --name hardened alpine:latest
acrylmicrovm start mybox
acrylmicrovm start openclaw
```

### `acrylmicrovm exec`

Run a command in a box. A safely resumable stopped box starts implicitly; a
job box whose start would run or re-run its user-selected main command is
refused, so start that box deliberately with `acrylmicrovm start` first.

**Usage:** `acrylmicrovm exec [OPTIONS] BOX -- COMMAND [ARGS]...`

| Option | Short | Description |
|--------|-------|-------------|
| `--interactive` | `-i` | Keep STDIN open; conflicts with `--detach` |
| `--tty` | `-t` | Allocate a TTY; foreground mode requires TTY-attached stdin, while detached mode attaches no stdin |
| `--env KEY[=VALUE]` | `-e` | Environment variables; a bare key inherits its host value |
| `--workdir PATH` | `-w` | Working directory |
| `--user NAME[:GROUP]` | `-u` | Run as a name/uid and optional group/gid |
| `--detach` | `-d` | Run in background (don’t wait) |

**Example:**

```bash
acrylmicrovm exec -it mybox -- /bin/sh
```

For `exec`, a command the guest refuses to start is a completed execution:
foreground `acrylmicrovm exec` exits `127` when the executable is not found and
`126` when it exists but is not executable. The guest's diagnostic goes to
stderr without a TTY, or to merged stdout with `--tty`. The same execution
statuses and streams come back through `acrylmicrovm serve` over REST. Rejection of
the box's init command during `run` remains a CLI error with exit code `1`.

`126`/`127` are the special codes for an `exec` startup refusal; every other
value keeps its usual meaning.

### `acrylmicrovm list` (alias: `ls`, `ps`)

List boxes.

**Usage:** `acrylmicrovm list [OPTIONS]`

| Option | Short | Description |
|--------|-------|-------------|
| `--all` | `-a` | Show all boxes (default: running only) |
| `--quiet` | `-q` | Show only IDs; conflicts with `--format` |
| `--format FMT` | | Output format: `table`, `json`, `yaml`; conflicts with `--quiet` (default: `table`) |

### `acrylmicrovm start`

Start one or more stopped boxes.

**Usage:** `acrylmicrovm start BOX [BOX ...]`

### `acrylmicrovm stop`

Stop one or more running boxes.

**Usage:** `acrylmicrovm stop BOX [BOX ...]`

### `acrylmicrovm restart`

Restart one or more boxes.

**Usage:** `acrylmicrovm restart BOX [BOX ...]`

### `acrylmicrovm rm`

Remove one or more boxes.

**Usage:** `acrylmicrovm rm [OPTIONS] BOX [BOX ...]` or `acrylmicrovm rm [OPTIONS] --all`

| Option | Short | Description |
|--------|-------|-------------|
| `--force` | `-f` | Force remove (e.g. running box) |
| `--all` | `-a` | Remove all boxes (prompts unless `--force`) |

### `acrylmicrovm pull`

Pull an image from a registry.

**Usage:** `acrylmicrovm pull [OPTIONS] IMAGE`

| Option | Short | Description |
|--------|-------|-------------|
| `--quiet` | `-q` | Only print digest |

### `acrylmicrovm inspect`

Display detailed information on one or more boxes (JSON, YAML, or Go-style template).

**Usage:** `acrylmicrovm inspect [OPTIONS] [BOX ...]` or `acrylmicrovm inspect --latest`

| Option | Short | Description |
|--------|-------|-------------|
| `--latest` | `-l` | Inspect the most recently created box (cannot be used with BOX) |
| `--format FMT` | `-f` | Output: `json`, `yaml`, or a Go template (e.g. `{{.State.Status}}`, `{{.Id}}`). Default: `json`. Table format is not supported. |

`Network` holds both directions of the box's policy, `Outbound` and `Inbound`,
each with `Mode` (`enabled` or `disabled`) and `AllowNet`. It is `null` when the
runtime does not know the policy, such as a REST server that predates it.

**Examples:**

```bash
acrylmicrovm inspect mybox
acrylmicrovm inspect -f '{{.State.Status}}' mybox
acrylmicrovm inspect -f '{{.Network.Inbound.Mode}}' mybox
acrylmicrovm inspect --latest -f yaml
acrylmicrovm inspect box1 box2 -f json
```

### `acrylmicrovm images`

List cached images.

**Usage:** `acrylmicrovm images [OPTIONS]`

| Option | Short | Description |
|--------|-------|-------------|
| `--quiet` | `-q` | Show only image IDs; conflicts with `--format` |
| `--format FMT` | | Output format: `table`, `json`, `yaml`; conflicts with `--quiet` |

### `acrylmicrovm cp`

Copy files or directories between host and box.

**Usage:** `acrylmicrovm cp [OPTIONS] SRC DST`

- **SRC / DST:** host path or `BOX:PATH` (e.g. `mybox:/app/data`). A directory `SRC` lands as
  `docker cp` lands it: it becomes a `DST` that does not exist, and lands under its own name in
  one that does. End `SRC` with `/.` to copy its contents either way.

| Option | Description |
|--------|-------------|
| `--follow-symlinks` | Follow symlinks when copying (local runtime only) |
| `--no-overwrite` | Do not overwrite existing files (local runtime only) |

**Examples:**

```bash
acrylmicrovm cp ./local.txt mybox:/workspace/
acrylmicrovm cp mybox:/app/out ./output
acrylmicrovm cp ./dist/. mybox:/srv            # the contents of ./dist, not dist/ itself
```

Paths at or under a mount inside the box are refused in both directions — `/tmp`,
`/dev/shm`, volumes, and the `/etc/hosts`, `/etc/hostname`, `/etc/resolv.conf` binds.
`cp` works on the rootfs layer from outside the box's mount namespace, so such a
copy would write where nothing in the box can see it, or read back the image's file
rather than the one the box has.

A directory that merely *contains* a mount differs by direction: copying one **out** is
refused (`acrylmicrovm cp box:/etc ./etc`), while copying **in** is allowed and refused only
if a file would land on a mount — so `acrylmicrovm cp ./x box:/etc` works.

Copy a path outside the mount, or pipe a tar through
`acrylmicrovm exec`. Files copied in are owned by the box's exec user.

### `acrylmicrovm info`

Display system-wide runtime information (version, paths, host/virtualization, box and image counts). Default output is YAML.

**Usage:** `acrylmicrovm info [OPTIONS]`

| Option | Description |
|--------|-------------|
| `--format FMT` | Output format: `yaml`, `json` (default: `yaml`). Table format is not supported. |

**Output fields:** `version`, `homeDir`, `virtualization`, `os`, `arch`, `boxesTotal`, `boxesRunning`, `boxesStopped`, `boxesConfigured`, `imagesCount`.

**Examples:**

```bash
acrylmicrovm info
acrylmicrovm info --format json
```

## Shell completion

Generate completion scripts for your shell:

```bash
# Bash
acrylmicrovm completion bash > /etc/bash_completion.d/acrylmicrovm
# or for current user
acrylmicrovm completion bash > ~/.local/share/bash-completion/completions/acrylmicrovm

# Zsh
acrylmicrovm completion zsh > "${fpath[1]}/_acrylmicrovm"

# Fish
acrylmicrovm completion fish > ~/.config/fish/completions/acrylmicrovm.fish
```

Then reload your shell or source the file.

## Environment variables

| Variable | Description |
|----------|-------------|
| `ACRYLMICROVM_HOME` | Runtime, credential, and context preference directory (default: `~/.acryl/microvm`). Overridden by `--home`. |
| `ACRYLMICROVM_REST_URL` | REST server endpoint; equivalent to `--url`. |
| `ACRYLMICROVM_API_KEY` | Long-lived API key sent as `Authorization: Bearer`. Overrides any stored credentials. |
| `ACRYLMICROVM_CONTEXT` | Override the saved current context; explicit `--context` takes precedence. |
| `ACRYLMICROVM_PROFILE` | Legacy fallback when `ACRYLMICROVM_CONTEXT` is unset or empty. |
| `ACRYLMICROVM_REST_PATH_PREFIX` | REST routing path segment; equivalent to `--path-prefix`. |
| `ACRYLMICROVM_SECURITY` | `run`/`create` sandbox preset; equivalent to `--security`. |
| `ACRYLMICROVM_SERVE_API_KEY` | Expected bearer key for `serve`; equivalent to `serve --api-key`. |
| `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_AUDIENCE` | OIDC login fallbacks after flags and server discovery. |
| `ACRYLMICROVM_LOG_FILE` | Explicit JSON log-file path for any command; `serve` otherwise defaults to `<ACRYLMICROVM_HOME>/logs/serve.log`. |
| `ACRYLMICROVM_EXPERIMENTAL` | Comma-separated RC features: `custom-kernel`, `nested-virtualization`, `native-vmm`. |
| `ACRYLMICROVM_RECONNECT_GRACE` | `serve` reconnect grace (default `5m`). |
| `ACRYLMICROVM_SHUTDOWN_GRACE` | `serve` shutdown-stage grace (default `30s`). |
| `ACRYLMICROVM_MAX_SESSION_LIFETIME` | `serve` session hard cap (default `24h`). |
| `SSH_CONNECTION` | `auth login` headless detection; choosing Browser from the TTY picker falls back to device flow when set. |
| `RUST_LOG` | Log level: `trace`, `debug`, `info`, `warn`, `error`. Use `RUST_LOG=debug` for troubleshooting. |

## Configuration file

Use `--config PATH` to load a JSON config file. Useful for default registries and other options. See [Image registry configuration](../../docs/guides/image-registry-configuration.md) for details.

## Troubleshooting

### Image pull fails
- Check network and registry access.
- For private registries, see [Image registry configuration](../../docs/guides/image-registry-configuration.md) for details.
- **"Failed to pull manifest"** or **"error sending request for url"** (e.g. to `index.docker.io`): often network-related or Docker Hub rate limit/access in some regions. Retry later, use a mirror, or configure registries via `--registry` / `--config`. See [issue #190](https://github.com/acryldev/acrylmicrovm/issues/190) for discussion.
- Enable debug output: `acrylmicrovm --debug pull IMAGE` or `RUST_LOG=debug acrylmicrovm pull IMAGE`.

### Box fails to start
- Enable debug output: `acrylmicrovm --debug run IMAGE [COMMAND]...` or `RUST_LOG=debug acrylmicrovm run IMAGE [COMMAND]...`.



## Further documentation

- [ACRYL microVMs README](../../README.md) — Project overview and SDK quick starts
- [Getting started](../../docs/getting-started/README.md) — Prerequisites and platform setup
- [Reference](../../docs/reference/README.md) — Python, Node, Rust, C API reference


## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.
