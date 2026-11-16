// Modified by ACRYL, 2026
/**
 * ACRYL microVMs C SDK - REST runtime construction tests.
 *
 * Unit-level: REST runtime construction performs no I/O (the HTTP
 * connection is lazy, on first operation), so these run without a VM
 * or network. They exercise the opaque credential + options FFI
 * (acrylmicrovm_api_key_credential_new / acrylmicrovm_rest_options_new /
 * _set_credential / _set_prefix / acrylmicrovm_rest_runtime_new_with_options)
 * and that the resulting runtime frees via the shared
 * acrylmicrovm_runtime_free.
 */

#include "acrylmicrovm.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>

static void test_rest_options_null_out_rejected(void) {
  printf("\nTEST: REST - null out_options is rejected\n");
  CAcrylMicrovmError error = {0};

  AcrylMicrovmErrorCode code =
      acrylmicrovm_rest_options_new("https://api.example.com", NULL, &error);

  assert(code == InvalidArgument);
  acrylmicrovm_error_free(&error);
  printf("  ✓ null out_options -> InvalidArgument\n");
}

static void test_credential_null_out_rejected(void) {
  printf("\nTEST: REST - null out_credential is rejected\n");
  CAcrylMicrovmError error = {0};

  AcrylMicrovmErrorCode code =
      acrylmicrovm_api_key_credential_new("blk_live_example", NULL, &error);

  assert(code == InvalidArgument);
  acrylmicrovm_error_free(&error);
  printf("  ✓ null out_credential -> InvalidArgument\n");
}

static void test_runtime_null_options_rejected(void) {
  printf("\nTEST: REST - null options is rejected\n");
  CAcrylMicrovmRuntime *rt = NULL;
  CAcrylMicrovmError error = {0};

  AcrylMicrovmErrorCode code =
      acrylmicrovm_rest_runtime_new_with_options(NULL, &rt, &error);

  assert(code == InvalidArgument);
  assert(rt == NULL);
  acrylmicrovm_error_free(&error);
  printf("  ✓ null options -> InvalidArgument\n");
}

static void test_rest_url_only(void) {
  printf("\nTEST: REST - url only (unauthenticated) constructs\n");
  CAcrylMicrovmRestOptions *opts = NULL;
  CAcrylMicrovmRuntime *rt = NULL;
  CAcrylMicrovmError error = {0};

  AcrylMicrovmErrorCode code =
      acrylmicrovm_rest_options_new("http://localhost:8100", &opts, &error);
  assert(code == Ok);
  assert(opts != NULL);

  code = acrylmicrovm_rest_runtime_new_with_options(opts, &rt, &error);
  assert(code == Ok);
  assert(rt != NULL);

  acrylmicrovm_rest_options_free(opts);
  acrylmicrovm_runtime_free(rt);
  printf("  ✓ constructed + freed via acrylmicrovm_runtime_free\n");
}

static void test_rest_credential_and_prefix(void) {
  printf("\nTEST: REST - credential + prefix constructs\n");
  CAcrylMicrovmCredential *cred = NULL;
  CAcrylMicrovmRestOptions *opts = NULL;
  CAcrylMicrovmRuntime *rt = NULL;
  CAcrylMicrovmError error = {0};

  AcrylMicrovmErrorCode code =
      acrylmicrovm_api_key_credential_new("blk_live_example", &cred, &error);
  assert(code == Ok);
  assert(cred != NULL);

  code = acrylmicrovm_rest_options_new("https://api.example.com", &opts, &error);
  assert(code == Ok);
  assert(opts != NULL);

  acrylmicrovm_rest_options_set_credential(opts, cred);
  acrylmicrovm_rest_options_set_path_prefix(opts, "acme");

  code = acrylmicrovm_rest_runtime_new_with_options(opts, &rt, &error);
  assert(code == Ok);
  assert(rt != NULL);

  /* Options/credential are owned by the caller and freed independently;
   * the runtime keeps its own cloned reference. */
  acrylmicrovm_rest_options_free(opts);
  acrylmicrovm_credential_free(cred);
  acrylmicrovm_runtime_free(rt);
  printf("  ✓ constructed with credential + prefix + freed\n");
}

int main(void) {
  printf("=== AcrylMicrovm C SDK: REST runtime tests ===\n");
  test_rest_options_null_out_rejected();
  test_credential_null_out_rejected();
  test_runtime_null_options_rejected();
  test_rest_url_only();
  test_rest_credential_and_prefix();
  printf("\nAll REST runtime tests passed.\n");
  return 0;
}
