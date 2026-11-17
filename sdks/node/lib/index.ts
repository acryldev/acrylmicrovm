// Modified by ACRYL, 2026
/**
 * ACRYL microVMs Node.js SDK
 *
 * Embeddable VM runtime for secure, isolated code execution environments.
 *
 * @example
 * ```typescript
 * import { SimpleBox } from 'acrylmicrovm';
 *
 * const box = new SimpleBox({ image: 'alpine:latest' });
 * try {
 *   const result = await box.exec('echo', 'Hello from ACRYL microVMs!');
 *   console.log(result.stdout);
 * } finally {
 *   await box.stop();
 * }
 * ```
 *
 * @packageDocumentation
 */

import {
  getNativeModule,
  getJsAcrylMicrovm,
  getNativeAcrylMicrovmRestOptions,
} from "./native.js";
import { AcrylMicrovmRestOptions } from "./options.js";
import type {
  JsAcrylMicrovm as JsAcrylMicrovmInstance,
  JsAcrylMicrovmConstructor,
  JsOptions,
} from "./native-contracts.js";
export type {
  CheckpointHandle,
  CheckpointInfo,
  Domain,
  DomainHandle,
  DomainList,
  DomainTarget,
  ExecutionInfo,
  ImageDetail,
  ImageHandle,
  ImageInfo,
  ImagePullResult,
  ImageAliasEvent,
  ImageUsage,
  ImageVersion,
  NewRegistryCredential,
  RegistryCredential,
  RegistryHandle,
  VolumeHandle,
  VolumeInfo,
  JsImageRegistry,
  JsImageRegistryAuth,
  JsBoxInfo,
  JsNetworkInfo,
  JsOptions,
  JsPublishedPort,
} from "./native-contracts.js";

// The public `rest` takes the cross-SDK `AcrylMicrovmRestOptions` bag. The
// positional→bag adaptation now lives in the Rust binding (the native
// `rest` takes its own `AcrylMicrovmRestOptions` class — see
// `sdks/node/src/options.rs`, mirroring the Python SDK). This subclass
// only restates `rest` over the bag; `new`, `withDefaultConfig`, and
// `initDefault` inherit unchanged from the native class.
// The constructor type only replaces the native positional `rest` factory;
// instance methods and the other static factories remain native.
export type AcrylMicrovm = JsAcrylMicrovmInstance;

export type AcrylMicrovmConstructor = Omit<
  JsAcrylMicrovmConstructor,
  "rest" | "withDefaultConfig"
> & {
  new (options: JsOptions): AcrylMicrovm;
  withDefaultConfig(): AcrylMicrovm;
  rest(options: AcrylMicrovmRestOptions): AcrylMicrovm;
};

const nativeAcrylMicrovm = getJsAcrylMicrovm();
const NativeAcrylMicrovmRestOptions = getNativeAcrylMicrovmRestOptions();

class AcrylMicrovmWithBagRest extends (nativeAcrylMicrovm as unknown as {
  new (options: JsOptions): JsAcrylMicrovmInstance;
}) {
  static rest(options: AcrylMicrovmRestOptions): AcrylMicrovm {
    return nativeAcrylMicrovm.rest(
      new NativeAcrylMicrovmRestOptions(
        options.url,
        options.credential ?? null,
        options.pathPrefix ?? null,
      ),
    );
  }
}

export const JsAcrylMicrovm =
  AcrylMicrovmWithBagRest as unknown as AcrylMicrovmConstructor;
export { AcrylMicrovmRestOptions } from "./options.js";
export type * from "./agents.js";
export type { CopyOptions } from "./copy.js";

// Credential abstraction: structural `Credential` interface + concrete
// `ApiKeyCredential` class.
export {
  ApiKeyCredential,
  type Credential,
  type AccessToken,
} from "./credential.js";

// Export native module loader for advanced use cases
export { getNativeModule, getJsAcrylMicrovm };

// Re-export TypeScript wrappers
export {
  SimpleBox,
  BoxTunnel,
  TunnelForwarder,
  NetworkHandle,
  Executions,
  SshHandle,
  type NetworkSpec,
  type InboundNetworkSpec,
  type OutboundNetworkSpec,
  type SocketAddress,
  type TunnelOptions,
  type AdvancedBoxOptions,
  type ContainerCapabilities,
  type SimpleBoxOptions,
  type SecurityOptions,
  type Secret,
  type SshAccount,
  type SshCaConfig,
  type SshConfig,
  type SshStatus,
} from "./simplebox.js";
export { type ExecResult } from "./exec.js";
export {
  AcrylMicrovmError,
  ExecError,
  TimeoutError,
  ParseError,
  errorCode,
  type AcrylMicrovmErrorCode,
} from "./errors.js";
export * from "./constants.js";

// Specialized boxes
export { CodeBox, type CodeBoxOptions } from "./codebox.js";
export {
  BrowserBox,
  type BrowserBoxOptions,
  type BrowserType,
} from "./browserbox.js";
export {
  ComputerBox,
  type ComputerBoxOptions,
  type Screenshot,
} from "./computerbox.js";
export {
  InteractiveBox,
  type InteractiveBoxOptions,
} from "./interactivebox.js";
export { SkillBox, type SkillBoxOptions } from "./skillbox.js";
