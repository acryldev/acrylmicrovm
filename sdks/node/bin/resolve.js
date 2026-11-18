// Finds the prebuilt acrylmicrovm CLI binary in this platform's package (acrylmicrovm-<platform>),
// the same binary `cargo install acrylmicrovm-cli` builds. Used by bin/acrylmicrovm.js.
import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";

const PLATFORMS = {
  "darwin-arm64": "acrylmicrovm-darwin-arm64",
  "linux-x64": "acrylmicrovm-linux-x64-gnu",
  "linux-arm64": "acrylmicrovm-linux-arm64-gnu",
};

export function binaryPath(
  platform = process.platform,
  arch = process.arch,
  require = createRequire(import.meta.url),
) {
  if (process.env.ACRYLMICROVM_BINARY) return process.env.ACRYLMICROVM_BINARY;
  const pkg = PLATFORMS[`${platform}-${arch}`];
  if (!pkg) {
    throw new Error(
      `acrylmicrovm has no prebuilt binary for ${platform}-${arch}. Supported: ${Object.keys(PLATFORMS).join(", ")}.`,
    );
  }
  let dir;
  try {
    dir = dirname(require.resolve(`${pkg}/package.json`));
  } catch {
    throw new Error(
      `The package ${pkg} is missing. Reinstall acrylmicrovm without --no-optional / --omit=optional.`,
    );
  }
  const bin = join(dir, "acrylmicrovm");
  if (!existsSync(bin))
    throw new Error(`${pkg} has no acrylmicrovm binary at ${bin}.`);
  return bin;
}
