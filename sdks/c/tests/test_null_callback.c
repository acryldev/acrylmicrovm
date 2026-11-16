// Modified by ACRYL, 2026
/**
 * ACRYL microVMs C SDK — NULL-callback rejection (Codex finding #3 reproducer)
 *
 * The async FFI entrypoints take callback parameters that, on the Rust side,
 * are typed as `Option<extern "C" fn(...)>`. Passing NULL from C must yield
 * a synchronous AcrylMicrovmErrorCode_InvalidArgument, never UB.
 *
 * BEFORE FIX: typedefs were bare `extern "C" fn(...)` (non-null by Rust ABI).
 * Passing NULL invoked UB the moment Rust tried to load/store the parameter,
 * and dispatch eventually called a NULL fn pointer → segfault.
 * AFTER FIX: typedefs are `Option<extern "C" fn(...)>`; cbindgen still emits
 * `void (*cb)(...)` in the C header so this file compiles unchanged. Rust
 * receives `None` via NPO and the entrypoint returns InvalidArgument.
 */

#include "acrylmicrovm.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const char *temp_home_dir(void) {
  /* Fixed path is fine for a single-process test; concurrent test runs are
   * not part of the supported workflow for this suite. */
  return "/tmp/acrylmicrovm-c-null-cb";
}

static CAcrylMicrovmRuntime *new_runtime(void) {
  CAcrylMicrovmRuntime *rt = NULL;
  CAcrylMicrovmError err = (CAcrylMicrovmError){0};
  AcrylMicrovmErrorCode code =
      acrylmicrovm_runtime_new(temp_home_dir(), NULL, 0, &rt, &err);
  if (code != Ok) {
    printf("runtime_new failed: code=%d msg=%s\n", code,
           err.message ? err.message : "<none>");
    acrylmicrovm_error_free(&err);
    exit(1);
  }
  return rt;
}

static void assert_null_cb_rejected(AcrylMicrovmErrorCode code, CAcrylMicrovmError *err,
                                    const char *which) {
  if (code != InvalidArgument) {
    printf("%s: expected InvalidArgument (5), got code=%d msg=%s\n", which,
           code, err->message ? err->message : "<none>");
    exit(1);
  }
  if (err->message == NULL || strstr(err->message, "cb") == NULL) {
    printf("%s: error message should mention 'cb' but was: %s\n", which,
           err->message ? err->message : "<none>");
    exit(1);
  }
  printf("  [ok] %s -> InvalidArgument: %s\n", which, err->message);
  acrylmicrovm_error_free(err);
}

static void test_create_box_null_cb(CAcrylMicrovmRuntime *rt) {
  CAcrylMicrovmOptions *opts = NULL;
  CAcrylMicrovmError err = (CAcrylMicrovmError){0};
  AcrylMicrovmErrorCode oc = acrylmicrovm_options_new("alpine:latest", &opts, &err);
  assert(oc == Ok);

  const char *hosts[] = {"example.com"};
  acrylmicrovm_options_add_secret(opts, "legacy", "value", "${LEGACY}", hosts, 1);
  const char *header_names[] = {"X-Api-Key"};
  const char *header_templates[] = {"{value}"};
  acrylmicrovm_options_add_secret_with_headers(opts, "custom", "value", "${CUSTOM}",
                                          hosts, 1, header_names,
                                          header_templates, 1);

  err = (CAcrylMicrovmError){0};
  AcrylMicrovmErrorCode code = acrylmicrovm_create_box(rt, opts, NULL, NULL, &err);
  assert_null_cb_rejected(code, &err, "acrylmicrovm_create_box(cb=NULL)");

  /* opts is only consumed by acrylmicrovm_create_box on success — free here. */
  acrylmicrovm_options_free(opts);
}

static void test_runtime_metrics_null_cb(CAcrylMicrovmRuntime *rt) {
  CAcrylMicrovmError err = (CAcrylMicrovmError){0};
  AcrylMicrovmErrorCode code = acrylmicrovm_runtime_metrics(rt, NULL, NULL, &err);
  assert_null_cb_rejected(code, &err, "acrylmicrovm_runtime_metrics(cb=NULL)");
}

static void test_list_info_null_cb(CAcrylMicrovmRuntime *rt) {
  CAcrylMicrovmError err = (CAcrylMicrovmError){0};
  AcrylMicrovmErrorCode code = acrylmicrovm_list_info(rt, NULL, NULL, &err);
  assert_null_cb_rejected(code, &err, "acrylmicrovm_list_info(cb=NULL)");
}

static void test_runtime_shutdown_null_cb(CAcrylMicrovmRuntime *rt) {
  CAcrylMicrovmError err = (CAcrylMicrovmError){0};
  AcrylMicrovmErrorCode code = acrylmicrovm_runtime_shutdown(rt, 0, NULL, NULL, &err);
  assert_null_cb_rejected(code, &err, "acrylmicrovm_runtime_shutdown(cb=NULL)");
}

int main(void) {
  printf("\n═══════════════════════════════════════\n");
  printf("  AcrylMicrovm C SDK - NULL callback rejection\n");
  printf("═══════════════════════════════════════\n\n");

  CAcrylMicrovmRuntime *rt = new_runtime();

  test_create_box_null_cb(rt);
  test_runtime_metrics_null_cb(rt);
  test_list_info_null_cb(rt);
  test_runtime_shutdown_null_cb(rt);

  acrylmicrovm_runtime_free(rt);

  printf("\n═══════════════════════════════════════\n");
  printf("  ✅ ALL NULL-CB TESTS PASSED (4 tests)\n");
  printf("═══════════════════════════════════════\n");
  return 0;
}
