// Modified by ACRYL, 2026
/**
 * Credential abstraction for the ACRYL microVMs REST client.
 *
 * `Credential` is a structural interface: every credential class (today
 * `ApiKeyCredential`, a future `OAuthCredential`) structurally satisfies
 * it without a nominal `implements`. Callers type against `Credential`
 * and never change when new credential kinds are added.
 */

import type {
  ApiKeyCredential as NativeApiKeyCredential,
  JsAccessToken,
} from "./native-contracts.js";
import { getApiKeyCredential } from "./native.js";

/** A bearer token plus its expiry. */
export type AccessToken = JsAccessToken;

/** Anything that can produce a bearer token for `AcrylMicrovm.rest()`. */
export interface Credential {
  getToken(): AccessToken;
}

/**
 * Long-lived opaque API key credential. Concrete `Credential`.
 *
 * @example
 * ```ts
 * import { ACRYL microVMs, ApiKeyCredential } from '@acrylmicrovm/sdk';
 * const client = ACRYL microVMs.rest('http://localhost:8100',
 *                             new ApiKeyCredential('sk_...'));
 * ```
 */
export const ApiKeyCredential = getApiKeyCredential();

/** Instance type of {@link ApiKeyCredential}. */
export type ApiKeyCredential = NativeApiKeyCredential;
