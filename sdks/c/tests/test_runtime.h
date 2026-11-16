// Modified by ACRYL, 2026
#ifndef ACRYLMICROVM_C_TEST_RUNTIME_H
#define ACRYLMICROVM_C_TEST_RUNTIME_H

#include "acrylmicrovm.h"
#include <assert.h>
#include <ftw.h>
#include <stdio.h>
#include <sys/stat.h>
#include <unistd.h>

static const AcrylMicrovmImageRegistry TEST_REGISTRIES[] = {
    {.host = "docker.m.daocloud.io",
     .transport = AcrylMicrovmRegistryTransportHttps,
     .skip_verify = 0,
     .search = 1,
     .username = NULL,
     .password = NULL,
     .bearer_token = NULL},
    {.host = "docker.xuanyuan.me",
     .transport = AcrylMicrovmRegistryTransportHttps,
     .skip_verify = 0,
     .search = 1,
     .username = NULL,
     .password = NULL,
     .bearer_token = NULL},
    {.host = "docker.1ms.run",
     .transport = AcrylMicrovmRegistryTransportHttps,
     .skip_verify = 0,
     .search = 1,
     .username = NULL,
     .password = NULL,
     .bearer_token = NULL},
    {.host = "docker.io",
     .transport = AcrylMicrovmRegistryTransportHttps,
     .skip_verify = 0,
     .search = 1,
     .username = NULL,
     .password = NULL,
     .bearer_token = NULL}};

static const int TEST_REGISTRIES_COUNT = 4;

static int remove_tree_entry(const char *path, const struct stat *statbuf,
                             int typeflag, struct FTW *ftwbuf) {
  (void)statbuf;
  (void)typeflag;
  (void)ftwbuf;
  return remove(path);
}

static void reset_test_home(const char *temp_dir) {
  struct stat statbuf;
  if (lstat(temp_dir, &statbuf) != 0) {
    return;
  }

  assert(nftw(temp_dir, remove_tree_entry, 32, FTW_DEPTH | FTW_PHYS) == 0);
}

static CAcrylMicrovmRuntime *new_test_runtime(const char *temp_dir,
                                         CAcrylMicrovmError *error) {
  CAcrylMicrovmRuntime *runtime = NULL;
  AcrylMicrovmErrorCode code = acrylmicrovm_runtime_new(
      temp_dir, TEST_REGISTRIES, TEST_REGISTRIES_COUNT, &runtime, error);
  assert(code == Ok);
  assert(runtime != NULL);
  return runtime;
}

static CAcrylMicrovmOptions *new_alpine_options(CAcrylMicrovmError *error) {
  CAcrylMicrovmOptions *opts = NULL;
  AcrylMicrovmErrorCode code = acrylmicrovm_options_new("alpine:3.19", &opts, error);
  assert(code == Ok);
  assert(opts != NULL);
  acrylmicrovm_options_set_network_enabled(opts);
  acrylmicrovm_options_set_auto_remove(opts, 0);
  return opts;
}

static CBoxHandle *create_box_with_options(CAcrylMicrovmRuntime *runtime,
                                           CAcrylMicrovmOptions *opts,
                                           CAcrylMicrovmError *error) {
  CBoxHandle *box = NULL;
  AcrylMicrovmErrorCode code = acrylmicrovm_create_box(runtime, opts, &box, error);
  if (code != Ok) {
    printf("  ✗ Failed to create box: code=%d, message=%s\n", error->code,
           error->message ? error->message : "(null)");
    acrylmicrovm_error_free(error);
  }
  assert(code == Ok);
  assert(box != NULL);
  return box;
}

static CBoxHandle *create_test_box(CAcrylMicrovmRuntime *runtime,
                                   CAcrylMicrovmError *error) {
  CAcrylMicrovmOptions *opts = new_alpine_options(error);
  return create_box_with_options(runtime, opts, error);
}

static AcrylMicrovmErrorCode
test_execute_cmd(CBoxHandle *box, const AcrylMicrovmCommand *cmd,
                 void (*callback)(const char *, int, void *), void *user_data,
                 int *out_exit_code, CAcrylMicrovmError *error) {
  CExecutionHandle *execution = NULL;
  AcrylMicrovmErrorCode code =
      acrylmicrovm_execute(box, cmd, callback, user_data, &execution, error);
  if (code != Ok) {
    return code;
  }

  code = acrylmicrovm_execution_wait(execution, out_exit_code, error);
  acrylmicrovm_execution_free(execution);
  return code;
}

static AcrylMicrovmErrorCode
test_execute(CBoxHandle *box, const char *command, const char *const *args,
             int argc, void (*callback)(const char *, int, void *),
             void *user_data, int *out_exit_code, CAcrylMicrovmError *error) {
  AcrylMicrovmCommand cmd = {.command = command,
                        .args = args,
                        .argc = argc,
                        .env_pairs = NULL,
                        .env_count = 0,
                        .workdir = NULL,
                        .user = NULL,
                        .timeout_secs = 0.0,
                        .tty = 0};
  return test_execute_cmd(box, &cmd, callback, user_data, out_exit_code, error);
}

#endif
