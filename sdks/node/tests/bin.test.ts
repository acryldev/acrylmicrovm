import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
// @ts-expect-error plain ESM script without types
import { binaryPath } from "../bin/acrylmicrovm.js";

function fakePackage(withBinary: boolean) {
  const dir = mkdtempSync(join(tmpdir(), "amv-bin-"));
  mkdirSync(dir, { recursive: true });
  writeFileSync(join(dir, "package.json"), "{}");
  if (withBinary) writeFileSync(join(dir, "acrylmicrovm"), "");
  const require = {
    resolve: (id: string) => {
      if (id !== "acrylmicrovm-linux-x64-gnu/package.json")
        throw new Error("not found");
      return join(dir, "package.json");
    },
  };
  return { dir, require };
}

describe("npm bin: binaryPath", () => {
  it("finds the binary in this platform's package", () => {
    const { dir, require } = fakePackage(true);
    expect(binaryPath("linux", "x64", require)).toBe(join(dir, "acrylmicrovm"));
  });

  it("names the supported platforms for an unsupported one", () => {
    expect(() => binaryPath("win32", "x64", fakePackage(true).require)).toThrow(
      /darwin-arm64, linux-x64, linux-arm64/,
    );
  });

  it("explains a missing platform package", () => {
    expect(() =>
      binaryPath("linux", "arm64", fakePackage(true).require),
    ).toThrow(/acrylmicrovm-linux-arm64-gnu is missing/);
  });

  it("reports a platform package without the binary", () => {
    expect(() =>
      binaryPath("linux", "x64", fakePackage(false).require),
    ).toThrow(/has no acrylmicrovm binary/);
  });
});
