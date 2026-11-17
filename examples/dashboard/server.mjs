// ACRYL microVMs dashboard: a small live view of any Box API server (acrylmicrovm serve, or a hosted gateway).
// Node >= 22, no dependencies. The API key stays in this process; the browser only talks to this server.
import http from 'node:http';
import net from 'node:net';
import os from 'node:os';
import tls from 'node:tls';
import { readFile } from 'node:fs/promises';

const API = (process.env.BOX_API_URL ?? 'http://127.0.0.1:8100').replace(/\/+$/, '');
const KEY = process.env.BOX_API_KEY ?? '';
const PREFIX = process.env.BOX_API_PREFIX ? `/${process.env.BOX_API_PREFIX}` : '';
const HOST = process.env.HOST ?? '127.0.0.1';
const PORT = Number(process.env.PORT ?? 8300);
const RUN_LIMIT_MS = 120_000;
// The host kernel is only meaningful when the boxes run on this same machine.
const LOCAL_API = /^https?:\/\/(127\.0\.0\.1|localhost|\[::1\])(:|\/|$)/.test(API);

const auth = () => (KEY ? { authorization: `Bearer ${KEY}` } : {});

async function api(method, path, body) {
  const res = await fetch(`${API}/v1${PREFIX}${path}`, {
    method,
    headers: { ...auth(), ...(body ? { 'content-type': 'application/json' } : {}) },
    body: body ? JSON.stringify(body) : undefined,
  });
  const text = await res.text();
  let data = null;
  try { data = text ? JSON.parse(text) : null; } catch { data = { message: text }; }
  if (!res.ok) {
    const message = data?.error?.message ?? data?.message ?? `${method} ${path}: HTTP ${res.status}`;
    throw Object.assign(new Error(message), { status: res.status });
  }
  return data;
}

/** One attach frame from the server: binary [channel][payload] or a JSON control message. */
export function decodeFrame(data) {
  if (typeof data === 'string') {
    const msg = JSON.parse(data);
    if (msg.type === 'exit') return { kind: 'exit', code: msg.exit_code };
    if (msg.type === 'error') return { kind: 'stderr', text: `${msg.message}\n` };
    return null;
  }
  const bytes = new Uint8Array(data);
  if (bytes.length === 0) return null;
  const text = new TextDecoder().decode(bytes.subarray(1));
  return { kind: bytes[0] === 2 ? 'stderr' : 'stdout', text };
}

/** Run `sh -c cmd` in a box and stream NDJSON events ({kind, text|code}) to `send` until it exits. */
async function run(boxId, cmd, send) {
  const { execution_id: execId } = await api('POST', `/boxes/${encodeURIComponent(boxId)}/exec`, {
    command: 'sh', args: ['-c', cmd],
  });
  const url = `${API.replace(/^http/, 'ws')}/v1${PREFIX}/boxes/${encodeURIComponent(boxId)}/executions/${encodeURIComponent(execId)}/attach?stdin=0`;
  await new Promise((resolve) => {
    const ws = new WebSocket(url, { headers: auth() });
    ws.binaryType = 'arraybuffer';
    let done = false;
    const finish = (event) => { if (done) return; done = true; clearTimeout(timer); if (event) send(event); resolve(); try { ws.close(); } catch {} };
    const timer = setTimeout(() => {
      api('DELETE', `/boxes/${encodeURIComponent(boxId)}/executions/${encodeURIComponent(execId)}`).catch(() => {});
      finish({ kind: 'stderr', text: `stopped after ${RUN_LIMIT_MS / 1000} s\n` });
    }, RUN_LIMIT_MS);
    ws.onmessage = (e) => {
      const event = decodeFrame(e.data);
      if (!event) return;
      if (event.kind === 'exit') finish(event); else send(event);
    };
    ws.onerror = () => finish({ kind: 'stderr', text: 'connection to the box failed\n' });
    ws.onclose = () => finish(null);
  });
}

async function state() {
  const [{ boxes }, config] = await Promise.all([api('GET', '/boxes'), api('GET', '/config').catch(() => null)]);
  return {
    api: API.replace(/^https?:\/\//, ''),
    hostKernel: LOCAL_API ? os.release() : null,
    hostName: LOCAL_API ? os.hostname() : null,
    capabilities: config?.capabilities ?? null,
    boxes: boxes.map((b) => ({
      id: b.box_id ?? b.id, name: b.name, status: b.status, image: b.image,
      cpus: b.cpus, memoryMib: b.memory_mib, createdAt: b.created_at,
    })),
  };
}

async function readJson(req) {
  let size = 0; const chunks = [];
  for await (const c of req) { size += c.length; if (size > 64 * 1024) throw Object.assign(new Error('request too large'), { status: 413 }); chunks.push(c); }
  return chunks.length ? JSON.parse(Buffer.concat(chunks).toString('utf8')) : {};
}

const files = {
  '/': ['index.html', 'text/html; charset=utf-8'],
  '/logo.png': ['../../docs/assets/logo/acrylmicrovm-logo.png', 'image/png'],
};

const server = http.createServer(async (req, res) => {
  const json = (status, body) => { res.writeHead(status, { 'content-type': 'application/json' }); res.end(JSON.stringify(body)); };
  try {
    const { pathname } = new URL(req.url, 'http://x');
    const box = pathname.match(/^\/api\/boxes\/([^/]+)(\/[a-z]+)?$/);
    if (req.method === 'GET' && files[pathname]) {
      const [file, type] = files[pathname];
      res.writeHead(200, { 'content-type': type, 'cache-control': 'no-store' });
      return res.end(await readFile(new URL(file, import.meta.url)));
    }
    if (req.method === 'GET' && pathname === '/api/state') return json(200, await state());
    if (req.method === 'POST' && pathname === '/api/boxes') {
      const { image = 'alpine:latest', name } = await readJson(req);
      const created = await api('POST', '/boxes', { image, ...(name ? { name } : {}) });
      const id = created.box_id ?? created.id;
      await api('POST', `/boxes/${encodeURIComponent(id)}/start`);
      return json(201, { id });
    }
    if (box && req.method === 'DELETE' && !box[2]) {
      await api('DELETE', `/boxes/${encodeURIComponent(box[1])}?force=true`);
      return json(200, { ok: true });
    }
    if (box && req.method === 'POST' && (box[2] === '/start' || box[2] === '/stop')) {
      await api('POST', `/boxes/${encodeURIComponent(box[1])}${box[2]}`);
      return json(200, { ok: true });
    }
    if (box && req.method === 'POST' && box[2] === '/shell') {
      // A login shell with a terminal; the browser then attaches to it through the upgrade relay below.
      const { execution_id: execId } = await api('POST', `/boxes/${encodeURIComponent(box[1])}/exec`, {
        command: 'sh', args: ['-c', 'cd ~ 2>/dev/null; if command -v bash >/dev/null; then exec bash -l; else exec sh -l; fi'],
        tty: true, env: { TERM: 'xterm-256color' },
      });
      return json(201, { execId });
    }
    if (box && req.method === 'POST' && box[2] === '/run') {
      const { cmd } = await readJson(req);
      if (typeof cmd !== 'string' || !cmd.trim()) return json(400, { error: 'cmd is required' });
      res.writeHead(200, { 'content-type': 'application/x-ndjson', 'cache-control': 'no-store' });
      const send = (event) => res.write(`${JSON.stringify(event)}\n`);
      try { await run(decodeURIComponent(box[1]), cmd, send); } catch (e) { send({ kind: 'stderr', text: `${e.message}\n` }); }
      return res.end();
    }
    json(404, { error: 'not found' });
  } catch (e) {
    if (res.headersSent) return res.end();
    json(e.status && e.status < 600 ? e.status : 502, { error: e.message });
  }
});

// Terminal: relay the browser's WebSocket to the Box API's attach endpoint byte for byte, adding the key.
// The browser speaks the Box API frame protocol itself; this process never parses the stream.
const SAFE_ID = /^[A-Za-z0-9._-]+$/;
server.on('upgrade', (req, socket, head) => {
  const m = new URL(req.url, 'http://x').pathname.match(/^\/api\/boxes\/([^/]+)\/executions\/([^/]+)\/attach$/);
  const origin = req.headers.origin;
  // Refuse other web pages: a page on another site must not open shells through a dashboard on localhost.
  const sameOrigin = !origin || new URL(origin).host === req.headers.host;
  if (!m || !SAFE_ID.test(m[1]) || !SAFE_ID.test(m[2]) || !sameOrigin) return socket.destroy();
  const target = new URL(`${API}/v1${PREFIX}/boxes/${m[1]}/executions/${m[2]}/attach`);
  const secure = target.protocol === 'https:';
  const port = Number(target.port || (secure ? 443 : 80));
  const upstream = secure
    ? tls.connect({ host: target.hostname, port, servername: target.hostname })
    : net.connect(port, target.hostname);
  const close = () => { socket.destroy(); upstream.destroy(); };
  upstream.on('error', close); socket.on('error', close); upstream.on('close', close); socket.on('close', close);
  upstream.on(secure ? 'secureConnect' : 'connect', () => {
    const skip = new Set(['host', 'authorization', 'origin', 'cookie']);
    const lines = [`GET ${target.pathname} HTTP/1.1`, `Host: ${target.host}`];
    for (const [k, v] of Object.entries(auth())) lines.push(`${k}: ${v}`);
    for (let i = 0; i < req.rawHeaders.length; i += 2) {
      if (!skip.has(req.rawHeaders[i].toLowerCase())) lines.push(`${req.rawHeaders[i]}: ${req.rawHeaders[i + 1]}`);
    }
    upstream.write(`${lines.join('\r\n')}\r\n\r\n`);
    if (head.length) upstream.write(head);
    socket.pipe(upstream).pipe(socket);
  });
});

if (import.meta.main ?? process.argv[1] === new URL(import.meta.url).pathname) {
  server.listen(PORT, HOST, () => console.log(`dashboard on http://${HOST}:${PORT} -> ${API}`));
}
