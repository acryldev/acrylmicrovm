// Modified by ACRYL, 2026
/**
 * ACRYL microVMs C SDK - Example 5: Performance metrics.
 */

#include "example_common.h"
#include <stdio.h>
#include <unistd.h>

static void print_runtime_metrics(CAcrylMicrovmRuntime *runtime) {
  CAcrylMicrovmError error = {0};
  CRuntimeMetrics metrics = {0};
  AcrylMicrovmErrorCode code =
      example_get_runtime_metrics(runtime, &metrics, &error);
  if (code != Ok) {
    acrylmicrovm_error_free(&error);
    return;
  }
  printf("created=%d failed=%d running=%d commands=%d exec_errors=%d\n",
         metrics.boxes_created_total, metrics.boxes_failed_total,
         metrics.num_running_boxes, metrics.total_commands_executed,
         metrics.total_exec_errors);
}

static void print_box_metrics(CAcrylMicrovmRuntime *runtime, CBoxHandle *box) {
  CAcrylMicrovmError error = {0};
  CBoxMetrics metrics = {0};
  AcrylMicrovmErrorCode code =
      example_get_box_metrics(runtime, box, &metrics, &error);
  if (code != Ok) {
    acrylmicrovm_error_free(&error);
    return;
  }
  printf("commands=%d exec_errors=%d memory=%lld cpu=%.2f\n",
         metrics.commands_executed, metrics.exec_errors,
         (long long)metrics.memory_bytes, metrics.cpu_percent);
}

int main(void) {
  printf("=== AcrylMicrovm Example: Performance Metrics ===\n\n");

  CAcrylMicrovmRuntime *runtime = create_runtime_or_exit();
  if (runtime == NULL) {
    return 1;
  }

  printf("Initial runtime metrics:\n");
  print_runtime_metrics(runtime);
  printf("\n");

  CBoxHandle *box = create_alpine_box_or_exit(runtime);
  if (box == NULL) {
    acrylmicrovm_runtime_free(runtime);
    return 1;
  }

  CAcrylMicrovmError error = {0};
  const char *const echo_args[] = {"test"};
  int exit_code = 0;
  for (int i = 0; i < 5; i++) {
    execute_and_wait(runtime, box, "/bin/echo", echo_args, 1, NULL, NULL,
                     &exit_code, &error);
  }

  printf("Runtime metrics after commands:\n");
  print_runtime_metrics(runtime);
  printf("\nBox metrics:\n");
  print_box_metrics(runtime, box);

  printf("\nMetric samples:\n");
  const char *const uname_args[] = {"-a"};
  for (int i = 0; i < 3; i++) {
    execute_and_wait(runtime, box, "/bin/uname", uname_args, 1, NULL, NULL,
                     &exit_code, &error);
    printf("sample %d: ", i + 1);
    print_box_metrics(runtime, box);
    sleep(1);
  }

  char *id = acrylmicrovm_box_id(box);
  example_remove_box(runtime, id, 1, &error);
  acrylmicrovm_box_free(box);
  acrylmicrovm_free_string(id);
  acrylmicrovm_runtime_free(runtime);

  printf("\n=== Metrics Demo Complete ===\n");
  return 0;
}
