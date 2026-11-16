// Modified by ACRYL, 2026
/* Curated base image for C e2e drivers run directly from /tmp.
 * Keep this fallback in step with apps/box-images/VERSION; run.sh and the
 * pytest wrappers normally provide ACRYLMICROVM_E2E_IMAGE.
 */
#ifndef ACRYLMICROVM_E2E_IMAGE_H
#define ACRYLMICROVM_E2E_IMAGE_H

#include <stdlib.h>
#include <string.h>

#define ACRYLMICROVM_E2E_DEFAULT_IMAGE "ghcr.io/acryldev/acrylmicrovm-agent-base:v0.1.0"

static inline const char* acrylmicrovm_e2e_default_image(void) {
    const char* image = getenv("ACRYLMICROVM_E2E_IMAGE");
    return (image && strlen(image)) ? image : ACRYLMICROVM_E2E_DEFAULT_IMAGE;
}

#endif /* ACRYLMICROVM_E2E_IMAGE_H */
