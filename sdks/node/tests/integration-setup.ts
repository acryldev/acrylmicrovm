// Modified by ACRYL, 2026
/**
 * Vitest setup file for integration tests.
 *
 * Initializes the default AcrylMicrovmRuntime with a unique temp directory
 * to avoid lock contention with any running ACRYL microVMs process (e.g.,
 * acrylmicrovm-mcp). Follows the same pattern as test-utils/PerTestBoxHome.
 */

import { mkdtempSync, rmSync } from "node:fs";
import { afterAll } from "vitest";
import { getJsAcrylMicrovm } from "../lib/native.js";
import { testRegistries } from "./test-registries.js";

// Use /tmp/ (not os.tmpdir()) to keep Unix socket paths under macOS
// 104-char SUN_LEN limit. Same pattern as test-utils/PerTestBoxHome.
const testHome = mkdtempSync("/tmp/acrylmicrovm-test-node-");

const AcrylMicrovm = getJsAcrylMicrovm();
AcrylMicrovm.initDefault({ homeDir: testHome, imageRegistries: testRegistries() });

afterAll(async () => {
  try {
    // Get the default runtime and shut it down
    const runtime = AcrylMicrovm.withDefaultConfig();
    await runtime.shutdown();
  } catch {
    // Ignore shutdown errors during cleanup
  }
  rmSync(testHome, { recursive: true, force: true });
});
