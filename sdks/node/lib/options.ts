// Modified by ACRYL, 2026
/**
 * REST options for the ACRYL microVMs client.
 *
 * `AcrylMicrovmRestOptions` is the single options bag passed to
 * `JsAcrylMicrovm.rest(...)`. The name and shape are intentionally identical
 * across every SDK (C/Go/Node/Python) for cross-SDK parity.
 *
 * @example
 * ```ts
 * import { JsAcrylMicrovm, AcrylMicrovmRestOptions, ApiKeyCredential } from 'acrylmicrovm';
 *
 * const rt = JsAcrylMicrovm.rest(new AcrylMicrovmRestOptions({
 *   url: 'https://api.example.com',
 *   credential: new ApiKeyCredential('blk_live_...'),
 *   pathPrefix: 'acme',
 * }));
 * ```
 */

import type { Credential } from "./credential.js";

/** Configuration for connecting to a remote ACRYL microVMs REST API server. */
export class AcrylMicrovmRestOptions {
  /** REST API base URL (e.g. `https://api.example.com`). */
  readonly url: string;

  /** Bearer credential. Omit for an unauthenticated runtime. */
  readonly credential?: Credential;

  /**
   * Routing-slot value substituted into the `{prefix}` URL
   * segment. Opaque — the server tells the client what to put here
   * (via `Principal.path_prefix` from `GET /v1/me`). Omit / empty /
   * null → URL skips the segment (`/v1/boxes/…`) — the single-
   * tenant deployment shape.
   */
  readonly pathPrefix?: string;

  constructor(options: {
    url: string;
    credential?: Credential;
    pathPrefix?: string;
  }) {
    this.url = options.url;
    this.credential = options.credential;
    this.pathPrefix = options.pathPrefix;
  }
}
