#!/usr/bin/env node
// The `acrylmicrovm` command for npm, pnpm, yarn, bun and deno users: runs the prebuilt CLI binary.
// No "am I the main module" check: package managers start bin files through symlinks, shims or
// their own paths, so this file always runs and keeps the lookup in resolve.js.
import { spawn } from "node:child_process";
import { binaryPath } from "./resolve.js";

let bin;
try {
  bin = binaryPath();
} catch (e) {
  console.error(`acrylmicrovm: ${e.message}`);
  process.exit(1);
}
const child = spawn(bin, process.argv.slice(2), { stdio: "inherit" });
// Pass stop signals on, so `kill`, process managers and `docker stop` stop the binary too
// (Ctrl+C already reaches it through the terminal's process group).
for (const signal of ["SIGINT", "SIGTERM", "SIGHUP", "SIGQUIT"]) {
  process.on(signal, () => child.kill(signal));
}
child.on("error", (e) => {
  console.error(`acrylmicrovm: ${e.message}`);
  process.exit(1);
});
child.on("exit", (code, signal) => {
  if (signal) {
    process.removeAllListeners(signal);
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});
