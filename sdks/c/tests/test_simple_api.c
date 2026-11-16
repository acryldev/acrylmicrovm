// Modified by ACRYL, 2026
/**
 * ACRYL microVMs C SDK - Simple API Tests
 *
 * Tests the simple convenience API with automatic cleanup.
 */

#include "acrylmicrovm.h"
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void test_simple_create() {
  printf("\nTEST: Simple API - create box\n");

  CAcrylMicrovmSimple *box;
  CAcrylMicrovmError error = {0};

  AcrylMicrovmErrorCode code = acrylmicrovm_simple_new("alpine:3.19", // image
                                             2,             // cpus
                                             512,           // memory_mib
                                             &box, &error);

  assert(code == Ok);
  assert(box != NULL);
  printf("  ✓ Box created with simple API\n");

  acrylmicrovm_simple_free(box);
  printf("  ✓ Box auto-cleaned up\n");
}

void test_simple_default_resources() {
  printf("\nTEST: Simple API - default resources\n");

  CAcrylMicrovmSimple *box;
  CAcrylMicrovmError error = {0};

  // Use 0 for default cpus and memory
  AcrylMicrovmErrorCode code = acrylmicrovm_simple_new("alpine:3.19",
                                             0, // cpus = default
                                             0, // memory_mib = default
                                             &box, &error);

  assert(code == Ok);
  assert(box != NULL);
  printf("  ✓ Box created with default resources\n");

  acrylmicrovm_simple_free(box);
}

void test_simple_run_command() {
  printf("\nTEST: Simple API - run command\n");

  CAcrylMicrovmSimple *box;
  CAcrylMicrovmError error = {0};

  AcrylMicrovmErrorCode code = acrylmicrovm_simple_new("alpine:3.19", 0, 0, &box, &error);
  assert(code == Ok);

  // Run a simple command
  const char *args[] = {"hello", NULL};
  CAcrylMicrovmExecResult *result;

  code = acrylmicrovm_simple_run(box, "/bin/echo", args, 1, &result, &error);
  assert(code == Ok);
  assert(result != NULL);
  assert(result->exit_code == 0);
  assert(result->stdout_text != NULL);
  printf("  ✓ Command executed: exit_code=%d\n", result->exit_code);
  printf("  ✓ Output: %s\n", result->stdout_text);

  acrylmicrovm_result_free(result);
  acrylmicrovm_simple_free(box);
}

void test_simple_run_no_args() {
  printf("\nTEST: Simple API - run command without args\n");

  CAcrylMicrovmSimple *box;
  CAcrylMicrovmError error = {0};

  AcrylMicrovmErrorCode code = acrylmicrovm_simple_new("alpine:3.19", 0, 0, &box, &error);
  assert(code == Ok);

  // Run command with no args (NULL, 0)
  CAcrylMicrovmExecResult *result;
  code = acrylmicrovm_simple_run(box, "/bin/pwd", NULL, 0, &result, &error);

  assert(code == Ok);
  assert(result->exit_code == 0);
  printf("  ✓ Command executed without args\n");

  acrylmicrovm_result_free(result);
  acrylmicrovm_simple_free(box);
}

void test_simple_run_failure() {
  printf("\nTEST: Simple API - run failing command\n");

  CAcrylMicrovmSimple *box;
  CAcrylMicrovmError error = {0};

  AcrylMicrovmErrorCode code = acrylmicrovm_simple_new("alpine:3.19", 0, 0, &box, &error);
  assert(code == Ok);

  // Run command that will fail
  const char *args[] = {"/nonexistent", NULL};
  CAcrylMicrovmExecResult *result;

  code = acrylmicrovm_simple_run(box, "/bin/ls", args, 1, &result, &error);
  assert(code == Ok);             // API call succeeds
  assert(result->exit_code != 0); // But command fails
  printf("  ✓ Command failed as expected: exit_code=%d\n", result->exit_code);

  // Check stderr contains error message
  if (result->stderr_text && strlen(result->stderr_text) > 0) {
    printf("  ✓ Stderr captured: %s\n", result->stderr_text);
  }

  acrylmicrovm_result_free(result);
  acrylmicrovm_simple_free(box);
}

void test_simple_multiple_commands() {
  printf("\nTEST: Simple API - multiple commands\n");

  CAcrylMicrovmSimple *box;
  CAcrylMicrovmError error = {0};

  AcrylMicrovmErrorCode code = acrylmicrovm_simple_new("alpine:3.19", 0, 0, &box, &error);
  assert(code == Ok);

  CAcrylMicrovmExecResult *result;

  // Command 1
  const char *args1[] = {"test1", NULL};
  code = acrylmicrovm_simple_run(box, "/bin/echo", args1, 1, &result, &error);
  assert(code == Ok);
  assert(result->exit_code == 0);
  acrylmicrovm_result_free(result);

  // Command 2
  const char *args2[] = {"test2", NULL};
  code = acrylmicrovm_simple_run(box, "/bin/echo", args2, 1, &result, &error);
  assert(code == Ok);
  assert(result->exit_code == 0);
  acrylmicrovm_result_free(result);

  // Command 3
  const char *args3[] = {"test3", NULL};
  code = acrylmicrovm_simple_run(box, "/bin/echo", args3, 1, &result, &error);
  assert(code == Ok);
  assert(result->exit_code == 0);
  acrylmicrovm_result_free(result);

  printf("  ✓ Ran 3 commands successfully\n");

  acrylmicrovm_simple_free(box);
}

void test_simple_result_cleanup() {
  printf("\nTEST: Simple API - result cleanup\n");

  CAcrylMicrovmSimple *box;
  CAcrylMicrovmError error = {0};

  AcrylMicrovmErrorCode code = acrylmicrovm_simple_new("alpine:3.19", 0, 0, &box, &error);
  assert(code == Ok);

  CAcrylMicrovmExecResult *result;
  const char *args[] = {"hello", NULL};
  code = acrylmicrovm_simple_run(box, "/bin/echo", args, 1, &result, &error);
  assert(code == Ok);

  // Free result multiple times should be safe
  acrylmicrovm_result_free(result);
  acrylmicrovm_result_free(NULL); // Should not crash

  printf("  ✓ Result cleanup is safe\n");

  acrylmicrovm_simple_free(box);
}

void test_simple_null_safety() {
  printf("\nTEST: Simple API - NULL safety\n");

  // Free NULL box (should not crash)
  acrylmicrovm_simple_free(NULL);
  printf("  ✓ acrylmicrovm_simple_free(NULL) is safe\n");

  // Free NULL result (should not crash)
  acrylmicrovm_result_free(NULL);
  printf("  ✓ acrylmicrovm_result_free(NULL) is safe\n");
}

void test_simple_auto_cleanup() {
  printf("\nTEST: Simple API - auto cleanup\n");

  CAcrylMicrovmSimple *box;
  CAcrylMicrovmError error = {0};

  AcrylMicrovmErrorCode code = acrylmicrovm_simple_new("alpine:3.19", 0, 0, &box, &error);
  assert(code == Ok);

  // Run a command
  CAcrylMicrovmExecResult *result;
  const char *args[] = {"test", NULL};
  code = acrylmicrovm_simple_run(box, "/bin/echo", args, 1, &result, &error);
  assert(code == Ok);
  acrylmicrovm_result_free(result);

  // Just free - should auto-stop and remove
  acrylmicrovm_simple_free(box);
  printf("  ✓ Box auto-stopped and removed on free\n");
}

int main() {
  printf("═══════════════════════════════════════\n");
  printf("  AcrylMicrovm C SDK - Simple API Tests\n");
  printf("═══════════════════════════════════════\n");

  test_simple_create();
  test_simple_default_resources();
  test_simple_run_command();
  test_simple_run_no_args();
  test_simple_run_failure();
  test_simple_multiple_commands();
  test_simple_result_cleanup();
  test_simple_null_safety();
  test_simple_auto_cleanup();

  printf("\n═══════════════════════════════════════\n");
  printf("  ✅ ALL TESTS PASSED (%d tests)\n", 9);
  printf("═══════════════════════════════════════\n");

  return 0;
}
