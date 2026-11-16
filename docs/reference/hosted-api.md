# Hosted API: what `api.webboxes.ai` has to provide

ACRYL microVMs runs boxes on your own machine. The hosted counterpart, the same API served for many users
from one address, is **`https://api.webboxes.ai`**. The SDKs, the CLI and the end-to-end suite already point
at it. This page says what that service has to implement so they work unchanged.

> **Status.** The hosted API is not part of this repository and is not built yet. Only the production host
> `api.webboxes.ai` is named by the owner. The names `api.dev.webboxes.ai`, `api.stage.webboxes.ai`,
> `auth.webboxes.ai`, `dex.webboxes.ai`, `app.webboxes.ai` and `*.proxy.dev.webboxes.ai` are the old stage
> names moved to the new domain; treat them as placeholders until they exist.

## The rule: same operations, same formats

The contract is the two OpenAPI files in [`openapi/`](../../openapi):

| File | What it covers | Operations |
| --- | --- | --- |
| [`box.openapi.yaml`](../../openapi/box.openapi.yaml) | Boxes, executions, files, images, snapshots, volumes, domains, checkpoints, metrics, tunnels, `GET /v1/config`, `GET /v1/me` | 58 |
| [`admin.openapi.yaml`](../../openapi/admin.openapi.yaml) | Organization users and invitations | 5 |

Implement the paths, methods, `operationId`s, request and response schemas, status codes and error format exactly
as written there. Do not rename or reshape anything. The generated clients in [`apps/libs`](../../apps/libs) and the
SDKs are built from these files, so a difference shows up as a client bug.

Names this project changed from the code it was derived from are already applied in the files. The hosted
service must use the **current** names:

- headers: `X-AcrylMicrovm-Organization-ID`, `X-AcrylMicrovm-Authorization`, `X-AcrylMicrovm-Access-Token`,
  and the response header `x-acrylmicrovm-execution-id`;
- the query parameter `acrylmicrovm-access-token`;
- error `type` values such as `InvalidArgumentError` (the `AcrylMicrovmError` variants).

If the owner later picks a different product name, this page and the OpenAPI files change together.

## Where clients look for it

| Client | Setting | Default |
| --- | --- | --- |
| CLI | `--url`, then `ACRYLMICROVM_REST_URL`, then the selected context, then the built-in default | `https://api.webboxes.ai` (`src/cli/src/defaults.rs`) |
| SDKs | the `url` option of the REST runtime | none; examples use `https://api.webboxes.ai` |
| End-to-end suite | `ACRYLMICROVM_E2E_API_URL` | see [`apps/e2e/README.md`](../../apps/e2e/README.md) |

## What to create

Nothing below needs the removed hosted-platform code. The Apache-2.0 runtime in this repository already does the
single-machine work; the hosted service adds users, routing and fleet management around it.

### 1. Identity and discovery (needed first)

- `GET /v1/config`, **no authentication**. Returns `defaults` (cpus, memory, disk, auto-delete), `overrides`,
  and `capabilities`. An operation the hosted service does not offer must be reported as unsupported here, so
  clients can say so before calling it.
- Login discovery for the CLI (optional). `acrylmicrovm auth login` asks for `GET {url}/config`, where `{url}`
  is the server URL exactly as configured, so with the default it requests `https://api.webboxes.ai/config`,
  not `/v1/config`. Serve the configuration document at both. It may carry an `oidc` object with `issuer`,
  `clientId` (or `client_id`) and `audience`. That object is **not in the OpenAPI schema**; it is an extension the
  CLI reads. Without it the CLI falls back to the `OIDC_ISSUER`, `OIDC_CLIENT_ID` and `OIDC_AUDIENCE` environment
  variables, and API-key login does not need it.
- `GET /v1/me`. Returns the caller's `sub`, `principal_type` (`user` or `service_account`), `scopes`, optional
  `email` and `display_name`, and **`path_prefix`**, the value clients put in the `{prefix}` slot of box-scoped
  URLs. A null or empty `path_prefix` means URLs have no prefix (`/v1/boxes/...`).
  Known mismatch: the single-machine server (`acrylmicrovm serve`) names this field `prefix`, not `path_prefix`
  as the spec does. The hosted service should return `path_prefix`; returning both is harmless.
- Bearer authentication on everything else (`Authorization: Bearer <token>`, RFC 6750). The token sources are
  listed under `BearerAuth` in `box.openapi.yaml`. The first is required: **API keys** shaped `blk_live_...` and
  `blk_test_...`, scoped to an organization, revoked when their creator leaves it. Issuing them needs a page at
  `https://app.webboxes.ai/settings/api-keys`.
- An OIDC issuer for browser and device login (the CLI supports both), if interactive login is wanted.

### 2. Boxes (the core)

Create, list, get, existence check (`HEAD`), remove, start, stop. Then command execution (start, list, status,
kill, signal, resize), the two WebSocket attach endpoints (`attachBox`, `attachExecution`) with the frame
protocol, single-attach rule and keepalive described in the spec, and file upload and download. Plus `GET images`
and `GET images/{image_name}`, and per-box and runtime metrics.

Also required across all of these: the error body (`ErrorResponse` with `message`, `type`, `code`,
`request_id`; `X-Request-Id` on every response; the Box API only, see section 6 for Admin), opaque page tokens, and the `Idempotency-Key` header on
mutations.

### 3. State features

Volumes, snapshots (create, list, get, remove, restore), clone, export and import, checkpoints, and guest SSH
(enable, configure, status, disable). Report any you leave out as unsupported in `/v1/config`.

### 4. Networking features

Outbound and inbound policy updates, service tunnels (`prepareTunnel`, `revokeTunnel`) and custom domains
(`listDomains`, `claimDomain`, `bindDomain`, `unbindDomain`). Tunnels need a **preview proxy**: a host name per
tunnel (the e2e fixtures use `<port>-<token>.proxy.<domain>`) that forwards HTTP to the box port and accepts an
HTTP/1.1 `CONNECT` for raw TCP, authenticated by the headers above.

### 5. Image registries

`GET/POST/DELETE images/registries` and image aliases. Private registry credentials must reach the machine that
pulls the image without being exposed to the guest. The old deployment used a registry proxy for this.

### 6. Organizations

The five admin operations: list and remove users, list, create and cancel invitations. No SDK or CLI command
calls them yet, so build them last. They use a **flat** error body, not the `ErrorResponse` of the Box API;
`admin.openapi.yaml` gives the exact shape.

### 7. What runs the boxes

The API itself does not run VMs. Each host that does needs the runtime; the simplest choice is
`acrylmicrovm serve` (`src/cli/src/commands/serve`), which already implements the Box API for one machine. The
hosted service then is a gateway that authenticates, picks a host for each box, forwards requests (including
WebSockets) and enforces quotas. Scheduling, quotas and billing are deployment decisions and are not in the
contract.

## Order of work

1. `/v1/config`, `/v1/me`, API-key authentication, box create/get/list/remove/start/stop, exec, attach, files,
   images list/get. This is enough for the SDK quick-start and the CLI.
2. Metrics, volumes, snapshots, clone, export/import, checkpoints, SSH.
3. Tunnels, domains, outbound/inbound policy, registries and aliases.
4. Organizations, OIDC login, quotas.

## How to test an implementation

- **Mock for client tests:** the reference server in [`openapi/reference-server`](../../openapi/reference-server)
  implements part of the Box API on top of the local runtime (single user, no persistence). It is a client test
  fixture, not a conformance target and not for production. Extend it, or write a small mock per feature, when a
  test needs a response it does not give.
- **Single-machine server:** `acrylmicrovm serve` listens on `127.0.0.1:8100` and answers at `/v1/...`.
- **Conformance:** the suite in [`apps/e2e`](../../apps/e2e) drives the SDKs and the CLI against any server named
  by `ACRYLMICROVM_E2E_API_URL`. The cloud-only cases assume the hosted features above.
- **Contract drift:** regenerate a client from the OpenAPI files and run it against the service; a spec change
  and a code change land together.

## Licensing and where code may come from

The OpenAPI files are Apache-2.0, so building the service to them is allowed. Write it from the spec. The
previous hosted platform was AGPL-3.0 code derived from another project and was removed from this repository
(see `NOTICE`); do not copy it, from this repository's history or elsewhere, into the hosted service. If any AGPL
code is ever run as a service for other people, its source has to be offered to those users, so keep it in a
separate service.
