// Modified by ACRYL, 2026
#pragma once

#if __has_include("include/acrylmicrovm.h")
#include "include/acrylmicrovm.h"
#elif __has_include("../c/include/acrylmicrovm.h")
#include "../c/include/acrylmicrovm.h"
#else
#error "acrylmicrovm.h not found; run go run github.com/acryldev/acrylmicrovm/sdks/go/cmd/setup"
#endif
