// Modified by ACRYL, 2026
/**
 * ACRYL microVMs C SDK — managed-volume FFI surface.
 *
 * Covers the three ABI breaks recorded in the README migration guide, from a
 * real C consumer rather than from Rust:
 *
 *   1. `acrylmicrovm_options_add_volume` split into `acrylmicrovm_options_add_bind_mount`
 *      (a path on this machine) and `acrylmicrovm_options_add_managed_volume`
 *      (a server-side volume, by id or name).
 *   2. `acrylmicrovm_volume_create` gained a `const char *name` second parameter.
 *   3. `CVolumeInfo` gained a `name` member, changing the struct layout.
 *
 * Compiling this file is itself most of the test: a consumer built against the
 * old header would not link. The runtime assertions cover the argument
 * validation that a signature change alone cannot prove.
 */

#include "acrylmicrovm.h"
#include <assert.h>
#include <stddef.h>
#include <stdio.h>

static void on_volume_created(struct CVolumeInfo *info, CAcrylMicrovmError *error,
                              void *user_data) {
  (void)info;
  (void)error;
  (void)user_data;
}

/* (3) The struct carries a name, and it sits between id and created_at. */
static void test_volume_info_layout(void) {
  struct CVolumeInfo info = {0};

  info.id = NULL;
  info.name = NULL;
  info.created_at = NULL;

  assert(offsetof(struct CVolumeInfo, id) < offsetof(struct CVolumeInfo, name));
  assert(offsetof(struct CVolumeInfo, name) <
         offsetof(struct CVolumeInfo, created_at));
  printf("  ok: CVolumeInfo carries a name field\n");
}

/* (1) Both mount origins are reachable, and neither crashes on a NULL handle.
 */
static void test_mount_origin_entrypoints(void) {
  acrylmicrovm_options_add_bind_mount(NULL, "/host/data", "/data", 0);
  acrylmicrovm_options_add_managed_volume(NULL, "my-data", "/data", 0);
  printf(
      "  ok: bind-mount and managed-volume entrypoints accept a NULL handle\n");
}

/* (2) The name parameter exists, and a NULL handle is still rejected
 * synchronously rather than queueing work against nothing. */
static void test_volume_create_rejects_null_handle(void) {
  CAcrylMicrovmError error = {0};

  enum AcrylMicrovmErrorCode named =
      acrylmicrovm_volume_create(NULL, "my-data", on_volume_created, NULL, &error);
  assert(named == InvalidArgument);
  acrylmicrovm_error_free(&error);

  enum AcrylMicrovmErrorCode unnamed =
      acrylmicrovm_volume_create(NULL, NULL, on_volume_created, NULL, &error);
  assert(unnamed == InvalidArgument);
  acrylmicrovm_error_free(&error);

  printf(
      "  ok: acrylmicrovm_volume_create takes a name and rejects a NULL handle\n");
}

int main(void) {
  printf("test_volume_ffi\n");
  test_volume_info_layout();
  test_mount_origin_entrypoints();
  test_volume_create_rejects_null_handle();
  printf("PASS\n");
  return 0;
}
