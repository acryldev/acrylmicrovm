// Modified by ACRYL, 2026
//go:build acrylmicrovm_dev && !acrylmicrovm_coverage

package acrylmicrovm

// Development CGO directives — links against target/debug/libacrylmicrovm.a.
// Build the library first: make dev:go

/*
#cgo CFLAGS: -I${SRCDIR}/../c/include

#cgo darwin LDFLAGS: ${SRCDIR}/../../target/debug/libacrylmicrovm.a
#cgo darwin LDFLAGS: -framework CoreFoundation -framework Security -framework IOKit
#cgo darwin LDFLAGS: -framework Hypervisor -framework vmnet -lresolv

#cgo linux LDFLAGS: ${SRCDIR}/../../target/debug/libacrylmicrovm.a
#cgo linux LDFLAGS: -lresolv -lpthread -ldl -lrt -lm
*/
import "C"
