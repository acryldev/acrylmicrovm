// Modified by ACRYL, 2026
//go:build acrylmicrovm_dev && acrylmicrovm_integration && acrylmicrovm_coverage

package archive_test

import (
	"fmt"
	"os"
	"testing"

	"github.com/acryldev/acrylmicrovm/sdks/go/integration/archive/internal/nativeprofile"
)

func TestMain(m *testing.M) {
	result := m.Run()
	if !nativeprofile.Flush() {
		fmt.Fprintln(os.Stderr, "failed to flush native LLVM coverage")
		result = 1
	}
	os.Exit(result)
}
