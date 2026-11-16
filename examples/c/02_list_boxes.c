// Modified by ACRYL, 2026
/**
 * ACRYL microVMs C SDK - Example 2: List and inspect boxes.
 */

#include "example_common.h"
#include <stdio.h>
#include <string.h>

static void print_box_info(const CBoxInfo *info) {
  printf("id=%s image=%s status=%s running=%d pid=%d\n",
         info->id ? info->id : "", info->image ? info->image : "",
         info->status ? info->status : "", info->running, info->pid);
}

typedef struct InfoRequest {
  const char *heading;
  int done;
} InfoRequest;

static void on_box_info(CBoxInfo *info, CAcrylMicrovmError *error, void *user_data) {
  InfoRequest *request = user_data;
  if (error->code != Ok) {
    print_error(request->heading, error);
  } else if (info == NULL) {
    fprintf(stderr, "%s failed: callback returned no box info\n",
            request->heading);
  } else {
    printf("%s:\n", request->heading);
    print_box_info(info);
    acrylmicrovm_free_box_info(info);
  }
  request->done = 1;
}

static void on_box_list(CBoxInfoList *list, CAcrylMicrovmError *error,
                        void *user_data) {
  InfoRequest *request = user_data;
  if (error->code != Ok) {
    print_error(request->heading, error);
  } else if (list == NULL) {
    fprintf(stderr, "%s failed: callback returned no box list\n",
            request->heading);
  } else {
    printf("All boxes (%d):\n", list->count);
    for (int i = 0; i < list->count; i++) {
      print_box_info(&list->items[i]);
    }
    acrylmicrovm_free_box_info_list(list);
  }
  request->done = 1;
}

int main(void) {
  printf("=== AcrylMicrovm Example: List and Inspect Boxes ===\n\n");

  CAcrylMicrovmRuntime *runtime = create_runtime_or_exit();
  if (runtime == NULL) {
    return 1;
  }

  CBoxHandle *box1 = create_alpine_box_or_exit(runtime);
  CBoxHandle *box2 = create_alpine_box_or_exit(runtime);
  CBoxHandle *box3 = create_alpine_box_or_exit(runtime);
  if (box1 == NULL || box2 == NULL || box3 == NULL) {
    acrylmicrovm_runtime_free(runtime);
    return 1;
  }

  char *id1 = acrylmicrovm_box_id(box1);
  char *id2 = acrylmicrovm_box_id(box2);
  char *id3 = acrylmicrovm_box_id(box3);
  printf("Created boxes:\n  %s\n  %s\n  %s\n\n", id1, id2, id3);

  CAcrylMicrovmError error = {0};
  InfoRequest list_request = {.heading = "list boxes", .done = 0};
  AcrylMicrovmErrorCode code =
      acrylmicrovm_list_info(runtime, on_box_list, &list_request, &error);
  if (code != Ok) {
    print_error("list boxes", &error);
    acrylmicrovm_error_free(&error);
  } else {
    example_drain_until_done(runtime, &list_request.done, "drain list boxes");
  }
  printf("\n");

  InfoRequest id_request = {.heading = "Box 1 info", .done = 0};
  code = acrylmicrovm_get_info(runtime, id1, on_box_info, &id_request, &error);
  if (code != Ok) {
    print_error("get box info", &error);
    acrylmicrovm_error_free(&error);
  } else {
    example_drain_until_done(runtime, &id_request.done, "drain get box info");
  }
  printf("\n");

  char prefix[9] = {0};
  size_t prefix_length = strlen(id3);
  if (prefix_length > 8) {
    prefix_length = 8;
  }
  for (size_t index = 0; index < prefix_length; index++) {
    prefix[index] = id3[index];
  }
  char prefix_heading[64] = {0};
  snprintf(prefix_heading, sizeof(prefix_heading), "Prefix lookup (%s)",
           prefix);
  InfoRequest prefix_request = {.heading = prefix_heading, .done = 0};
  code =
      acrylmicrovm_get_info(runtime, prefix, on_box_info, &prefix_request, &error);
  if (code != Ok) {
    print_error("prefix lookup", &error);
    acrylmicrovm_error_free(&error);
  } else {
    example_drain_until_done(runtime, &prefix_request.done,
                             "drain prefix lookup");
  }
  printf("\n");

  CRuntimeMetrics metrics = {0};
  code = example_get_runtime_metrics(runtime, &metrics, &error);
  if (code == Ok) {
    printf("Runtime metrics: created=%d running=%d commands=%d\n",
           metrics.boxes_created_total, metrics.num_running_boxes,
           metrics.total_commands_executed);
  }

  example_remove_box(runtime, id1, 1, &error);
  example_remove_box(runtime, id2, 1, &error);
  example_remove_box(runtime, id3, 1, &error);
  acrylmicrovm_box_free(box1);
  acrylmicrovm_box_free(box2);
  acrylmicrovm_box_free(box3);
  acrylmicrovm_free_string(id1);
  acrylmicrovm_free_string(id2);
  acrylmicrovm_free_string(id3);
  acrylmicrovm_runtime_free(runtime);

  printf("\n=== List and Inspect Demo Complete ===\n");
  return 0;
}
