# Dashboard

A small live view of a Box API server, made for demos. It lists the boxes, starts new ones, and opens a real
terminal inside any running box, so you can show its own filesystem, its own kernel and the programs in its
image (for example Node.js in `node:slim`).

It works with anything that serves the Box API: `acrylmicrovm serve` on your machine, or a hosted service.

## Run

Node.js 22 or newer, no dependencies.

```sh
acrylmicrovm serve --port 8100 --api-key "$KEY" &
BOX_API_URL=http://127.0.0.1:8100 BOX_API_KEY="$KEY" node examples/dashboard/server.mjs
# open http://127.0.0.1:8300
```

| Variable | Default | Meaning |
| --- | --- | --- |
| `BOX_API_URL` | `http://127.0.0.1:8100` | the Box API server |
| `BOX_API_KEY` | none | sent as `Authorization: Bearer`; stays in the server process, never reaches the browser |
| `BOX_API_PREFIX` | none | the `{prefix}` path segment, if the server uses one |
| `HOST`, `PORT` | `127.0.0.1`, `8300` | where the dashboard listens |

The terminal uses xterm.js from a CDN, so the browser needs internet access.

## How it works

- The page calls this server; this server calls the Box API with the key.
- **Terminal:** the server starts a login shell in the box (`POST /boxes/{id}/exec` with `tty: true`), then relays
  the browser's WebSocket to `GET /boxes/{id}/executions/{exec_id}/attach` byte for byte. The browser speaks the
  attach frame protocol directly: keystrokes as binary frames, `{"type":"resize"}` as text, output as
  `[channel][bytes]`.
- **Preset buttons** type a command into the terminal, as if you typed it.
- The host kernel is shown only when the Box API runs on the same machine as the dashboard.

## Safety

Anyone who can open the dashboard can run commands in every box the key can reach. It listens on `127.0.0.1` by
default; reach it from another machine through an SSH tunnel (`ssh -L 8300:127.0.0.1:8300 host`) rather than by
binding it to a public address. WebSocket upgrades from other origins are refused.

## Test

```sh
node --test examples/dashboard/server.test.mjs
```
