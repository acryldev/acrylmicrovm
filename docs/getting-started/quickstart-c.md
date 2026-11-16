# C quick start

Get up and running with ACRYL microVMs C SDK in 5 minutes.

## Installation

### Prerequisites

**macOS (Apple Silicon):**
- macOS 11.0+ (Big Sur or later)
- Xcode Command Line Tools: `xcode-select --install`
- GCC or Clang

**Linux:**
- x86_64 or ARM64 architecture
- KVM enabled (`/dev/kvm` accessible)
- GCC or Clang, CMake 3.15+

### Building the SDK

```bash
# Clone the repository
git clone https://github.com/acryldev/acrylmicrovm.git
cd acrylmicrovm

# Initialize submodules (REQUIRED!)
git submodule update --init --recursive

# Build C SDK
cargo build --release -p acrylmicrovm-c

# Outputs:
# - target/release/libacrylmicrovm.{dylib,so}  (shared library)
# - target/release/libacrylmicrovm.a           (static library)
# - sdks/c/include/acrylmicrovm.h              (header file)
```

**Verify Build:**
```bash
ls -la target/release/libacrylmicrovm.*
ls -la sdks/c/include/acrylmicrovm.h
```

---

## Simple API (easiest)

Create a file `hello.c`:

```c
#include <stdio.h>
#include "acrylmicrovm.h"

int main() {
    CAcrylMicrovmSimple* box = NULL;
    CAcrylMicrovmError error = {0};

    // Create box and auto-start it
    if (acrylmicrovm_simple_new("python:slim", 0, 0, &box, &error) != Ok) {
        fprintf(stderr, "Error %d: %s\n", error.code, error.message);
        acrylmicrovm_error_free(&error);
        return 1;
    }

    // Run command and get buffered result
    const char* args[] = {"-c", "print('Hello from AcrylMicrovm!')", NULL};
    CAcrylMicrovmExecResult* result = NULL;

    if (acrylmicrovm_simple_run(box, "python", args, 2, &result, &error) == Ok) {
        printf("Output: %s", result->stdout_text);
        printf("Exit code: %d\n", result->exit_code);
        acrylmicrovm_result_free(result);
    } else {
        fprintf(stderr, "Exec error: %s\n", error.message);
        acrylmicrovm_error_free(&error);
    }

    acrylmicrovm_simple_free(box);  // Auto-cleanup
    return 0;
}
```

Build and run:

```bash
# macOS
gcc -o hello hello.c \
    -I/path/to/acrylmicrovm/sdks/c/include \
    -L/path/to/acrylmicrovm/target/release \
    -lacrylmicrovm

export DYLD_LIBRARY_PATH=/path/to/acrylmicrovm/target/release:$DYLD_LIBRARY_PATH
./hello

# Linux
gcc -o hello hello.c \
    -I/path/to/acrylmicrovm/sdks/c/include \
    -L/path/to/acrylmicrovm/target/release \
    -lacrylmicrovm

export LD_LIBRARY_PATH=/path/to/acrylmicrovm/target/release:$LD_LIBRARY_PATH
./hello
```

**What's happening:**
1. ACRYL microVMs pulls the `python:slim` OCI image (first run only)
2. Creates a lightweight VM with the image
3. Executes the Python command inside the VM
4. Buffers stdout/stderr and returns the result
5. Automatically cleans up when `acrylmicrovm_simple_free()` is called

---

## Native API (full control)

For advanced use cases with streaming output and custom configuration.

Create a file `native.c`:

```c
#include <stdio.h>
#include "acrylmicrovm.h"

void output_callback(const char* text, int is_stderr, void* user_data) {
    FILE* stream = is_stderr ? stderr : stdout;
    fprintf(stream, "%s", text);
}

int main() {
    CAcrylMicrovmRuntime* runtime = NULL;
    CBoxHandle* box = NULL;
    CAcrylMicrovmError error = {0};

    // Create runtime
    if (acrylmicrovm_runtime_new(NULL, NULL, 0, &runtime, &error) != Ok) {
        fprintf(stderr, "Runtime error: %s\n", error.message);
        acrylmicrovm_error_free(&error);
        return 1;
    }

    printf("AcrylMicrovm v%s\n", acrylmicrovm_version());

    // Create box with typed options
    CAcrylMicrovmOptions* opts = NULL;
    if (acrylmicrovm_options_new("alpine:3.19", &opts, &error) != Ok) {
        fprintf(stderr, "Options error: %s\n", error.message);
        acrylmicrovm_error_free(&error);
        acrylmicrovm_runtime_free(runtime);
        return 1;
    }
    acrylmicrovm_options_set_cpus(opts, 2);
    acrylmicrovm_options_set_memory(opts, 512);
    acrylmicrovm_options_set_network_enabled(opts);

    if (acrylmicrovm_create_box(runtime, opts, &box, &error) != Ok) {
        fprintf(stderr, "Box error: %s\n", error.message);
        acrylmicrovm_error_free(&error);
        acrylmicrovm_options_free(opts);
        acrylmicrovm_runtime_free(runtime);
        return 1;
    }
    acrylmicrovm_options_free(opts);

    // Execute commands with streaming output
    int exit_code = 0;
    const char* args[] = {"-la", "/"};
    AcrylMicrovmCommand cmd = {.command = "/bin/ls", .args = args, .argc = 2};
    CExecutionHandle* execution = NULL;

    printf("\n--- Running: ls -la / ---\n");
    if (acrylmicrovm_execute(box, &cmd, output_callback, NULL, &execution, &error) == Ok) {
        if (acrylmicrovm_execution_wait(execution, &exit_code, &error) == Ok) {
            printf("\nExit code: %d\n", exit_code);
        }
        acrylmicrovm_execution_free(execution);
    }
    if (error.code != Ok) {
        fprintf(stderr, "Execute error: %s\n", error.message);
        acrylmicrovm_error_free(&error);
    }

    // Cleanup (runtime frees all boxes)
    acrylmicrovm_runtime_free(runtime);
    return 0;
}
```

Build and run using the same commands as above.

---

## Running examples

ACRYL microVMs includes 8 comprehensive C examples:

```bash
# Navigate to examples
cd examples/c

# Build examples with CMake
mkdir -p build && cd build
cmake ..
make

# Run examples
./simple_api_demo    # Simple API basics
./execute            # Command execution with streaming
./shutdown           # Graceful shutdown
./01_lifecycle       # Create/stop/restart/remove
./02_list_boxes      # Discovery and introspection
./03_streaming_output  # Real-time output handling
./04_error_handling  # Error recovery patterns
./05_metrics         # Performance monitoring
```

**Examples overview:**

| Example | Description |
|---------|-------------|
| `simple_api_demo.c` | Quick start with Simple API |
| `execute.c` | Command execution with streaming output |
| `shutdown.c` | Runtime shutdown with multiple boxes |
| `01_lifecycle.c` | Complete box lifecycle (create/stop/restart/remove) |
| `02_list_boxes.c` | Discovery, introspection, ID prefix lookup |
| `03_streaming_output.c` | Real-time output handling with callbacks |
| `04_error_handling.c` | Error codes, retry logic, graceful degradation |
| `05_metrics.c` | Runtime and per-box metrics |

---

## Error handling

The C SDK uses structured error handling:

```c
CAcrylMicrovmError error = {0};  // Always initialize to zero
AcrylMicrovmErrorCode code = acrylmicrovm_simple_new(..., &error);

if (code != Ok) {
    // Check specific error codes
    switch (code) {
        case NotFound:
            printf("Resource not found\n");
            break;
        case Image:
            printf("Image pull failed: %s\n", error.message);
            break;
        default:
            printf("Error %d: %s\n", code, error.message);
    }
    acrylmicrovm_error_free(&error);  // Always free on error
}
```

**Error codes:**
- `Ok (0)` - Success
- `NotFound (2)` - Resource not found
- `InvalidArgument (5)` - Invalid parameter
- `Image (8)` - Image pull/resolution failed
- `Execution (10)` - Command execution failed

See [C SDK API Reference](../reference/c/README.md#acrylmicrovmerrorcode) for the complete list.

---

## Next steps

- **[C SDK README](../../sdks/c/README.md)** - Complete SDK documentation
  - Simple API and Native API details
  - Typed box options
  - Memory management rules
  - Threading and safety
  - Troubleshooting guide

- **[C SDK API Reference](../reference/c/README.md)** - Function signatures and parameters

- **[C Examples](../../examples/c/)** - Working code examples
