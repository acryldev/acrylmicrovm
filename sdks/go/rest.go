// Modified by ACRYL, 2026
package acrylmicrovm

/*
#include "bridge.h"
#include <stdlib.h>
*/
import "C"
import (
	"fmt"
	"unsafe"
)

// AcrylMicrovmRestOptions configures a REST-backed runtime created by
// NewRest. The name is intentionally identical across every SDK
// (C/Go/Node/Python) for cross-SDK parity; the acrylmicrovm.AcrylMicrovmRestOptions
// package stutter is accepted as a deliberate trade-off.
//
//nolint:revive
type AcrylMicrovmRestOptions struct {
	// URL is the REST API base URL (required, e.g.
	// "https://api.example.com").
	URL string

	// Credential authenticates requests. Nil = unauthenticated. Today
	// only *ApiKeyCredential is supported.
	Credential Credential

	// PathPrefix is the routing-slot value substituted into the
	// {prefix} URL segment on box-scoped requests. Opaque — the
	// server tells the client what to use here via
	// Principal.path_prefix from GET /v1/me. Empty → URL skips the
	// segment entirely (/v1/boxes/...) — the single-tenant
	// deployment shape.
	PathPrefix string
}

// NewRest creates a runtime that connects to a remote ACRYL microVMs REST
// server at opts.URL instead of running boxes in-process.
//
// The returned *Runtime is identical to one from NewRuntime — every
// box, exec, copy, image, and metrics operation works against it
// unchanged; REST is just an alternative backend. The HTTP connection
// is lazy (established on the first operation), so NewRest itself does
// no network I/O.
//
//	rt, err := acrylmicrovm.NewRest(acrylmicrovm.AcrylMicrovmRestOptions{
//	        URL:        "https://api.example.com",
//	        Credential: acrylmicrovm.NewApiKeyCredential("blk_live_..."),
//	})
//	if err != nil { return err }
//	defer rt.Close()
func NewRest(opts AcrylMicrovmRestOptions) (*Runtime, error) {
	cURL := toCString(opts.URL)
	defer C.free(unsafe.Pointer(cURL))

	var cerr C.CAcrylMicrovmError
	var cOpts *C.CAcrylMicrovmRestOptions
	if code := C.acrylmicrovm_rest_options_new(cURL, &cOpts, &cerr); code != C.Ok {
		return nil, freeError(&cerr)
	}
	defer C.acrylmicrovm_rest_options_free(cOpts)

	if opts.Credential != nil {
		apiKey, ok := opts.Credential.(*ApiKeyCredential)
		if !ok {
			return nil, fmt.Errorf(
				"acrylmicrovm: unsupported Credential type %T (only *ApiKeyCredential is supported)",
				opts.Credential,
			)
		}
		cKey := toCString(apiKey.key)
		defer C.free(unsafe.Pointer(cKey))

		var cCred *C.CAcrylMicrovmCredential
		if code := C.acrylmicrovm_api_key_credential_new(cKey, &cCred, &cerr); code != C.Ok {
			return nil, freeError(&cerr)
		}
		defer C.acrylmicrovm_credential_free(cCred)
		C.acrylmicrovm_rest_options_set_credential(cOpts, cCred)
	}

	if opts.PathPrefix != "" {
		cPathPrefix := toCString(opts.PathPrefix)
		defer C.free(unsafe.Pointer(cPathPrefix))
		C.acrylmicrovm_rest_options_set_path_prefix(cOpts, cPathPrefix)
	}

	var handle *C.CAcrylMicrovmRuntime
	if code := C.acrylmicrovm_rest_runtime_new_with_options(cOpts, &handle, &cerr); code != C.Ok {
		return nil, freeError(&cerr)
	}

	return &Runtime{
		handle:  handle,
		closing: make(chan struct{}),
	}, nil
}
