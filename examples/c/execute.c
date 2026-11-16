// Modified by ACRYL, 2026
/**
 * ACRYL microVMs C SDK - Command execution with streaming output.
 */

#include "example_common.h"
#include <stdio.h>

static void output_callback(const char *text, int is_stderr, void *user_data) {
  (void)user_data;
  FILE *stream = is_stderr ? stderr : stdout;
  fprintf(stream, "%s", text ? text : "");
}

int main(void) {
  printf("AcrylMicrovm C API Example\n");
  printf("Version: %s\n\n", acrylmicrovm_version());

  CAcrylMicrovmRuntime *runtime = create_runtime_or_exit();
  if (runtime == NULL) {
    return 1;
  }

  CBoxHandle *box = create_alpine_box_or_exit(runtime);
  if (box == NULL) {
    acrylmicrovm_runtime_free(runtime);
    return 1;
  }

  const char *const ls_args[] = {"-alrt", "/"};
  const char *const ip_args[] = {"addr"};
  int exit_code = 0;
  CAcrylMicrovmError error = {0};

  printf("Command 1: ls -alrt /\n---\n");
  AcrylMicrovmErrorCode code =
      execute_and_wait(runtime, box, "/bin/ls", ls_args, 2, output_callback,
                       NULL, &exit_code, &error);
  if (code != Ok) {
    print_error("ls", &error);
    acrylmicrovm_error_free(&error);
  }
  printf("\nExit code: %d\n\n", exit_code);

  printf("Command 2: ip addr\n---\n");
  error = (CAcrylMicrovmError){0};
  exit_code = 0;
  code = execute_and_wait(runtime, box, "ip", ip_args, 1, output_callback, NULL,
                          &exit_code, &error);
  if (code != Ok) {
    print_error("ip addr", &error);
    acrylmicrovm_error_free(&error);
  }
  printf("\nExit code: %d\n\n", exit_code);

  printf("Command 3: env\n---\n");
  error = (CAcrylMicrovmError){0};
  exit_code = 0;
  code = execute_and_wait(runtime, box, "/usr/bin/env", NULL, 0,
                          output_callback, NULL, &exit_code, &error);
  if (code != Ok) {
    print_error("env", &error);
    acrylmicrovm_error_free(&error);
  }
  printf("\nExit code: %d\n", exit_code);

  char *id = acrylmicrovm_box_id(box);
  example_remove_box(runtime, id, 1, &error);
  acrylmicrovm_box_free(box);
  acrylmicrovm_free_string(id);
  acrylmicrovm_runtime_free(runtime);
  return 0;
}
