// Modified by ACRYL, 2026
/**
 * Runtime shutdown example.
 */

#include "example_common.h"
#include <stdio.h>

int main(void) {
  printf("=== Runtime Shutdown Example ===\n\n");

  CAcrylMicrovmRuntime *runtime = create_runtime_or_exit();
  if (runtime == NULL) {
    return 1;
  }

  CAcrylMicrovmError error = {0};
  CBoxHandle *boxes[3] = {0};
  for (int i = 0; i < 3; i++) {
    boxes[i] = create_alpine_box_or_exit(runtime);
    if (boxes[i] == NULL) {
      continue;
    }
    char *id = acrylmicrovm_box_id(boxes[i]);
    printf("Created box %d: %s\n", i + 1, id);
    acrylmicrovm_free_string(id);
  }

  CRuntimeMetrics metrics = {0};
  AcrylMicrovmErrorCode code =
      example_get_runtime_metrics(runtime, &metrics, &error);
  if (code == Ok) {
    printf("\nBefore shutdown: running=%d created=%d\n",
           metrics.num_running_boxes, metrics.boxes_created_total);
  } else {
    acrylmicrovm_error_free(&error);
  }

  printf("\nShutting down all boxes (5 second timeout)...\n");
  error = (CAcrylMicrovmError){0};
  code = example_shutdown_runtime(runtime, 5, &error);
  if (code != Ok) {
    acrylmicrovm_error_free(&error);
  } else {
    printf("Shutdown complete\n");
  }

  printf("\nTrying to create a new box after shutdown...\n");
  CBoxHandle *new_box = create_alpine_box_or_exit(runtime);
  if (new_box != NULL) {
    printf("unexpected success\n");
    acrylmicrovm_box_free(new_box);
  } else {
    printf("Expected box creation failure after shutdown\n");
  }

  for (int i = 0; i < 3; i++) {
    acrylmicrovm_box_free(boxes[i]);
  }

  acrylmicrovm_runtime_free(runtime);
  return 0;
}
