# ACRYL microVMs CLI reference

Exhaustive reference for the `acrylmicrovm` command-line interface — every subcommand, every flag, every exit code.

**Platforms:** macOS (Apple Silicon), Linux (x86_64, ARM64), Windows (WSL2)

For a quick start, see [`src/cli/README.md`](../../../src/cli/README.md).

## Table of contents

- [Synopsis](#synopsis)
- [Installation & Verification](#installation--verification)
- [Global Options](#global-options)
- [Environment Variables](#environment-variables)
- [Connecting to the cloud](#connecting-to-the-cloud)
- [Commands](#commands)
  - [`acrylmicrovm context`](#acrylmicrovm-context)
  - [`acrylmicrovm auth login`](#acrylmicrovm-auth-login)
  - [`acrylmicrovm auth logout`](#acrylmicrovm-auth-logout)
  - [`acrylmicrovm auth status`](#acrylmicrovm-auth-status)
  - [`acrylmicrovm auth whoami`](#acrylmicrovm-auth-whoami)
  - [`acrylmicrovm run`](#acrylmicrovm-run)
  - [`acrylmicrovm exec`](#acrylmicrovm-exec)
  - [`acrylmicrovm top`](#acrylmicrovm-top)
  - [`acrylmicrovm attach`](#acrylmicrovm-attach)
  - [`acrylmicrovm kill`](#acrylmicrovm-kill)
  - [`acrylmicrovm signal`](#acrylmicrovm-signal)
  - [`acrylmicrovm create`](#acrylmicrovm-create)
  - [`acrylmicrovm list`](#acrylmicrovm-list)
  - [`acrylmicrovm rm`](#acrylmicrovm-rm)
  - [`acrylmicrovm start`](#acrylmicrovm-start)
  - [`acrylmicrovm stop`](#acrylmicrovm-stop)
  - [`acrylmicrovm restart`](#acrylmicrovm-restart)
  - [`acrylmicrovm pull`](#acrylmicrovm-pull)
  - [`acrylmicrovm images`](#acrylmicrovm-images)
  - [`acrylmicrovm inspect`](#acrylmicrovm-inspect)
  - [`acrylmicrovm cp`](#acrylmicrovm-cp)
  - [`acrylmicrovm info`](#acrylmicrovm-info)
  - [`acrylmicrovm logs`](#acrylmicrovm-logs)
  - [`acrylmicrovm stats`](#acrylmicrovm-stats)
  - [`acrylmicrovm network tunnel`](#acrylmicrovm-network-tunnel)
  - [`acrylmicrovm network inbound / outbound`](#acrylmicrovm-network-inbound--outbound)
  - [`acrylmicrovm ssh`](#acrylmicrovm-ssh)
  - [`acrylmicrovm agent`](#acrylmicrovm-agent)
  - [`acrylmicrovm serve`](#acrylmicrovm-serve)
  - [`acrylmicrovm volume`](#acrylmicrovm-volume)
  - [`acrylmicrovm domain`](#acrylmicrovm-domain)
  - [`acrylmicrovm completion`](#acrylmicrovm-completion)
- [Shared Flag Groups](#shared-flag-groups)
- [Volume Mount Syntax](#volume-mount-syntax)
- [Port Publish Syntax](#port-publish-syntax)
- [Output Formats](#output-formats)
- [Configuration File](#configuration-file)
- [Exit Codes](#exit-codes)
- [See Also](#see-also)

---

## Synopsis

```
acrylmicrovm [GLOBAL OPTIONS] <COMMAND> [ARGS...]
```

`acrylmicrovm` is the command-line interface for the ACRYL microVMs runtime. It creates and manages "boxes" (lightweight VMs running OCI containers) on the local host or — with `--url` — against a remote ACRYL microVMs REST server.

---

## Installation & verification

The `acrylmicrovm` CLI can be installed these ways (all of them, with pnpm, yarn, bun and deno too, are in
[Installation](../../getting-started/installation.md)):

- From npm, prebuilt: `npx acrylmicrovm` or `npm install -g acrylmicrovm`
- One-line script: `curl -fsSL https://github.com/acryldev/acrylmicrovm/releases/latest/download/install.sh | sh`
- From crates.io: `cargo install acrylmicrovm-cli`
- Prebuilt binary via cargo: `cargo binstall acrylmicrovm-cli`

The one-line script installs to `$HOME/.local/bin/acrylmicrovm` and embeds the runtime, no extra setup. It is the
`install.sh` published on every GitHub Release, which is what `gh attestation verify` covers.

### Pin a version or override the install dir

```bash
curl -fsSL https://github.com/acryldev/acrylmicrovm/releases/latest/download/install.sh \
  | ACRYLMICROVM_VERSION=v0.13.1 ACRYLMICROVM_INSTALL_DIR=/usr/local/bin sh
```

The env-var prefix has to sit on the `sh` side of the pipe — variables placed before `curl` only decorate the curl process and never reach the installer.

### Pinning with an attested digest

When pinning a non-latest version, the installer falls back to the remote `.sha256` sidecar in that release for the expected digest. That anchor shares its trust root with the tarball, so for a guarantee independent of the release page, look up the digest in the release's attested `SHA256SUMS` and pass it in explicitly:

```bash
curl -fsSL https://github.com/acryldev/acrylmicrovm/releases/latest/download/install.sh \
  | ACRYLMICROVM_VERSION=v0.13.1 \
    ACRYLMICROVM_EXPECTED_SHA256=<sha256-of-acrylmicrovm-cli-vX.Y.Z-target.tar.gz> sh
```

### Verifying a downloaded tarball

Each release publishes raw tarballs (`acrylmicrovm-cli-vX.Y.Z-<target>.tar.gz`), matching `.sha256` sidecars, a combined `SHA256SUMS`, and sigstore-backed build provenance attestations. To verify a manually-downloaded artifact:

```bash
sha256sum -c "acrylmicrovm-cli-${VERSION}-${TARGET}.tar.gz.sha256"
gh attestation verify "acrylmicrovm-cli-${VERSION}-${TARGET}.tar.gz" \
  --repo acryldev/acrylmicrovm
```

### Verifying `install.sh` before running it

The `curl … | sh` shortcut can't self-verify, since the script runs as it is piped in. For users who want to verify the installer first, `install.sh` is also covered by `SHA256SUMS`, an `install.sh.sha256` sidecar, and the same sigstore attestation:

```bash
curl -fsSL -o install.sh \
  "https://github.com/acryldev/acrylmicrovm/releases/latest/download/install.sh"
curl -fsSL -o install.sh.sha256 \
  "https://github.com/acryldev/acrylmicrovm/releases/latest/download/install.sh.sha256"
sha256sum -c install.sh.sha256
gh attestation verify install.sh --repo acryldev/acrylmicrovm
sh ./install.sh
```

### Related operator notes

- [`scripts/release/install.sh.template`](../../../scripts/release/install.sh.template): the installer source, with inline comments documenting how each verification anchor is consumed

---

## Global options

Place these options before the top-level command or after the complete command
path. A parent command accepts only options shared by all children; for example,
use `acrylmicrovm auth status --url URL` or `acrylmicrovm --url URL auth status`, not
`acrylmicrovm auth --url URL status`. Help lists only options meaningful to that
backend, and executing invocations reject unsupported options. Clap's `--help`
and `--version` display actions remain terminal.

Both `-h` and `--help` show command-specific `Options` before `Global options`.
The global section contains only shared options accepted by that command.
For example, `acrylmicrovm list -h` keeps `--all`, `--quiet`, and `--format` together,
with `--context` and `--url` in the global section below.
A command-specific definition, such as `auth login --url`, stays in `Options`.
Commands without shared options omit the global section.

| Flag | Scope | Env Var | Description |
|------|-------|---------|-------------|
| `--debug` | all commands except `completion` | `RUST_LOG` (lower precedence) | Enable debug output. |
| `--home PATH` | local runtime + credentials + contexts | `ACRYLMICROVM_HOME` | Absolute runtime data, credential, and context preference directory. |
| `--registry REGISTRY` | `run`, `create`, `pull`, `serve` | — | Image registry hostname; repeatable and prepended to config. |
| `--config PATH` | local-capable commands | — | JSON runtime config (see [Configuration File](#configuration-file)). |
| `--url URL` | REST-capable commands | `ACRYLMICROVM_REST_URL` | Select a REST API server. |
| `--context NAME` | REST-capable commands + `auth` + `context` | `ACRYLMICROVM_CONTEXT` | Override the saved current context. Legacy aliases: `--profile`, `ACRYLMICROVM_PROFILE`. |
| `--path-prefix VALUE` | REST box/volume commands | `ACRYLMICROVM_REST_PATH_PREFIX` | Override the context's routing path segment; an explicit empty flag or environment value clears it. |

**Precedence** (from `src/cli/src/cli.rs`):

1. On dual-backend commands, a REST URL from the selected context or URL override
   selects REST; local runtime config is then unused. See [context precedence](#acrylmicrovm-context).
2. Otherwise, `--config` is the base, `--home` overrides `home_dir`, and
   `--registry` values are prepended to `image_registries`.
3. `serve`, `pull`, `images`, `info`, and `logs` always use the embedded local runtime.

---

## Environment variables

| Variable | Read by | Description |
|----------|---------|-------------|
| `ACRYLMICROVM_HOME` | `--home` | Runtime data, credentials, and context preferences; equivalent to `--home` |
| `ACRYLMICROVM_REST_URL` | `--url` | REST server endpoint; equivalent to `--url` |
| `ACRYLMICROVM_API_KEY` | REST runtime | Long-lived API key sent as `Authorization: Bearer`. Overrides any stored credentials. |
| `ACRYLMICROVM_CONTEXT` | context selection | Override the saved current context |
| `ACRYLMICROVM_PROFILE` | context selection | Legacy fallback when `ACRYLMICROVM_CONTEXT` is unset or empty |
| `ACRYLMICROVM_REST_PATH_PREFIX` | `--path-prefix` | REST routing path segment; overrides the selected context's value |
| `ACRYLMICROVM_SECURITY` | `run`, `create` | Sandbox preset; equivalent to `--security` |
| `ACRYLMICROVM_SERVE_API_KEY` | `serve` | Expected bearer key; equivalent to `serve --api-key` |
| `OIDC_ISSUER` | `auth login` | OIDC issuer fallback after an explicit flag and server discovery |
| `OIDC_CLIENT_ID` | `auth login` | OIDC client-id fallback after an explicit flag and server discovery |
| `OIDC_AUDIENCE` | `auth login` | OIDC audience fallback after an explicit flag and server discovery |
| `ACRYLMICROVM_LOG_FILE` | logging | Explicit JSON log-file path for any command; a bare filename is relative to the current directory. `serve` otherwise logs to `<ACRYLMICROVM_HOME>/logs/serve.log`. |
| `ACRYLMICROVM_EXPERIMENTAL` | `run`, `create` | Comma-separated RC features: `custom-kernel`, `nested-virtualization`, `native-vmm` |
| `ACRYLMICROVM_RECONNECT_GRACE` | `serve` | Reconnect grace before SIGHUP; default `5m` |
| `ACRYLMICROVM_SHUTDOWN_GRACE` | `serve` | Grace between shutdown escalation stages; default `30s` |
| `ACRYLMICROVM_MAX_SESSION_LIFETIME` | `serve` | Session hard cap; default `24h` |
| `SSH_CONNECTION` | `auth login` | Headless detection; choosing Browser from the TTY picker falls back to device flow when set |
| `RUST_LOG` | tracing | Log level/filter (`error`, `warn`, `info`, `debug`, `trace`; or per-module e.g. `acrylmicrovm=debug`) |

The three `serve` duration variables accept bare seconds or `Ns`, `Nm`, and
`Nh`. Invalid values log a warning and use the documented default.

---

## Connecting to the cloud

Create a remote context with `auth login`, then select it with `context use`.
The selected context supplies the endpoint, tenant path, and stored credentials.
`ACRYLMICROVM_API_KEY` overrides only the bearer credential.

```bash
# Interactive; select once for subsequent commands
acrylmicrovm --context cloud auth login --url https://<your-server>
acrylmicrovm context use cloud
acrylmicrovm list

# CI / scripted (API key from stdin)
echo "$KEY" | acrylmicrovm auth login --api-key-stdin --url https://<your-server>

# CI via env vars only
ACRYLMICROVM_API_KEY=$KEY ACRYLMICROVM_REST_URL=https://<your-server> acrylmicrovm list
```

Credentials are stored at `<ACRYLMICROVM_HOME>/credentials.toml` (default
`~/.acryl/microvm/credentials.toml`, perms `0600`). See [`acrylmicrovm auth login`](#acrylmicrovm-auth-login),
[`acrylmicrovm auth logout`](#acrylmicrovm-auth-logout), [`acrylmicrovm auth status`](#acrylmicrovm-auth-status),
and [`acrylmicrovm auth whoami`](#acrylmicrovm-auth-whoami) for the full command surface.

---

## Commands

### `acrylmicrovm context`

| Command | Behavior |
|---------|----------|
| `acrylmicrovm context use NAME` | Save the current context for subsequent commands. Validates that a named context exists; `default` and `embedded` always exist. |
| `acrylmicrovm context show` | Print the effective name, including flag and environment overrides. Does not validate credentials or contact the server. |
| `acrylmicrovm context ls` | List names and endpoints; mark the effective context with `*`. Alias: `list`. `-q` / `--quiet` prints names only. |

Create or update contexts with `acrylmicrovm --context NAME auth login --url URL`.
Login stores credentials without changing the saved selection. All context
commands work offline and omit tokens from their output.

Selection follows Docker's context/host behavior (`--url` is ACRYL microVMs' host
override). For commands other than `auth login`, resolve in this order:

1. Explicit `--context NAME` (legacy alias `--profile NAME`).
2. Explicit `--url URL`, selecting `default` with that URL.
3. Nonempty `ACRYLMICROVM_REST_URL`, selecting `default` with that URL.
4. Nonempty `ACRYLMICROVM_CONTEXT`.
5. Nonempty `ACRYLMICROVM_PROFILE` (legacy fallback).
6. Saved `current_context` in `<ACRYLMICROVM_HOME>/cli.toml`.
7. `default`.

Explicit context and URL flags conflict, except for `auth login`, where the URL
configures the selected context. An explicit named context ignores the URL
environment variable; explicit `--context default` still honors it. Empty
environment values are unset; empty selector flags are errors. Overrides never
change the saved selection. `context use` warns when an override remains active.

`default` represents the existing ACRYL microVMs default configuration: use the URL
override, otherwise credentials named `default`, otherwise the embedded runtime.
Existing remote default logins therefore keep working. `local` is an ordinary
user-defined name. A missing named context fails before a workload starts, without
falling back to the embedded runtime. Logout removes credentials but keeps the
selection; log in again or explicitly `context use default` to recover.

`embedded` is a built-in context that always selects the embedded runtime. It is
always listed, and `context use embedded` works in any home, so the embedded
runtime stays reachable after a login stored under `default`:

```bash
acrylmicrovm auth login --url https://<your-server>   # stored under `default`
acrylmicrovm context ls
# NAME                     ENDPOINT
# default *                https://<your-server>
# embedded                 embedded
acrylmicrovm context use embedded
acrylmicrovm list                       # embedded runtime
```

`auth login` refuses the name `embedded`, and `ACRYLMICROVM_API_KEY` has no effect on
it. It follows the precedence above like any named context: explicit
`--context embedded` ignores `ACRYLMICROVM_REST_URL`, while a saved selection ranks
below a nonempty one. A login stored under `embedded` by an earlier CLI keeps
targeting its server and prints a warning on each workload command. Log in under
another name, then run `acrylmicrovm --context embedded auth logout`; from then on
`embedded` selects the embedded runtime.

Preferences and credentials share the `--home` / `ACRYLMICROVM_HOME` directory.
`cli.toml` stores only the current name; `credentials.toml` retains the compatible
`[profiles.NAME]` schema. Switching atomically replaces a private preference file
and does not rewrite credentials. Concurrent switches use the last completed
write. Malformed or unreadable preferences are errors when selection needs them.
Workload commands snapshot the context and credentials before dispatch, so a
concurrent switch cannot redirect a running command. Local-only commands (`serve`,
`pull`, `images`, `info`, `logs`), help, and completion do not read context state.

```bash
acrylmicrovm context use cloud
acrylmicrovm list                       # cloud
acrylmicrovm --context staging list     # one invocation; cloud stays selected
acrylmicrovm context show              # cloud
acrylmicrovm context use default       # restore the default configuration
```

The precedence is verified against Docker CLI 29.4.3 and its
[context resolver source](https://github.com/docker/cli/blob/v28.5.2/cli/command/cli.go#L461-L485).
Docker's [context use documentation](https://docs.docker.com/reference/cli/docker/context/use/)
describes persisted switching. Its [environment-variable documentation](https://docs.docker.com/reference/cli/docker/#environment-variables)
differs from that executable ordering. ACRYL microVMs follows the executable behavior above.
See [design #2167](https://github.com/acryldev/acrylmicrovm/issues/2167).

---

### `acrylmicrovm auth login`

**Synopsis:** `acrylmicrovm auth login [OPTIONS]`

Log in with a long-lived API key, browser OIDC, or device-code OIDC.
Credentials are stored at `<ACRYLMICROVM_HOME>/credentials.toml` (default
`~/.acryl/microvm/credentials.toml`, perms `0600`).
`--context NAME` selects the context to create or update; otherwise login uses
the effective current context. `--url` configures that context and does not
change the selected name. Login never changes the saved current context.

**Options:**

| Flag | Description |
|------|-------------|
| `--url URL` | Server URL. Precedence: flag, `ACRYLMICROVM_REST_URL`, selected context URL, then `https://api.webboxes.ai`. |
| `--method <api-key\|browser\|device>` | Select the login flow explicitly. |
| `--api-key-stdin` | Read the API key from stdin (one line); selects `api-key` and conflicts with explicit browser/device methods. |
| `--no-browser` | Use device-code login when no method is explicit; conflicts with API-key stdin and explicit `api-key`/`browser` methods. |
| `--issuer URL` | Browser/device only: override the OIDC issuer discovered from the server. |
| `--client-id ID` | Browser/device only: override the discovered OIDC client ID. |
| `--audience VALUE` | Browser/device only: override the discovered OIDC audience. |
| `--callback-port PORT` | Browser only: callback port, `1..=65535` (default `5555`). |

**Examples:**

```bash
# Interactive — prompts for the API key with hidden input
acrylmicrovm auth login

# API key from stdin (CI-friendly)
echo "$KEY" | acrylmicrovm auth login --api-key-stdin --url https://<your-server>
```

---

### `acrylmicrovm auth logout`

**Synopsis:** `acrylmicrovm auth logout [OPTIONS]`

Delete the selected context from `<ACRYLMICROVM_HOME>/credentials.toml`. The file is
removed when its final context is deleted. Prompts unless `--yes` is given.
The current selection stays unchanged. Workload commands fail if the selected
named context is now missing; log in again or switch explicitly.

**Options:**

| Flag | Short | Description |
|------|-------|-------------|
| `--yes` | `-y` | Skip the confirmation prompt |

---

### `acrylmicrovm auth status`

**Synopsis:** `acrylmicrovm auth status [--url URL | --context NAME]`

Print the current authentication state without revealing the secret. Reports
the context name and selection source, logged-in URL, and credential source
(stored file vs env var). If neither the
file nor env vars are present, prints
``Not logged in (context `NAME`).`` An explicit `--url`
selects the default context with that endpoint. With only `ACRYLMICROVM_API_KEY` set, the
auth probe reports the default API endpoint, `https://api.webboxes.ai`.

**Example output:**

```
Context:         default (from default)
Logged in to:    https://api.webboxes.ai
Credential:      API key (from <ACRYLMICROVM_HOME>/credentials.toml [default])
```

When the env var override is active:

```
Credential:      API key (from ACRYLMICROVM_API_KEY env var)
```

---

### `acrylmicrovm auth whoami`

**Synopsis:** `acrylmicrovm auth whoami [--url URL | --context NAME]`

Call `GET /v1/me` with the active credential and print the server-resolved
principal, path prefix, scopes, and server URL. An explicit `--url` selects the
default context with that endpoint. OIDC credentials refresh when near expiry.

**Example output:**

```
Logged in as:    dev@acme.test
Name:            Dev McAcme
Principal:       auth0|abc123 (user)
Path prefix:     acme
Server:          https://api.webboxes.ai
Scopes:          box:read, box:write, box:exec
```

---

### `acrylmicrovm run`

**Synopsis:**

- `acrylmicrovm run [OPTIONS] IMAGE [COMMAND...]`
- `acrylmicrovm run [OPTIONS] --rootfs PATH [COMMAND...]`

Create a box from an image (or a prepared rootfs via `--rootfs`) and run a
command, with docker's semantics: `COMMAND` replaces the image's `CMD`, the
image's `ENTRYPOINT` is prepended, and the result **is** the container's init
(PID 1).

`--entrypoint EXEC` replaces the image entrypoint and clears its default
CMD; only an explicit `COMMAND` is appended. Omit both to run the image's defaults.
See [startup command overrides](../../concepts/images.md#startup-command-overrides).

The box's lifetime is that command's lifetime. When it exits, the box stops and
takes the command's exit code; `acrylmicrovm ps` shows it stopped and
`acrylmicrovm inspect -f '{{.State.ExitCode}}'` gives the code.

**Options:** Uses [`ProcessFlags`](#processflags) + [`CapabilityFlags`](#capabilityflags) + [`ResourceFlags`](#resourceflags) + [`PublishFlags`](#publishflags) + [`VolumeFlags`](#volumeflags) + [`NetworkFlags`](#networkflags) + [`ManagementFlags`](#managementflags), plus:

| Flag | Short | Description |
|------|-------|-------------|
| `--rootfs PATH` | — | Use a prepared rootfs path instead of pulling/resolving an image |
| `--entrypoint EXEC` | — | Override the image entrypoint |
| `--detach` | `-d` | Run in the background and print the box ID |
| `--rm` | — | Remove the box when it stops; conflicts with lifecycle deadlines and is ignored with `--detach` |

**Exit behavior:**

- Default (foreground): streams stdout/stderr to the terminal, exits with the box command's exit code. If the command was killed by signal *N*, exits with `128 + N` (Unix convention, see [Exit Codes](#exit-codes)).
- If the box's init command is missing or non-executable, startup fails: the CLI prints `Error: ...` to stderr and exits `1`. The `exec` startup-refusal statuses below do not apply to `run`.
- `-d`/`--detach`: prints the box ID to stdout and exits `0` immediately; remove-on-stop is force-disabled in this mode so the box outlives the CLI process.
- `--tty` with non-TTY stdin: fails with `the input device is not a TTY.`

**Examples:**

```bash
acrylmicrovm run alpine:latest echo "Hello"
acrylmicrovm run -it --rm alpine:latest /bin/sh
acrylmicrovm run -d --name web -p 8080:80 nginx:alpine
acrylmicrovm run -v $(pwd):/work -w /work alpine:latest ls -la
acrylmicrovm run --cpus 4 --memory 4096 python:slim python -c "print(2+2)"
acrylmicrovm run --cap-add SYS_ADMIN --cap-drop NET_RAW alpine:latest sh
acrylmicrovm run --rootfs /path/to/rootfs /bin/sh
```

---

### `acrylmicrovm exec`

**Synopsis:** `acrylmicrovm exec [OPTIONS] BOX -- COMMAND [ARGS...]`

Run a command in a box. A safely resumable stopped box starts implicitly; a
job box whose start would run or re-run its user-selected main command is
refused, so start it deliberately with `acrylmicrovm start` first. The `--`
separator is required (`src/cli/src/commands/exec.rs:22`, `last = true`).

**Options:** Uses [`ProcessFlags`](#processflags), plus:

| Flag | Short | Description |
|------|-------|-------------|
| `--detach` | `-d` | Start the command and return immediately without streaming |

**Exit behavior:** Same as `acrylmicrovm run` (foreground streams + propagates exit code; detach exits `0`). After a foreground exec finishes the CLI calls `runtime.shutdown(None)` to release the box handle gracefully.

For `exec`, a command the guest refuses to launch is still a completed execution:
`127` when the executable is not found, `126` when it exists but is not
executable. Foreground exec propagates that status. The guest's diagnostic goes
to stderr without a TTY, or to merged stdout with `--tty`. `acrylmicrovm serve`
returns the same execution statuses and streams over REST. Rejection of the
box's init command during `run` remains a CLI error with exit code `1`.

**Examples:**

```bash
acrylmicrovm exec mybox -- echo "hello"
acrylmicrovm exec -it mybox -- /bin/sh
acrylmicrovm exec -e DEBUG=1 -w /app mybox -- pytest tests/
```

---

### `acrylmicrovm top`

**Synopsis:** `acrylmicrovm top [OPTIONS] BOX`

List live guest executions, including the box's init process. The list is read
from the guest and does not start a configured or stopped box. A paused box is
rejected with an invalid-state error. Execution IDs are canonical guest IDs and
can be used by `attach`, `kill`, and `signal`, including from a different CLI
process.

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--quiet` | `-q` | `false` | Print only execution IDs; conflicts with `--format` |
| `--format FMT` | — | `table` | Output format: `table`, `json`, or `yaml` |

**Examples:**

```bash
acrylmicrovm top mybox
acrylmicrovm top mybox --format json
acrylmicrovm top mybox -q
```

---

### `acrylmicrovm attach`

**Synopsis:** `acrylmicrovm attach [--no-stdin] BOX EXECUTION_ID`

Attach to an existing guest execution, replaying available output. This does
not start the box. `--no-stdin` keeps the attach read-only.

---

### `acrylmicrovm kill`

**Synopsis:** `acrylmicrovm kill BOX EXECUTION_ID`

Send SIGKILL to a listed guest execution. This is independent of any existing
output reader or CLI process.

---

### `acrylmicrovm signal`

**Synopsis:** `acrylmicrovm signal BOX EXECUTION_ID SIGNAL`

Send a supported cooperative Unix signal to a listed guest execution. Use
`acrylmicrovm kill` for SIGKILL.

---

### `acrylmicrovm create`

**Synopsis:**

- `acrylmicrovm create [OPTIONS] IMAGE [COMMAND...]`
- `acrylmicrovm create [OPTIONS] --rootfs PATH [COMMAND...]`

Create a box without starting it. Prints the new box's ID to stdout.

`COMMAND` is stored, not run — it becomes the box's main command (the container's
init) when the box is next started, exactly as it would under `run`.
`--entrypoint EXEC` uses the same override rules as [`run`](#acrylmicrovm-run), including
clearing the image CMD. Omit both `COMMAND` and `--entrypoint` to use image defaults.

A box created **with** `COMMAND` or `--entrypoint` is a job: `exec` and `cp` will not
start it implicitly, because starting it runs that command. Start it deliberately with
`acrylmicrovm start`. A box created **without** either boots the image's default, and
`exec` still starts it on demand.

**Options:**

| Flag | Short | Description |
|------|-------|-------------|
| `--rootfs PATH` | — | Use a prepared rootfs path instead of pulling/resolving an image |
| `--env KEY[=VALUE]` | `-e` | Set environment variables; a bare key inherits its host value (repeatable) |
| `--workdir PATH` | `-w` | Working directory inside the box |
| `--entrypoint EXEC` | — | Replace the image entrypoint with one executable |

Also uses [`CapabilityFlags`](#capabilityflags) + [`ResourceFlags`](#resourceflags) + [`PublishFlags`](#publishflags) + [`VolumeFlags`](#volumeflags) + [`NetworkFlags`](#networkflags) + [`ManagementFlags`](#managementflags).

> Note: `create` accepts `--env` and `--workdir` directly rather than via `ProcessFlags` (no `-i`/`-t`/`-u` here, since no command is being executed).

**Examples:**

```bash
acrylmicrovm create --name mybox alpine:latest
acrylmicrovm create -p 8080:80 -v /data:/app/data --name web nginx:alpine
acrylmicrovm create --cap-drop ALL --cap-add NET_BIND_SERVICE --name web nginx:alpine
acrylmicrovm create --rootfs /path/to/rootfs --name local-rootfs
```

---

### `acrylmicrovm list`

**Aliases:** `ls`, `ps`

**Synopsis:** `acrylmicrovm list [OPTIONS]`

List boxes.

**Options:**

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--all` | `-a` | `false` | Show all boxes (default: only active) |
| `--quiet` | `-q` | `false` | Print only IDs; conflicts with `--format` |
| `--format FMT` | — | `table` | Output format; conflicts with `--quiet` (see [Output Formats](#output-formats)) |

**Examples:**

```bash
acrylmicrovm list                  # active boxes, table
acrylmicrovm ls -aq                # all box IDs, one per line
acrylmicrovm ps --format json
```

---

### `acrylmicrovm rm`

**Synopsis:** `acrylmicrovm rm [OPTIONS] [BOX...]`

Remove one or more boxes. Either name them or use `--all`.

**Options:**

| Flag | Short | Description |
|------|-------|-------------|
| `--force` | `-f` | Force-remove a running box |
| `--all` | `-a` | Remove all boxes (prompts unless `--force`) |

**Exit behavior:** Prints each removed box ID to stdout. If any target fails, prints its error to stderr and exits non-zero after attempting the rest.

**Examples:**

```bash
acrylmicrovm rm mybox
acrylmicrovm rm -f mybox1 mybox2
acrylmicrovm rm --all --force
```

---

### `acrylmicrovm start`

**Synopsis:** `acrylmicrovm start BOX [BOX...]`

Start one or more stopped boxes. No options. Prints each started box's name/ID to stdout; aggregates errors and exits non-zero if any failed.

---

### `acrylmicrovm stop`

**Synopsis:** `acrylmicrovm stop BOX [BOX...]`

Stop one or more running boxes. Same shape as `start`.

---

### `acrylmicrovm restart`

**Synopsis:** `acrylmicrovm restart BOX [BOX...]`

Stop then start one or more boxes. If `stop` fails for a box, that box is skipped (resources may still be locked) and the error is reported. After `stop`, the CLI re-fetches the box handle with `runtime.get()` because the post-stop handle is invalidated.

---

### `acrylmicrovm pull`

**Synopsis:** `acrylmicrovm pull [OPTIONS] IMAGE`

Pull an image from a registry into the local image cache.

**Options:**

| Flag | Short | Description |
|------|-------|-------------|
| `--quiet` | `-q` | Print only the image's config digest |

**Examples:**

```bash
acrylmicrovm pull alpine:latest
acrylmicrovm pull -q ghcr.io/openclaw/openclaw:main
```

---

### `acrylmicrovm images`

**Synopsis:** `acrylmicrovm images [OPTIONS]`

List cached images.

**Options:**

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--quiet` | `-q` | `false` | Print only image IDs; conflicts with `--format` |
| `--format FMT` | — | `table` | Output format; conflicts with `--quiet` (see [Output Formats](#output-formats)) |

---

### `acrylmicrovm inspect`

**Synopsis:** `acrylmicrovm inspect [OPTIONS] [BOX...]`

Show detailed information for one or more boxes.

**Options:**

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--latest` | `-l` | `false` | Inspect the most recently created box (cannot combine with `BOX`) |
| `--format FMT` | `-f` | `json` | `json`, `yaml`, or a Go template (e.g. `'{{.State.Status}}'`) |

The Go-template engine exposes a `json` function for serializing nested values.
`State.StartedAt` is when the box most recently entered `Running`, in RFC 3339
format, or `null` if the start time has not been recorded or is unavailable
over REST. `State.LastActivityAt` is when the box was last active in RFC 3339
format — the clock AutoStop measures idleness against — or `null` for a local
box, which records no activity. `Network.Outbound` and `Network.Inbound` each
carry `Mode` (`enabled` or `disabled`) and `AllowNet`; `Network` is `null` when
the runtime does not know the policy, such as a REST server that predates it.

**Examples:**

```bash
acrylmicrovm inspect mybox
acrylmicrovm inspect --format '{{.State.Status}}' mybox
acrylmicrovm inspect --format '{{.Network.Inbound.Mode}}' mybox
acrylmicrovm inspect --format '{{.State.StartedAt}}' mybox
acrylmicrovm inspect --format '{{.State.LastActivityAt}}' mybox
acrylmicrovm inspect -l --format yaml
```

---

### `acrylmicrovm cp`

**Synopsis:** `acrylmicrovm cp [OPTIONS] SRC DST`

Copy files/folders between host and box. Exactly one of `SRC` or `DST` must be a `BOX:PATH` reference.
A directory `SRC` lands as `docker cp` lands it: it becomes a `DST` that does not exist, and
lands under its own name in one that does. End `SRC` with `/.` to copy its contents either way.
A file `SRC`, or a symlink copied without `--follow-symlinks`, becomes `DST`, or lands inside it
when `DST` is a directory or ends in `/`. A directory cannot replace a file: copying one onto an
existing file `DST` fails.

**Options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--follow-symlinks` | `false` | Resolve symlink targets (embedded local runtime only) |
| `--no-overwrite` | `false` | Refuse to overwrite destination files (embedded local runtime only) |

If a stopped box is safe to resume, it is started temporarily and restored to
stopped state after the copy succeeds or fails. A job box whose start would run
or re-run its user-selected main command is refused; start it deliberately
before copying.

Paths at or under a mount inside the box are refused in both directions — `/tmp`,
`/dev/shm`, volumes, and the `/etc/hosts`, `/etc/hostname`, `/etc/resolv.conf` binds.
`cp` works on the rootfs layer from outside the box's mount namespace, so such a copy
would write where nothing in the box can see it, or read back the image's file rather
than the one the box has.

A directory that merely *contains* a mount is treated differently per direction. Copying
one **out** is refused (`acrylmicrovm cp box:/etc ./etc`), since the archive would carry the
image's files rather than the mounted ones. Copying **in** is allowed, and refused only
if a file being written would land on a mount — so `acrylmicrovm cp ./x box:/etc` works. The
copy streams into the box, so that file is refused before it is written, and the files
written before it stay.

Copy a path outside the mount, or pipe a tar through `acrylmicrovm exec`. Files copied in are
owned by the box's exec user.

**Examples:**

```bash
acrylmicrovm cp ./script.py mybox:/work/script.py        # host -> box
acrylmicrovm cp mybox:/var/log/app.log ./app.log         # box -> host
acrylmicrovm cp --no-overwrite ./data/ mybox:/data/      # local runtime only
acrylmicrovm cp ./dist/. mybox:/srv/www                  # the contents of ./dist
```

---

### `acrylmicrovm info`

**Synopsis:** `acrylmicrovm info [OPTIONS]`

System-wide runtime information: version, home dir, virtualization status, OS/arch, and box/image counts.

**Options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--format {yaml\|json}` | `yaml` | Output format |

Output fields: `version`, `homeDir`, `virtualization` (`available` or `unavailable: <reason>`), `os`, `arch`, `boxesTotal`, `boxesRunning`, `boxesStopped`, `boxesConfigured`, `imagesCount`.

---

### `acrylmicrovm logs`

**Synopsis:** `acrylmicrovm logs [OPTIONS] BOX`

Show the box's console log (`{home}/boxes/{box_id}/logs/console.log`).

**Options:**

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--tail N` | `-n` | `0` | Show only the last N lines (0 = all) |
| `--follow` | `-f` | `false` | Stream new output as it's written |

If the log file does not exist (box never started), prints a hint to stderr and exits `0`.

---

### `acrylmicrovm stats`

**Synopsis:** `acrylmicrovm stats [OPTIONS] BOX`

Display resource usage statistics for a box.

**Options:**

| Flag | Short | Default | Description |
|------|-------|---------|-------------|
| `--format FMT` | — | `table` | Output format (see [Output Formats](#output-formats)) |
| `--stream` | `-s` | `false` | Refresh every second until Ctrl-C |

---

### `acrylmicrovm network inbound / outbound`

```text
acrylmicrovm network inbound BOX MODE
acrylmicrovm network outbound BOX MODE [--allow-net RULE]...
```

`BOX` is an ID or name; `MODE` is required and accepts `enabled` or `disabled`.
Each invocation replaces the selected direction. `inbound` accepts no allowlist.

```bash
acrylmicrovm network inbound mybox enabled
acrylmicrovm network outbound mybox enabled --allow-net '*.example.com' --allow-net 10.0.0.0/8
acrylmicrovm network outbound mybox enabled  # default empty list: unrestricted
acrylmicrovm network outbound mybox disabled
acrylmicrovm inspect mybox
```

`--allow-net` takes one hostname, wildcard hostname, IPv4 address or CIDR.
Repeat it for multiple rules; comma-separated values and empty rules such as
`''` are invalid, as they are at create. Omit `--allow-net` for the default
empty list, which removes restrictions when enabled. Disabled mode accepts no
`--allow-net`. No previous allowlist is retained or restored.

Outbound updates require a running box with an existing NIC. Local inbound
updates are unsupported; cloud inbound uses the proxy's existing visibility
contract. See [runtime behavior and persistence](../../guides/networking.md#updating-a-running-box).

### `acrylmicrovm network tunnel`

**Synopsis:** `acrylmicrovm network tunnel BOX GUEST_PORT [--private] [--listen ADDRESS | --revoke]`

Without `--listen`, print the public URL supplied by a remote REST service;
local boxes have no public URL. With `--listen`, bind a local listener and
forward it to either a local or remote box instead:

```bash
acrylmicrovm network tunnel mybox 3000 --listen 8080
acrylmicrovm network tunnel mybox 3000 --listen 127.0.0.1:8080
acrylmicrovm network tunnel mybox 3000 --listen '[::1]:8080'
acrylmicrovm network tunnel mybox 3000 --listen unix:/tmp/app.sock
```

A bare port binds `127.0.0.1`; listener port `0` asks the OS to allocate a
port. A client that connects while nothing in the box listens on `GUEST_PORT`
is disconnected; the listener keeps running. The canonical bound address is
printed to stdout before clients are accepted.
Ctrl-C closes the listener and active connections. TCP hosts must be numeric
addresses, and Unix socket paths must be absolute.

A remote box needs inbound access enabled before it can be tunneled: create it
with `--inbound enabled`.

For a private cloud tunnel, use the context's API key or JWT:

```bash
acrylmicrovm --context cloud network tunnel mybox 3000 --private
acrylmicrovm --context cloud network tunnel mybox 3000 --private --listen 8080
```

Private creation prints one JSON line containing `uri`, `isPublic`, and `token`.
Only creation returns the complete token; repeating the same command returns its
mask. An existing public tunnel must be revoked before recreating it as private.
With `--listen`, the descriptor is printed and flushed **before** binding, followed
by the bound address on the next line. A bind failure leaves the created tunnel
and its already-printed token available; Ctrl-C closes only the local forwarder.
Each accepted client opens an authenticated CONNECT without preparing again.
There is no `--token` option: external visitors use the URL query or HTTP header,
while CLI traffic uses the configured organization credential.

`--revoke` withdraws the port's public or private tunnel instead, without `--private`. New requests to it get
`404` within 3 seconds; connections already open are not closed. It is safe to
repeat, and running the command again without `--revoke` reopens the same URL:

```bash
acrylmicrovm network tunnel mybox 3000 --revoke
```

---

### `acrylmicrovm ssh`

**Synopsis:**

```
acrylmicrovm ssh --configure BOX CONFIG
acrylmicrovm ssh BOX
acrylmicrovm ssh BOX --install-ssh-config [--alias NAME] [--overwrite]
acrylmicrovm ssh --status BOX
acrylmicrovm ssh --enable BOX
acrylmicrovm ssh --disable BOX
acrylmicrovm ssh --tunnel BOX [--listen ADDRESS]
acrylmicrovm ssh --proxy BOX

CONFIG := --ssh-config FILE
        | --host-key FILE --login NAME --authorized-key FILE... [--guest-listen ADDRESS]
```

Control the guest SSH server of a local box or a REST box (`--url`/`--context`).
With no operation option, `acrylmicrovm ssh BOX` prepares CLI-managed credentials and
logs in immediately using system OpenSSH.
Choose at most one of `--install-ssh-config`, `--configure`, `--status`, `--enable`,
`--disable`, `--tunnel`, or `--proxy`; options may precede or follow BOX.
These replace the former SSH subcommands: for example, `ssh enable dev` becomes
`ssh dev --enable`, and `ssh setup dev` becomes `ssh dev`.
Default login saves client and host keys locally; complete `--configure`
continues to read supplied key files.

These commands add no HTTP routes. Backend requirements differ:

| Commands | Local backend | REST backend |
|----------|---------------|--------------|
| `--configure`, `--status`, `--enable`, `--disable` | Existing SSH SDK operations | Requires SSH control; supported by `acrylmicrovm serve` |
| default login | CLI-managed credentials and an OpenSSH session through a loopback forwarder | Requires SSH control and tunnel support; unavailable through `acrylmicrovm serve` |
| `--install-ssh-config` | Prepare managed SSH and install a native OpenSSH alias | Requires SSH control; does not open a tunnel during installation |
| `--proxy`, `--tunnel` | Stdio connection or local listener through the existing tunnel | Requires both SSH control and tunnel support; unavailable through `acrylmicrovm serve` |

| Operation | Effect | stdout |
|------------|--------|--------|
| `--configure` | Replace the complete configuration once; disconnects existing SSH clients | `SshStatus` JSON |
| default login | Generate/reuse local keys; configure new/generation-zero identities, enable disabled saved SSH, otherwise read status and connect | Remote session output only |
| `--install-ssh-config` | Prepare SSH, update the managed Host entry and Include, then exit without logging in | Empty; connection guidance goes to stderr |
| `--status` | Read state; never starts the box | `SshStatus` JSON |
| `--enable` | Enable saved configuration on a running box; preserve healthy connections | `SshStatus` JSON |
| `--disable` | Stop SSH on a running box; never starts the box | `SshStatus` JSON |
| `--tunnel` | Read status; fail if disabled; forward to the port SSH actually bound | Bound address |
| `--proxy` | Read status, then connect stdin/stdout to enabled SSH; never configures or enables it | SSH bytes only |

Connect directly:

```bash
acrylmicrovm ssh dev
```

Install once to use plain OpenSSH:

```bash
acrylmicrovm ssh dev --install-ssh-config
ssh dev
acrylmicrovm --context cloud ssh dev --install-ssh-config --alias dev-cloud
```

The default alias is the resolved box name, or full ID for unnamed boxes.
Reinstalling the same target is idempotent; conflicts with another ACRYL microVMs-managed
target require `--alias NAME` or `--overwrite`. Overwrite affects only managed
entries, not handwritten Host blocks. The installer preserves existing user
configuration, adds one leading Include and stores entries at
`~/.acryl/microvm/ssh/config`. User/system Match conditions, Includes and personal
identities remain subject to normal OpenSSH rules; installation does not validate
the combined configuration or guarantee complete isolation. New original-config
backups live at `~/.acryl/microvm/ssh/config.backup`.
Generated proxies fix the backend/home and remote context/endpoint/routing without
storing API tokens. See [managed aliases](../../guides/ssh.md#managed-openssh-aliases)
for file safety, configuration precedence and lifecycle rules.

Custom configuration always replaces the whole SSH configuration and disconnects
clients. Default login does so only for a new managed identity or disabled
generation-zero status, preserving the SDK startup policy. Generation zero does
not imply absence of persisted configuration. Existing enabled services are
queried without modification; disabled services with a positive generation are
enabled once. No service mutation is retried within the command.

Configuration and the enable switch persist on the container disk; restarting
restores the saved enabled switch. Disable retains credentials. Explicit
enable/disable reject idle boxes; enable also fails without saved configuration.
Default login compares the reported host public key with its validated local host
key, ignoring comments, and refuses a mismatch without automatically reconfiguring.
Authentication failure never triggers configuration. `--tunnel` leaves configuration
and generation unchanged.

Default login requires executable `ssh` and `ssh-keygen` on PATH, accepts no key flags, and saves `client_key` and
`host_key` in `<home>/boxes/<box-id>/ssh/` for local boxes, or
`<home>/ssh/remote/<scope>/<box-id>/` for REST boxes (directory 0700, private files 0600).
The remote scope is a SHA-256 namespace of the context, effective URL and routing
prefix; changing any of these selects separate keys. Remote home is the credential
home (`--home` / `ACRYLMICROVM_HOME` / `~/.acryl/microvm`), independent of local `--config`.
The client private key stays on the host; deleting a local box removes it, while
remote keys remain for manual cleanup. Clone/export does not transfer the managed
client identity. Invalid or incomplete saved keys fail without regeneration.
Configuration failure retains generated keys for the next invocation. A private
`known_hosts` file is atomically derived from the saved host key; it never trusts a
new remote identity automatically. Canonical box IDs and scoped remote aliases
keep trust independent of box names and ephemeral loopback ports.

The command uses a single resolved context/authentication/endpoint/routing snapshot
throughout the session. Automatic OpenSSH uses `-F /dev/null`, strict host checking,
only the managed identity, no agent, and no multiplexing. It neither reads nor
modifies user SSH config. Use `--proxy` or `--tunnel` for a custom client invocation.
OpenSSH inherits stdio, selects PTY behavior and handles terminal resize; interactive
Ctrl-C reaches the guest. The CLI returns its exit code, including a remote shell's
nonzero status, after reaping the child and closing forwarding.

Local login, proxy and tunnel hold the runtime home lock until they end.
See the [SSH guide](../../guides/ssh.md#cli) for lifecycle and signal handling.

| Flag | Description |
|------|-------------|
| `--install-ssh-config` | Prepare managed SSH and install an alias; exclusive with the other operations. |
| `--alias NAME` | Installed OpenSSH alias instead of the box name/ID; requires `--install-ssh-config`. |
| `--overwrite` | Repoint a conflicting ACRYL microVMs-managed alias; requires `--install-ssh-config`. |
| `--ssh-config FILE` | Complete `SshConfig` JSON ([schema](../../guides/ssh.md#http-control-with-acrylmicrovm-serve)): several accounts, authorized keys, CAs. |
| `--host-key FILE` | Unencrypted OpenSSH host private key. Requires `--login` and `--authorized-key`. |
| `--login NAME` | The single login accepted for `--authorized-key`. |
| `--authorized-key FILE` | One OpenSSH public key per file; repeatable for `--configure`. |
| `--guest-listen ADDRESS` | Guest listener for flag mode; default `0.0.0.0:2222`; port `0` lets the guest choose. |
| `--listen ADDRESS` | `--tunnel` only: local listener, same syntax as [`network tunnel`](#acrylmicrovm-network-tunnel); default `127.0.0.1:0`. |

Configuration flags require `--configure`; `--listen` requires `--tunnel`.
They are rejected with incompatible operations before reading files or contacting a box.
`--ssh-config` conflicts with the
flag-mode options. Configuration errors name the
file and position but never print its contents. Custom and managed configuration follow the SDK's
startup rules: it may start a Configured or Stopped box that uses its image's default
command. Configuration is never retried.

`--tunnel` prints the canonical bound address to stdout, then
the host public key, its fingerprint, and a connect hint to stderr:

```bash
$ acrylmicrovm ssh --tunnel dev
tcp://127.0.0.1:52811
SSH host key: ssh-ed25519 AAAAC3Nz...
SSH host key fingerprint: SHA256:...
Connect: ssh -p 52811 -l LOGIN 127.0.0.1
```

`--tunnel` and `--proxy` require the guest to listen on `0.0.0.0` or the guest IP.
Ctrl-C closes the listener and its connections, then exits 0; guest SSH stays
configured. A forwarding failure leaves the preceding configuration unchanged.

- **Local boxes:** the foreground command holds the runtime home, so other local
  commands on that home fail until the session ends. Non-detached boxes stay alive
  during login and stop after CLI exit; use `acrylmicrovm create` or `run -d` to retain them.
- **REST boxes:** login/proxy/tunnel use the box's public tunnel URL for the SSH port and
  leave it open on exit, so anyone reaching the URL can attempt SSH logins. Withdraw it
  with `acrylmicrovm network tunnel BOX PORT --revoke`. `acrylmicrovm serve` has no tunnel
  route, so default login, `--proxy` and `--tunnel` fail there; explicit
  configure/status/enable/disable remain available. Cloud tunnels require inbound
  access; CLI exit does not stop the remote box.

---

### `acrylmicrovm agent`

Agent commands address a Box, an agent and, where needed, a turn. Each run
reconnects to the agent by ID, so separate processes share one agent. They work on a
local runtime with [OpenCode in the Box](../agents.md#opencode-on-a-local-runtime);
a `--url` profile and operations OpenCode lacks report `Unsupported`. Runtime errors
go to stderr with exit code 1. Malformed arguments use the normal CLI usage error
(code 2).

Use a running Box created with `acrylmicrovm create` and `acrylmicrovm start`, or `acrylmicrovm run -d`:
a Box that is not detached stops when the CLI process that started it exits, which cuts
off a running turn. Each local `acrylmicrovm` process holds the ACRYL microVMs home for its whole
run, so while `events` streams, other commands on that home fail until it ends.

| Purpose | Command after `acrylmicrovm agent` | Operation |
|---|---|---|
| Lifecycle | `create BOX --agent IMPLEMENTATION` | Create an instance with the selected implementation and initial configuration. |
| Lifecycle | `close BOX --agent AGENT` | End the agent. |
| Work | `send BOX --agent AGENT TEXT` | Submit a new turn. |
| Work | `wait BOX --agent AGENT --turn TURN` | Wait for an existing turn's result. |
| Observe | `inspect BOX --agent AGENT [--turn TURN]` | Read agent or turn state. |
| Observe | `events BOX --agent AGENT [--turn TURN]` | Follow agent or turn events. |
| Interact | `steer BOX --agent AGENT --turn TURN TEXT` | Redirect that turn without submitting another. |
| Interact | `interrupt BOX --agent AGENT --turn TURN` | Interrupt that turn's agent loop. |
| Interact | `respond BOX --agent AGENT --request REQUEST JSON` | Answer a question or approve/deny a permission request. |

`BOX` is positional, as in `acrylmicrovm exec`. Creation requires `--agent IMPLEMENTATION`.
For example, `acrylmicrovm agent create dev --agent opencode` selects implementation
`opencode` inside Box `dev`. Other commands use `--agent ID` for a created instance.
Mutations never infer an active turn. `inspect --view config|capabilities|requests` selects an agent-only
read and cannot be combined with `--turn`; omission reads state. The SDK's
`get`/`get_turn` handle lookups are implicit in CLI targeting.

The optional flags are grouped by operation:

| Group | Commands | Flags |
|---|---|---|
| `AgentCreateOptions` | `create` | `--cwd PATH`, `--agent-config JSON`, `--idempotency-key KEY` |
| `AgentSendOptions` | `send` | `--mode queue\|reject` (default `queue`), `--data JSON`, `--idempotency-key KEY` |
| `AgentControlOptions` | close, respond, steer, interrupt | `--idempotency-key KEY` |
| `AgentWaitOptions` | `wait` | `--timeout-seconds SECONDS` (nonnegative whole seconds; omitted means no deadline) |
| `AgentEventsOptions` | `events` | `--after CURSOR` |
| `AgentInspectOptions` | `inspect` | `--view config\|capabilities\|requests` |

`--agent-config` is distinct from the global runtime `--config FILE`. Config,
data and response arguments must be valid JSON. Response JSON uses either
`{"type":"answer","text":"yes"}` (optional `data`) or
`{"type":"permission","approved":false}`. The
[agent API reference](../agents.md#inputs-and-records) describes other record fields.

Output is JSON on stdout, in the
[record shapes of the C API](../agents.md#c-api-and-ownership):

| Command | Prints | Exit code |
|---|---|---|
| `create` | `{"id":"ses_…"}`, the ID later commands pass as `--agent` | 0 |
| `send`, `respond`, `inspect --turn` | Turn snapshot | 0 |
| `inspect` | Agent snapshot | 0 |
| `inspect --view config\|capabilities\|requests` | Config, capabilities, request array | 0 |
| `wait` | Result, including for `failed` and `cancelled` turns; branch on its `status` | 0; 1 when `--timeout-seconds` expires or the turn is `unknown`, which never interrupts it |
| `steer` | Steering receipt | 0 |
| `events` | One event record per line, flushed as it arrives, until the stream ends | 1 when the stream ends with an error, after every earlier record |
| `close`, `interrupt` | Nothing | 0 |

The agent-level OpenCode stream always ends with a `Network` error, for example
when its Box stops, so `events` without `--turn` exits with code 1.

```bash
acrylmicrovm create --name dev debian:bookworm-slim sleep infinity
acrylmicrovm start dev
acrylmicrovm agent create dev --agent opencode --cwd /workspace --agent-config '{"model":"provider/model"}'
# {"id":"ses_01K…"}
acrylmicrovm agent send dev --agent ses_01K… "Review this code"
# {"id":"msg_01K…","agent_id":"ses_01K…","status":"queued",…}
acrylmicrovm agent wait dev --agent ses_01K… --turn msg_01K… --timeout-seconds 600
# {"turn_id":"msg_01K…","status":"completed","outcome":"answer","text":"…",…}
acrylmicrovm agent events dev --agent ses_01K…   # until the stream ends; exit 1
```

Agent commands accept the usual Box
targeting globals (`--home`, `--config`, `--url`, `--context`, `--path-prefix`)
and `--debug`; `--registry` is irrelevant and rejected. Existing diagnostic
logging still applies.

### `acrylmicrovm serve`

**Synopsis:** `acrylmicrovm serve [OPTIONS]`

Run a long-running REST API server. The server holds a single `AcrylMicrovmRuntime` and exposes the full REST surface for `acrylmicrovm --url ...` clients and the language SDKs' REST mode.

**Options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--debug` | `false` | Enable debug output |
| `--port N` | `8100` | TCP port to listen on |
| `--home PATH` | `~/.acryl/microvm` | Absolute runtime data directory; env `ACRYLMICROVM_HOME` |
| `--host ADDR` | `127.0.0.1` | Bind address. Any non-loopback address (e.g. `0.0.0.0`) requires `--api-key`; `serve` refuses to start otherwise. |
| `--api-key KEY` | unset | Require this exact `Authorization: Bearer` value (constant-time match) on every route except `GET /v1/config`; env `ACRYLMICROVM_SERVE_API_KEY`. Unset is permissive. |
| `--registry REGISTRY` | none | Image registry; repeatable and prepended to config |
| `--config PATH` | none | JSON runtime configuration file |

**Examples:**

```bash
acrylmicrovm serve
acrylmicrovm serve --port 9000
ACRYLMICROVM_SERVE_API_KEY="$KEY" acrylmicrovm serve --host 0.0.0.0
```

---

### `acrylmicrovm volume`

**Synopsis:** `acrylmicrovm volume <create|ls|get|rm>`

Manage managed persistent volumes. Volumes are a REST-runtime capability — the
local runtime has no volume backend, so every subcommand returns
`named volumes are not supported yet` against it. See
[Connecting to the cloud](#connecting-to-the-cloud).

| Subcommand | Synopsis | Notes |
|---|---|---|
| `create` | `acrylmicrovm volume create [--name NAME]` | Prints the new id |
| `ls` (`list`) | `acrylmicrovm volume ls [-q \| --format FORMAT]` | `table`, `json`, or `yaml` (default `table`); quiet and format conflict |
| `get` (`inspect`) | `acrylmicrovm volume get ID [--format FORMAT]` | `table`, `json`, or `yaml` (default `table`); by id |
| `rm` (`delete`) | `acrylmicrovm volume rm ID... [--force]` | By id; `--force` ignores missing volumes |

**`--name`** is mountable in place of the id, so a box can ask for the volume it
wants without knowing the id:

```bash
acrylmicrovm volume create --name my-data       # prints e.g. vol_01K2EXAMPLE
acrylmicrovm run -v my-data:/data alpine        # by name
acrylmicrovm run -v vol_01K2EXAMPLE:/data alpine # or by id
```

Names must be at least two characters of `[a-zA-Z0-9][a-zA-Z0-9_.-]` — Docker's
rule, and for the same reason: the name has to survive
[`-v` parsing](#volume-mount-syntax). A leading `.`, `/` or `~` would classify
the source as a host path; a `:` would split the spec into the wrong fields.
Neither name could ever be mounted. Without `--name` the server names the
volume after its id, which stays mountable.

`get` and `rm` take an id only; mounting is what accepts either.

---

### `acrylmicrovm domain`

**Synopsis:** `acrylmicrovm domain <bind|ls|unbind>`

The domains your organization holds: names under the platform domain
(`demo.apps.example`) routed to a box port. A domain is the organization's,
not a box's. Hosted runtimes only; the embedded runtime answers "not
supported". See [Domains](../../concepts/networking.md#domains).

| Subcommand | Synopsis | Notes |
|---|---|---|
| `bind` | `acrylmicrovm domain bind DOMAIN --type TYPE [target options]` | `--type` is required; currently only `box`, which requires `--box BOX --port PORT` |
| `ls` (`list`) | `acrylmicrovm domain ls [-q \| --format FORMAT]` | Every full hostname and its target (`-` while unbound); table adds the allowance footer, JSON/YAML return an array of domain rows, quiet prints full hostnames |
| `unbind` | `acrylmicrovm domain unbind HOSTNAME` | Full hostname required; it stays with the organization |

For `bind`, `DOMAIN` is an owned full hostname (`demo1.apps.example`).
Request it first in **Dashboard → Domains** using a short name; the page shows
the full hostname and its binding. Select the
target with `--type`: currently only `box` is supported, with `--box` (an id
or name) and `--port` (1–65535) required for that type. The CLI rejects missing
or unsupported types before sending a request. Scripts using the previous
implicit Box syntax must add `--type box`. Binding prints the target and URL.
`unbind` accepts only a full hostname returned by `ls`; bare labels are rejected.
`bind` never requests a name: an unowned hostname returns `404 domain_not_owned`
and asks you to use Dashboard → Domains. Binding moves a held name from another
box port at once; other names on the target port stay. It does not make the port
public: the name answers once `acrylmicrovm network tunnel` has declared the port
and the box has inbound access enabled, in either order. `unbind` stops
routing the name;
requests that start afterwards get `404`, and the name still counts against
the allowance. Binding it again to a live box port does not count it twice.
New bindings reject a Box being destroyed or already removed. Existing bindings
survive Box deletion; deleting the owning organization releases its names.

```bash
acrylmicrovm domain bind demo1.apps.example --type box --box web-a --port 3000   # demo1.apps.example → web-a:3000
acrylmicrovm domain bind demo1.apps.example --type box --box web-b --port 3000   # move it, same URL
acrylmicrovm domain ls
acrylmicrovm domain unbind demo1.apps.example
```

---

### `acrylmicrovm completion`

**Synopsis:** `acrylmicrovm completion <SHELL>`

Print a shell completion script to stdout. *Hidden from `--help`* but functional.

**Supported shells:** `bash`, `zsh`, `fish`.

**Examples:**

```bash
acrylmicrovm completion bash > /etc/bash_completion.d/acrylmicrovm
acrylmicrovm completion zsh  > "${fpath[1]}/_acrylmicrovm"
acrylmicrovm completion fish > ~/.config/fish/completions/acrylmicrovm.fish
```

---

## Shared flag groups

Several commands flatten shared `clap` `Args` structs. Each is documented here once.

### `ProcessFlags`

Used by `run` and `exec` (defined in `src/cli/src/cli.rs`).

| Flag | Short | Description |
|------|-------|-------------|
| `--interactive` | `-i` | Keep STDIN open even if not attached; conflicts with `--detach` |
| `--tty` | `-t` | Allocate a pseudo-TTY (stdout and stderr are merged in TTY mode) |
| `--env KEY[=VALUE]` | `-e` | Set environment variables (repeatable; a bare key inherits from the host) |
| `--workdir PATH` | `-w` | Working directory inside the box |
| `--user NAME[:GROUP]` | `-u` | Run as `name`/`uid`[:`group`/`gid`] |

For `run`, `--tty` always requires TTY-attached stdin, including with
`--detach`. Foreground `exec --tty` has the same requirement and implies
interactive input; detached `exec --tty` skips the stdin check and attaches no
stdin.

### `CapabilityFlags`

Used by `run` and `create` to adjust the Linux capability set inherited by the
container's init and every later `exec` process.

| Flag | Description |
|------|-------------|
| `--cap-add CAPABILITY` | Add a capability; repeatable |
| `--cap-drop CAPABILITY` | Drop a capability; repeatable |

Names are case-insensitive and may include the `CAP_` prefix. `ALL` is
supported. With neither flag, ACRYL microVMs keeps its Docker-compatible 14-capability
baseline. `--cap-drop ALL --cap-add NET_BIND_SERVICE` creates a minimal set
containing only `NET_BIND_SERVICE`.

### `ResourceFlags`

Used by `run` and `create` (defined in `src/cli/src/cli.rs`).

| Flag | Type | Description |
|------|------|-------------|
| `--cpus N` | f64 | CPU allocation, 0.125–16 with the bundled kernel, step 0.001; fractions require a Linux execution host and round guest vCPUs up, while integers add no bandwidth quota; creation validates numeric values |
| `--memory MiB` | u32 | Memory limit in mebibytes |
| `--disk-size GB` | u64 | Sparse root filesystem disk size in gigabytes |

### `PublishFlags`

Used by `run` and `create` (defined in `src/cli/src/cli.rs`).

| Flag | Short | Description |
|------|-------|-------------|
| `--publish PORT` | `-p` | Publish a box port to the host; repeatable (see [Port Publish Syntax](#port-publish-syntax)) |

TCP is the only supported publication protocol; UDP is rejected.

### `VolumeFlags`

Used by `run` and `create` (defined in `src/cli/src/cli.rs`).

| Flag | Short | Description |
|------|-------|-------------|
| `--volume VOLUME` | `-v` | Mount a volume; repeatable (see [Volume Mount Syntax](#volume-mount-syntax)) |

### `NetworkFlags`

Used by `run` and `create` (defined in `src/cli/src/cli.rs`).

| Flag | Description |
|------|-------------|
| `--network <enabled\|disabled>` | Outbound mode; default `enabled`. Disabled mode creates no network interface. |
| `--allow-net HOST` | Restrict TCP/UDP egress to exact hosts, `*.example.com`, IPv4 addresses, or IPv4 CIDRs; repeatable, implies enabled networking, and is incompatible with `--network disabled`. Hostname-only rules deny UDP unless an IP/CIDR is also allowed. |
| `--inbound <enabled\|disabled>` | Inbound mode; default `disabled` (services exposed by the box are unreachable from outside). |
| `--net-tx-kbps KBPS` | Cap what the box sends (guest to internet), in kilobits/sec. `0` or unset leaves it uncapped. |
| `--net-rx-kbps KBPS` | Cap what reaches the box (internet to guest), in kilobits/sec. `0` or unset leaves it uncapped. |

Bandwidth caps are enforced below IP by the local gvproxy bridge, so a single
budget per direction covers TCP, UDP, ICMP and ARP together, and inbound
port-forward traffic counts against the same budget as outbound requests —
the cap is on the box's interface, not on a connection's direction.
Directions are named from the box's point of view, matching Firecracker.
Remote runtimes reject these flags: the server owns its own network policy.
Verified on Linux; on macOS the guest link is a datagram socket whose sender
behaviour under backpressure is not yet verified, so `--net-tx-kbps` may drop
frames there instead of slowing the guest.

### `ManagementFlags`

Used by `run` and `create` (defined in `src/cli/src/cli.rs`).
`--detach` and `--rm` belong only to `run`; `create` is always detached and
does not expose remove-on-stop.

| Flag | Short | Description |
|------|-------|-------------|
| `--name NAME` | — | Assign a name to the box |
| `--auto-stop DURATION` | — | Stop the box after this much inactivity; `0` disables |
| `--auto-delete DURATION` | — | Delete the box this long after it stops; `0` disables |
| `--no-auto-resume` | — | Ask a REST server to refuse implicit resume after a box has run; first boot is unaffected, and the embedded runtime records but does not enforce it |
| `--security <enable\|disable>` | — | Sandbox security for the embedded runtime (env `ACRYLMICROVM_SECURITY`, default `enable`); REST servers own this policy and do not accept a client override |

`DURATION` is seconds when bare, or a suffixed value: `30s`, `15m`, `2h`, `7d`.

> `--auto-stop` and `--auto-delete` are deadlines a sweeper acts on, so they need a server — `acrylmicrovm serve` or the cloud. Against the embedded runtime a non-zero value is refused rather than silently reinterpreted. The sweep runs on a 30s tick, so a deadline fires on the first tick at or after it. A server also requires `auto_delete` to exceed `auto_stop`; a box cannot be scheduled for deletion sooner than it is scheduled to stop.

> `--rm` removes the box when it stops. Against the embedded runtime that is synchronous, at the stop itself. Against `acrylmicrovm serve` it is carried as the shortest possible deadline and swept on the same 30s tick, so removal can lag the stop by up to one tick — and, because `serve` holds that policy in memory, is skipped entirely if the server restarts in between.

> `--rm` with `--detach` on `run` is silently downgraded — a detached box outlives the CLI process, so it keeps manual lifecycle control. An explicit `--auto-delete` survives detaching, which is the pairing the flag exists for. Use `acrylmicrovm rm` to clean up.

---

## Volume mount syntax

`-v`/`--volume` accepts the grammar implemented in `src/cli/src/volumespec.rs`:

```
VOLUME := SOURCE ':' BOX_PATH [':' OPTIONS]             # managed volume or bind mount
        | BOX_PATH [':' OPTIONS]                         # anonymous volume
```

`SOURCE` is classified by its **first character**, the same rule Docker uses
(`docker/cli/internal/volumespec`). Nothing inspects the filesystem, so a spec
means the same thing on every machine:

| `SOURCE` starts with | Meaning |
|---|---|
| `/`, `./`, `../`, `~` | Host path |
| `\\` (UNC / named pipe) | Host path |
| A drive letter, e.g. `C:\` or `D:/` | Host path |
| Anything else | **Managed volume**, by id or by name |

| Form | Example | Behavior |
|------|---------|----------|
| `BOX_PATH` | `/data` | Anonymous volume stored under `{home}/volumes/anonymous/<ulid>` |
| `BOX_PATH:ro` / `BOX_PATH:rw` | `/data:ro` | Anonymous volume with explicit mode |
| `VOLUME:BOX_PATH` | `my-data:/data` | Managed volume by name |
| `VOLUME:BOX_PATH` | `vol_01K2EXAMPLE:/data` | Managed volume by server-assigned id |
| `HOST_PATH:BOX_PATH` | `/host/data:/data` | Bind mount (host directory must exist) |
| `HOST_PATH:BOX_PATH:OPTIONS` | `./data:/data:ro` | Bind mount with options |
| `C:\HOST\PATH:/BOX_PATH[:OPTIONS]` | `C:\data:/app/data:ro` | Windows drive paths are handled — the drive-letter colon is not treated as a separator |

**Options:** `ro` (read-only) or `rw` (read-write, default). Other options are ignored. Relative host paths are canonicalized at parse time; missing host paths fail with `volume host path ...`.

**Runtime support.** Managed volumes require a REST runtime — the local runtime
has no volume backend and rejects them at create. Host binds are the mirror
image: they name a path on the machine running the box, so a REST runtime
refuses them. Manage volumes with [`acrylmicrovm volume`](#acrylmicrovm-volume).

> **Behavior change.** A bare relative source is no longer a bind mount:
> `-v data:/app` now means the managed volume `data`, not `./data`. Write
> `./data:/app` for the bind. Unlike Docker, a mistyped name cannot silently
> create an empty volume — acrylmicrovm never auto-creates, so an unknown reference
> fails with "Volume 'data' not found".

**Single-character names are not addressable via `-v`.** The parser cannot
distinguish `a:` from a drive letter, so `-v a:/data` is read as one field and
fails. Docker has the same limitation and rejects one-character volume names
outright.

The anonymous-volume base directory is resolved as: `--home`, else `$ACRYLMICROVM_HOME`, else `~/.acryl/microvm`, else the system temp dir.

---

## Port publish syntax

`-p`/`--publish` accepts the grammar implemented at
`src/cli/src/cli.rs`:

```
PORT := [HOST_PORT ':'] BOX_PORT ['/tcp']
```

| Form | Example | Behavior |
|------|---------|----------|
| `BOX_PORT` | `80` | Let the OS select an available host port and forward it to box port `80` |
| `HOST_PORT:BOX_PORT` | `8080:80` | Forward host port `8080` to box port `80` |
| `HOST_PORT:BOX_PORT/PROTO` | `8080:80/tcp` | Full form |

Ports must be in `1..=65535`. TCP is the only supported protocol; UDP is rejected.
`-p` is explicit local publication and is rejected for remote REST runtimes.
For a remote box, use `acrylmicrovm network tunnel BOX PORT` to obtain its public
service URL. For SDK code that must run with either runtime, use the box network
tunnel API; each returned tunnel is one-shot.
Image `EXPOSE` declarations remain metadata and do not open host listeners.

---

## Output formats

The `--format` flag is shared across `list`, `images`, `inspect`, `stats`,
`info`, `volume ls`, `volume get`, and `domain ls`. Valid values come from
`OutputFormat::from_str` at `src/cli/src/formatter.rs:26`:

| Format | Available on | Description |
|--------|--------------|-------------|
| `table` | `list`, `images`, `stats`, `volume ls`, `volume get`, `domain ls` | Human-readable columnar layout |
| `json` | `list`, `images`, `stats`, `inspect`, `info`, `volume ls`, `volume get`, `domain ls` | Pretty-printed JSON (`serde_json::to_string_pretty`) |
| `yaml` | `list`, `images`, `stats`, `inspect`, `info`, `volume ls`, `volume get`, `domain ls` | YAML (`serde_yaml::to_string`) |
| Go template | `inspect` only | Any `gtmpl` template, e.g. `'{{.State.Status}}'`; `{{json .Field}}` serializes a nested value |

Defaults:

- `list`, `images`, `stats`, `volume ls`, `volume get`, `domain ls`: `table`
- `inspect`: `json`
- `info`: `yaml`

---

## Configuration file

`--config PATH` accepts a JSON file deserialized into `AcrylMicrovmOptions`. The primary field is `image_registries`; `image_registry_proxy` sets a pull proxy (see [Pull proxy](../../guides/image-registry-configuration.md#pull-proxy)). CLI flags like `--home` and `--registry` are layered on top after loading. A context that targets a remote server ignores this file.

```json
{
  "home_dir": "/custom/.acrylmicrovm",
  "image_registries": [
    {
      "host": "registry.example.com",
      "protocol": "https",
      "search": true,
      "username": "user",
      "password": "password"
    },
    {
      "host": "127.0.0.1:5000",
      "protocol": "http",
      "search": false
    }
  ]
}
```

For a richer treatment of the registry config (auth flows, fallbacks, mirrors), see [`docs/guides/image-registry-configuration.md`](../../guides/image-registry-configuration.md).

---

## Exit codes

`acrylmicrovm` follows POSIX shell exit-code conventions. The mapping lives at `src/cli/src/util/mod.rs:11-15`.

| Code | Source | Meaning |
|------|--------|---------|
| `0` | success | Command (or box command) finished successfully |
| `1` | runtime | Any anyhow error from a CLI command — `main.rs:71` prints `Error: ...` to stderr and exits `1` |
| `2` | clap | Invalid CLI usage (unknown flag, missing required arg, bad value) |
| `N` (1-127) | box command | `run`/`exec` propagate the command's exit status |
| `126` | exec startup refusal | Special case: the command exists but is not executable |
| `127` | exec startup refusal | Special case: the command was not found |
| `128 + N` | signal | `run`/`exec` exited because the box command was killed by signal *N* (e.g. `137` for `SIGKILL`, `143` for `SIGTERM`) |

`126`/`127` are the only special cases: an `exec` startup refusal. Every other value keeps its usual meaning: the command's own exit status, a CLI error, or a signal.

`acrylmicrovm rm`, `start`, `stop`, `restart` aggregate per-target errors and exit `1` if any target failed, after attempting all targets.

---

## See also

- [`src/cli/README.md`](../../../src/cli/README.md) — quick start, install alternatives, common workflows
- [`docs/reference/README.md`](../README.md) — reference index (SDKs + CLI)
- [`docs/guides/image-registry-configuration.md`](../../guides/image-registry-configuration.md) — registry config deep dive
- [`docs/getting-started/`](../../getting-started/) — per-language getting-started guides
