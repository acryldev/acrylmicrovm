// Modified by ACRYL, 2026
// Curated base image for Go e2e drivers run directly from /tmp.
// Keep this fallback in step with apps/box-images/VERSION; run.sh and the
// pytest wrappers normally provide ACRYLMICROVM_E2E_IMAGE.
package main

import "os"

const defaultImageRef = "ghcr.io/acryldev/acrylmicrovm-agent-base:v0.1.0"

func defaultImage() string {
	if image := os.Getenv("ACRYLMICROVM_E2E_IMAGE"); image != "" {
		return image
	}
	return defaultImageRef
}
