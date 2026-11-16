// Modified by ACRYL, 2026
//go:build !acrylmicrovm_dev

package acrylmicrovm

// Prebuilt CGO directives — links against libacrylmicrovm.a downloaded by:
//   go run github.com/acryldev/acrylmicrovm/sdks/go/cmd/setup

/*
#cgo CFLAGS: -I${SRCDIR}/include

#cgo darwin LDFLAGS: ${SRCDIR}/libacrylmicrovm.a
#cgo darwin LDFLAGS: -framework CoreFoundation -framework Security -framework IOKit
#cgo darwin LDFLAGS: -framework Hypervisor -framework vmnet -lresolv

#cgo linux LDFLAGS: ${SRCDIR}/libacrylmicrovm.a
#cgo linux LDFLAGS: -lresolv -lpthread -ldl -lrt -lm
*/
import "C"
