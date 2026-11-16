// Modified by ACRYL, 2026
/**
 * ACRYL microVMs C SDK - Example 1: Box lifecycle.
 */

#include "example_common.h"
#include <stdio.h>

int main(void) {
  printf("=== AcrylMicrovm Example: Box Lifecycle ===\n\n");

  CAcrylMicrovmRuntime *runtime = create_runtime_or_exit();
  if (runtime == NULL) {
    return 1;
  }

  printf("1. Creating box...\n");
  CBoxHandle *box = create_alpine_box_or_exit(runtime);
  if (box == NULL) {
    acrylmicrovm_runtime_free(runtime);
    return 1;
  }

  CAcrylMicrovmError error = {0};
  char *box_id = acrylmicrovm_box_id(box);
  printf("   Box ID: %s\n\n", box_id);

  printf("2. Executing hostname...\n");
  int exit_code = 0;
  AcrylMicrovmErrorCode code = execute_and_wait(runtime, box, "/bin/hostname", NULL,
                                           0, NULL, NULL, &exit_code, &error);
  if (code != Ok) {
    print_error("hostname", &error);
    acrylmicrovm_error_free(&error);
  }
  printf("   Exit code: %d\n\n", exit_code);

  printf("3. Stopping box...\n");
  code = example_stop_box(runtime, box, &error);
  if (code != Ok) {
    acrylmicrovm_error_free(&error);
  }
  printf("   Stopped\n\n");

  printf("4. Reattaching and restarting...\n");
  CBoxHandle *box2 = NULL;
  code = example_get_box(runtime, box_id, &box2, &error);
  if (code != Ok) {
    acrylmicrovm_error_free(&error);
    acrylmicrovm_free_string(box_id);
    acrylmicrovm_runtime_free(runtime);
    return 1;
  }
  code = example_start_box(runtime, box2, &error);
  if (code != Ok) {
    acrylmicrovm_error_free(&error);
  }
  printf("   Restarted\n\n");

  printf("5. Executing uname -a...\n");
  const char *const uname_args[] = {"-a"};
  exit_code = 0;
  code = execute_and_wait(runtime, box2, "/bin/uname", uname_args, 1, NULL,
                          NULL, &exit_code, &error);
  if (code != Ok) {
    print_error("uname", &error);
    acrylmicrovm_error_free(&error);
  }
  printf("   Exit code: %d\n\n", exit_code);

  printf("6. Removing box...\n");
  code = example_remove_box(runtime, box_id, 1, &error);
  if (code != Ok) {
    acrylmicrovm_error_free(&error);
  }

  acrylmicrovm_box_free(box);
  acrylmicrovm_box_free(box2);
  acrylmicrovm_free_string(box_id);
  acrylmicrovm_runtime_free(runtime);

  printf("\n=== Lifecycle Demo Complete ===\n");
  return 0;
}
