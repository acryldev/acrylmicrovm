# ACRYL microVMs C Examples

This directory contains examples demonstrating the ACRYL microVMs C SDK v0.2.0 API.

## Prerequisites

1. Build the ACRYL microVMs C SDK from the repository root:
   ```bash
   # Initialize submodules (REQUIRED!)
   git submodule update --init --recursive

   # Build C SDK
   cargo build --release -p acrylmicrovm-c
   ```

   This creates:
   - `target/release/libacrylmicrovm.{dylib,so}` - Shared library
   - `sdks/c/include/acrylmicrovm.h` - Header file

## Building Examples

```bash
# From this directory (examples/c/)
mkdir -p build && cd build
cmake ..
make
```

## Examples Overview

### Basic Examples

| Example | Description |
|---------|-------------|
| `simple_api_demo` | Quick start using the Simple API |
| `execute` | Command execution with streaming output |
| `shutdown` | Graceful runtime shutdown with multiple boxes |

### Advanced Examples

| Example | Description |
|---------|-------------|
| `01_lifecycle` | Complete box lifecycle: create → stop → restart → remove |
| `02_list_boxes` | Discovery, introspection, ID prefix lookup, runtime metrics |
| `03_streaming_output` | Real-time output handling with callbacks |
| `04_error_handling` | Error codes, retry logic, graceful degradation |
| `05_metrics` | Runtime and per-box performance metrics |

---

## Running Examples

### simple_api_demo - Simple API Quick Start

The easiest way to get started.

```bash
./simple_api_demo
```

**Code pattern:**
```c
CAcrylMicrovmSimple* box = NULL;
CAcrylMicrovmError error = {0};

// Create box with defaults
if (acrylmicrovm_simple_new("python:slim", 0, 0, &box, &error) != Ok) {
    fprintf(stderr, "Error: %s\n", error.message);
    acrylmicrovm_error_free(&error);
    return 1;
}

// Run command
const char* args[] = {"-c", "print('hello')", NULL};
CAcrylMicrovmExecResult* result = NULL;

if (acrylmicrovm_simple_run(box, "python", args, 2, &result, &error) == Ok) {
    printf("Output: %s\n", result->stdout_text);
    acrylmicrovm_result_free(result);
}

acrylmicrovm_simple_free(box);  // Auto-cleanup
```

---

### execute - Command Execution with Streaming

Demonstrates the Native API with real-time output streaming.

```bash
./execute
```

**Expected output:**
```
AcrylMicrovm C SDK Example
Version: 0.5.7

Created box, executing commands...

Command 1: ls -la /
---
[directory listing output...]

Exit code: 0
```

**Code pattern:**
```c
void output_callback(const char* text, int is_stderr, void* user_data) {
    fprintf(is_stderr ? stderr : stdout, "%s", text);
}

CAcrylMicrovmRuntime* runtime = NULL;
CBoxHandle* box = NULL;
CAcrylMicrovmError error = {0};

// Create runtime and box
acrylmicrovm_runtime_new(NULL, NULL, 0, &runtime, &error);

CAcrylMicrovmOptions* opts = NULL;
acrylmicrovm_options_new("alpine:3.19", &opts, &error);
acrylmicrovm_options_set_network_enabled(opts);
acrylmicrovm_create_box(runtime, opts, &box, &error);
acrylmicrovm_options_free(opts);

// Execute with streaming callback
int exit_code = 0;
const char* args[] = {"-la", "/"};
AcrylMicrovmCommand cmd = {.command = "/bin/ls", .args = args, .argc = 2};
CExecutionHandle* execution = NULL;
if (acrylmicrovm_execute(box, &cmd, output_callback, NULL, &execution, &error) == Ok) {
    acrylmicrovm_execution_wait(execution, &exit_code, &error);
    acrylmicrovm_execution_free(execution);
}
printf("Exit code: %d\n", exit_code);

acrylmicrovm_runtime_free(runtime);
```

---

### shutdown - Runtime Shutdown

Demonstrates graceful shutdown of multiple boxes.

```bash
./shutdown
```

**Code pattern:**
```c
// Create multiple boxes
CBoxHandle* box1 = NULL;
CBoxHandle* box2 = NULL;
acrylmicrovm_create_box(runtime, opts, &box1, &error);
acrylmicrovm_create_box(runtime, opts, &box2, &error);

// Graceful shutdown (waits up to 10 seconds per box)
acrylmicrovm_runtime_shutdown(runtime, 0, &error);

acrylmicrovm_runtime_free(runtime);
```

---

### 01_lifecycle - Box Lifecycle Management

Demonstrates the complete lifecycle: create → stop → restart → remove.

```bash
./01_lifecycle
```

**Key operations:**
```c
// Create and start
acrylmicrovm_create_box(runtime, opts, &box, &error);

// Stop (preserves state)
acrylmicrovm_stop_box(box, &error);

// Restart using acrylmicrovm_get
char* box_id = acrylmicrovm_box_id(box);
acrylmicrovm_get(runtime, box_id, &box, &error);
acrylmicrovm_start_box(box, &error);

// Remove
acrylmicrovm_remove(runtime, box_id, 1, &error);
acrylmicrovm_free_string(box_id);
```

---

### 02_list_boxes - Discovery and Introspection

Demonstrates listing boxes, getting info, and ID prefix lookup.

```bash
./02_list_boxes
```

**Key operations:**
```c
typedef struct {
    int done;
} InfoRequest;

static void on_info(CBoxInfo* info, CAcrylMicrovmError* error, void* user_data) {
    InfoRequest* request = user_data;
    if (error->code == Ok) {
        printf("%s\n", info->id);
        acrylmicrovm_free_box_info(info);  // Callback owns a successful result.
    }
    request->done = 1;
}

static void on_list(CBoxInfoList* list, CAcrylMicrovmError* error,
                    void* user_data) {
    InfoRequest* request = user_data;
    if (error->code == Ok) {
        printf("Boxes: %d\n", list->count);
        acrylmicrovm_free_box_info_list(list);
    }
    request->done = 1;
}

static void drain_until_done(CAcrylMicrovmRuntime* runtime, InfoRequest* request) {
    CAcrylMicrovmError error = {0};
    while (!request->done) {
        if (acrylmicrovm_runtime_drain(runtime, -1, &error) < 0) {
            acrylmicrovm_error_free(&error);
            return;
        }
    }
}

InfoRequest list_request = {0};
if (acrylmicrovm_list_info(runtime, on_list, &list_request, &error) == Ok) {
    drain_until_done(runtime, &list_request);
} else {
    acrylmicrovm_error_free(&error);
}

InfoRequest get_request = {0};
if (acrylmicrovm_get_info(runtime, "01HJK4TN", on_info, &get_request, &error) == Ok) {
    drain_until_done(runtime, &get_request);  // ID prefix works.
} else {
    acrylmicrovm_error_free(&error);
}

InfoRequest handle_request = {0};
if (acrylmicrovm_box_info(box, on_info, &handle_request, &error) == Ok) {
    drain_until_done(runtime, &handle_request);
} else {
    acrylmicrovm_error_free(&error);
}
```

---

### 03_streaming_output - Real-time Output Handling

Demonstrates advanced streaming with user data and statistics.

```bash
./03_streaming_output
```

**Code pattern:**
```c
typedef struct {
    int stdout_chunks;
    int stderr_chunks;
} OutputStats;

void counting_callback(const char* text, int is_stderr, void* user_data) {
    OutputStats* stats = (OutputStats*)user_data;
    if (is_stderr) stats->stderr_chunks++;
    else stats->stdout_chunks++;
    printf("%s", text);
}

OutputStats stats = {0};
AcrylMicrovmCommand cmd = {.command = "/bin/ls", .args = args, .argc = 2};
CExecutionHandle* execution = NULL;
if (acrylmicrovm_execute(box, &cmd, counting_callback, &stats, &execution, &error) == Ok) {
    acrylmicrovm_execution_wait(execution, &exit_code, &error);
    acrylmicrovm_execution_free(execution);
}
printf("Received %d stdout chunks, %d stderr chunks\n",
       stats.stdout_chunks, stats.stderr_chunks);
```

---

### 04_error_handling - Error Recovery Patterns

Demonstrates error codes, retry logic, and graceful degradation.

```bash
./04_error_handling
```

**Key patterns:**
```c
// Switch on error codes
AcrylMicrovmErrorCode code = acrylmicrovm_get(runtime, "nonexistent", &box, &error);
switch (code) {
    case Ok:
        break;
    case NotFound:
        printf("Box not found - expected for this test\n");
        break;
    case InvalidArgument:
        printf("Invalid argument: %s\n", error.message);
        break;
    default:
        printf("Error %d: %s\n", code, error.message);
}
acrylmicrovm_error_free(&error);

// Retry logic
for (int i = 0; i < 3; i++) {
    code = acrylmicrovm_simple_new(image, 0, 0, &box, &error);
    if (code == Ok) break;
    acrylmicrovm_error_free(&error);
    sleep(1);
}
```

---

### 05_metrics - Performance Monitoring

Demonstrates runtime and per-box metrics.

```bash
./05_metrics
```

**Code pattern:**
```c
// Runtime metrics
CRuntimeMetrics metrics = {0};
acrylmicrovm_runtime_metrics(runtime, &metrics, &error);
printf("Running boxes: %d\n", metrics.num_running_boxes);

// Per-box metrics
CBoxMetrics box_metrics = {0};
acrylmicrovm_box_metrics(box, &box_metrics, &error);
printf("Memory bytes: %lld\n", (long long)box_metrics.memory_bytes);
```

---

## API Overview (v0.2.0)

The C SDK v0.2.0 uses structured error handling:

```c
// Initialize error struct
CAcrylMicrovmError error = {0};

// All functions return AcrylMicrovmErrorCode
AcrylMicrovmErrorCode code = acrylmicrovm_runtime_new(NULL, NULL, 0, &runtime, &error);

// Check result
if (code != Ok) {
    fprintf(stderr, "Error %d: %s\n", error.code, error.message);
    acrylmicrovm_error_free(&error);  // Always free on error
    return 1;
}
```

**Error codes:**
| Code | Value | Meaning |
|------|-------|---------|
| `Ok` | 0 | Success |
| `NotFound` | 2 | Resource not found |
| `InvalidState` | 4 | Invalid operation state |
| `InvalidArgument` | 5 | Bad parameter |
| `Image` | 8 | Image pull failed |
| `Execution` | 10 | Command failed |

---

## Troubleshooting

**Library not found:**
```bash
# macOS
export DYLD_LIBRARY_PATH=/path/to/acrylmicrovm/target/release:$DYLD_LIBRARY_PATH

# Linux
export LD_LIBRARY_PATH=/path/to/acrylmicrovm/target/release:$LD_LIBRARY_PATH
```

**Header not found:**
- Verify `sdks/c/include/acrylmicrovm.h` exists
- Check CMakeLists.txt `ACRYLMICROVM_ROOT` path

**Runtime errors:**
- Enable debug logging: `RUST_LOG=debug ./example`
- Check disk space: `df -h ~/.acryl/microvm`
- Verify KVM (Linux): `ls -l /dev/kvm`

---

## More Information

- **[C SDK README](../../sdks/c/README.md)** - Complete documentation
- **[C API Reference](../../docs/reference/c/README.md)** - Function signatures
- **[C Quickstart](../../docs/getting-started/quickstart-c.md)** - 5-minute guide
