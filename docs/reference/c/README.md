# C SDK API reference

Complete API reference for the ACRYL microVMs C SDK.

## Overview

The C SDK provides C-compatible FFI bindings for integrating ACRYL microVMs into C/C++ applications.

**Library**: `libacrylmicrovm`
**Header**: `acrylmicrovm.h`
**C Standard**: C11-compatible compiler (GCC/Clang)

### API styles

The SDK provides two API styles:

1. **Simple API** (`acrylmicrovm_simple_*`) - Convenience layer for common use cases
   - No runtime setup required
   - Auto-managed runtime
   - Buffered command results

2. **Native API** (`acrylmicrovm_*`) - Full-featured, flexible interface
   - Typed `CAcrylMicrovmOptions` configuration
   - Streaming output callbacks
   - Advanced features (volumes, networking, etc.)

---

## Table of contents

- [Quick Start](#quick-start)
- [Error Handling](#error-handling)
  - [AcrylMicrovmErrorCode](#acrylmicrovmerrorcode)
  - [CAcrylMicrovmError](#cacrylmicrovmerror)
  - [Error Handling Patterns](#error-handling-patterns)
- [Simple API](#simple-api)
  - [acrylmicrovm_simple_new](#acrylmicrovm_simple_new)
  - [acrylmicrovm_simple_run](#acrylmicrovm_simple_run)
  - [acrylmicrovm_simple_free](#acrylmicrovm_simple_free)
  - [acrylmicrovm_result_free](#acrylmicrovm_result_free)
- [Native API](#native-api)
  - [Runtime Management](#runtime-management)
  - [Box Management](#box-management)
  - [Network Tunnels](#network-tunnels)
  - [Command Execution](#command-execution)
  - [Discovery & Introspection](#discovery--introspection)
  - [Images](#images)
  - [Registries](#registries)
  - [Guest SSH Control](#guest-ssh-control)
  - [Metrics](#metrics)
- [Memory Management](#memory-management)
- [Thread Safety](#thread-safety)
- [Platform Requirements](#platform-requirements)
- [Migration from v0.1.x](#migration-from-v01x)

---

## Quick start

### Simple API (recommended)

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
        printf("Output: %s\n", result->stdout_text);
        printf("Exit code: %d\n", result->exit_code);
        acrylmicrovm_result_free(result);
    }

    acrylmicrovm_simple_free(box);  // Auto-cleanup
    return 0;
}
```

### Native API (full control)

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
        fprintf(stderr, "Error %d: %s\n", error.code, error.message);
        acrylmicrovm_error_free(&error);
        return 1;
    }

    // Create box with typed options
    CAcrylMicrovmOptions* opts = NULL;
    if (acrylmicrovm_options_new("alpine:3.19", &opts, &error) != Ok) {
        fprintf(stderr, "Error %d: %s\n", error.code, error.message);
        acrylmicrovm_error_free(&error);
        acrylmicrovm_runtime_free(runtime);
        return 1;
    }
    acrylmicrovm_options_set_network_enabled(opts);
    CAdvancedBoxOptions* advanced = NULL;
    if (acrylmicrovm_advanced_options_new(&advanced, &error) != Ok) {
        acrylmicrovm_options_free(opts);
        return 1;
    }
    const char* cap_add[] = {"NET_ADMIN"};
    const char* cap_drop[] = {"NET_RAW"};
    if (acrylmicrovm_advanced_options_set_capabilities_add(advanced, cap_add, 1) != Ok ||
        acrylmicrovm_advanced_options_set_capabilities_drop(advanced, cap_drop, 1) != Ok) {
        fprintf(stderr, "Invalid Linux capability list\n");
        acrylmicrovm_advanced_options_free(advanced);
        acrylmicrovm_options_free(opts);
        return 1;
    }
    acrylmicrovm_options_set_advanced(opts, advanced);
    acrylmicrovm_advanced_options_free(advanced);

    if (acrylmicrovm_create_box(runtime, opts, &box, &error) != Ok) {
        fprintf(stderr, "Error %d: %s\n", error.code, error.message);
        acrylmicrovm_error_free(&error);
        acrylmicrovm_options_free(opts);
        acrylmicrovm_runtime_free(runtime);
        return 1;
    }
    acrylmicrovm_options_free(opts);

    // Start command with streaming output, then wait for completion
    int exit_code = 0;
    const char* args[] = {"-la", "/"};
    AcrylMicrovmCommand cmd = {.command = "/bin/ls", .args = args, .argc = 2};
    CExecutionHandle* execution = NULL;

    if (acrylmicrovm_execute(box, &cmd, output_callback, NULL, &execution, &error) == Ok) {
        if (acrylmicrovm_execution_wait(execution, &exit_code, &error) == Ok) {
            printf("\nExit code: %d\n", exit_code);
        }
        acrylmicrovm_execution_free(execution);
    }
    if (error.code != Ok) {
        fprintf(stderr, "Error: %s\n", error.message);
        acrylmicrovm_error_free(&error);
    }

    // Cleanup
    acrylmicrovm_runtime_free(runtime);
    return 0;
}
```

### Building

```bash
# Compile with the AcrylMicrovm library
gcc -I/path/to/acrylmicrovm/sdks/c/include \
    -L/path/to/acrylmicrovm/target/release \
    -lacrylmicrovm \
    my_program.c -o my_program

# macOS: Set library path
export DYLD_LIBRARY_PATH=/path/to/acrylmicrovm/target/release:$DYLD_LIBRARY_PATH

# Linux: Set library path
export LD_LIBRARY_PATH=/path/to/acrylmicrovm/target/release:$LD_LIBRARY_PATH
```

---

## Error handling

The C SDK introduces structured error handling with error codes and detailed messages.

### AcrylMicrovmErrorCode

All API functions return `AcrylMicrovmErrorCode` to indicate success or failure type:

```c
typedef enum AcrylMicrovmErrorCode {
    Ok = 0,               // Success
    Internal = 1,         // Internal error
    NotFound = 2,         // Resource not found
    AlreadyExists = 3,    // Resource already exists
    InvalidState = 4,     // Invalid state for operation
    InvalidArgument = 5,  // Invalid argument
    Config = 6,           // Configuration error
    Storage = 7,          // Storage error
    Image = 8,            // Image error
    Network = 9,          // Network error
    Execution = 10,       // Execution error
    Stopped = 11,         // Resource stopped
    Engine = 12,          // Engine error
    Unsupported = 13,     // Unsupported operation
    Database = 14,        // Database error
    Portal = 15,          // Portal/communication error
    Rpc = 16,             // RPC error
    RpcTransport = 17,    // RPC transport error
    Metadata = 18,        // Metadata error
    UnsupportedEngine = 19, // Unsupported engine error
} AcrylMicrovmErrorCode;
```

### CAcrylMicrovmError

Detailed error information for debugging:

```c
typedef struct CAcrylMicrovmError {
    AcrylMicrovmErrorCode code;  // Error code for programmatic handling
    char* message;          // Detailed message (NULL if none)
} CAcrylMicrovmError;
```

### Error handling patterns

**Pattern 1: Basic Check**

```c
CAcrylMicrovmError error = {0};
AcrylMicrovmErrorCode code = acrylmicrovm_simple_new("alpine:3.19", 0, 0, &box, &error);

if (code != Ok) {
    fprintf(stderr, "Error %d: %s\n", error.code, error.message);
    acrylmicrovm_error_free(&error);
    return 1;
}
```

**Pattern 2: Switch on Error Code**

```c
AcrylMicrovmErrorCode code = acrylmicrovm_get(runtime, "box-id", &box, &error);

switch (code) {
    case Ok:
        // Success - use box
        break;
    case NotFound:
        fprintf(stderr, "Box not found\n");
        break;
    case InvalidState:
        fprintf(stderr, "Box in invalid state\n");
        break;
    default:
        fprintf(stderr, "Error %d: %s\n", error.code, error.message);
}

acrylmicrovm_error_free(&error);
```

**Pattern 3: Retry Logic**

```c
int retries = 3;
for (int i = 0; i < retries; i++) {
    code = acrylmicrovm_simple_new("alpine:3.19", 0, 0, &box, &error);
    if (code == Ok) break;

    fprintf(stderr, "Retry %d/%d: %s\n", i+1, retries, error.message);
    acrylmicrovm_error_free(&error);

    if (code == InvalidArgument || code == Unsupported) {
        break;  // Non-retryable errors
    }
    sleep(1);  // Backoff
}
```

---

## Simple API

The Simple API provides a streamlined interface for common use cases.

### acrylmicrovm_simple_new

Create and auto-start a box with sensible defaults.

```c
AcrylMicrovmErrorCode acrylmicrovm_simple_new(
    const char* image,
    double cpus,
    int memory_mib,
    CAcrylMicrovmSimple** out_box,
    CAcrylMicrovmError* out_error
);
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `image` | `const char*` | OCI image reference (e.g., `"python:slim"`, `"alpine:3.19"`) |
| `cpus` | `double` | CPU allocation; fractional values require a Linux execution host, 0 selects the default in the Simple API |
| `memory_mib` | `int` | Memory in MiB (0 = default: 512) |
| `out_box` | `CAcrylMicrovmSimple**` | Output: created box handle |
| `out_error` | `CAcrylMicrovmError*` | Output: error information |

#### Returns

`AcrylMicrovmErrorCode` - `Ok` on success, error code on failure.

#### Example

```c
CAcrylMicrovmSimple* box = NULL;
CAcrylMicrovmError error = {0};

// Default resources
if (acrylmicrovm_simple_new("alpine:3.19", 0, 0, &box, &error) != Ok) {
    fprintf(stderr, "Error: %s\n", error.message);
    acrylmicrovm_error_free(&error);
    return 1;
}

// Custom resources
if (acrylmicrovm_simple_new("python:slim", 4, 2048, &box, &error) != Ok) {
    // Handle error
}
```

---

### acrylmicrovm_simple_run

Run a command and get buffered result.

```c
AcrylMicrovmErrorCode acrylmicrovm_simple_run(
    CAcrylMicrovmSimple* box,
    const char* command,
    const char* const* args,
    int argc,
    CAcrylMicrovmExecResult** out_result,
    CAcrylMicrovmError* out_error
);
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `box` | `CAcrylMicrovmSimple*` | Box handle from `acrylmicrovm_simple_new` |
| `command` | `const char*` | Command to execute |
| `args` | `const char* const*` | NULL-terminated array of arguments |
| `argc` | `int` | Number of arguments (excluding NULL terminator) |
| `out_result` | `CAcrylMicrovmExecResult**` | Output: execution result |
| `out_error` | `CAcrylMicrovmError*` | Output: error information |

#### Result structure

```c
typedef struct CAcrylMicrovmExecResult {
    int exit_code;       // Command exit code
    char* stdout_text;   // Standard output
    char* stderr_text;   // Standard error
} CAcrylMicrovmExecResult;
```

#### Example

```c
const char* args[] = {"-c", "print('hello')", NULL};
CAcrylMicrovmExecResult* result = NULL;

if (acrylmicrovm_simple_run(box, "python", args, 2, &result, &error) == Ok) {
    printf("stdout: %s\n", result->stdout_text);
    printf("stderr: %s\n", result->stderr_text);
    printf("exit: %d\n", result->exit_code);
    acrylmicrovm_result_free(result);
}
```

---

### acrylmicrovm_simple_free

Free a simple box (auto-stops and removes).

```c
void acrylmicrovm_simple_free(CAcrylMicrovmSimple* box);
```

Safe to call with NULL.

---

### acrylmicrovm_result_free

Free an execution result.

```c
void acrylmicrovm_result_free(CAcrylMicrovmExecResult* result);
```

Safe to call with NULL.

---

## Native API

### Runtime management

#### acrylmicrovm_version

Get ACRYL microVMs version string.

```c
const char* acrylmicrovm_version(void);
```

Returns static string (do not free). Example: `"0.5.7"`.

---

#### acrylmicrovm_runtime_new

Create a new runtime instance.

```c
typedef enum AcrylMicrovmRegistryTransport {
    AcrylMicrovmRegistryTransportHttps = 0,
    AcrylMicrovmRegistryTransportHttp = 1,
} AcrylMicrovmRegistryTransport;

typedef struct AcrylMicrovmImageRegistry {
    const char* host;
    AcrylMicrovmRegistryTransport transport;
    int skip_verify;
    int search;
    const char* username;
    const char* password;
    const char* bearer_token;
} AcrylMicrovmImageRegistry;

AcrylMicrovmErrorCode acrylmicrovm_runtime_new(
    const char* home_dir,
    const AcrylMicrovmImageRegistry* image_registries,
    int image_registries_count,
    CAcrylMicrovmRuntime** out_runtime,
    CAcrylMicrovmError* out_error
);
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `home_dir` | `const char*` | Path to ACRYL microVMs home. `NULL` = default (`~/.acryl/microvm`) |
| `image_registries` | `const AcrylMicrovmImageRegistry*` | Optional registry transport, TLS, search, and auth settings |
| `image_registries_count` | `int` | Number of entries in `image_registries` |
| `out_runtime` | `CAcrylMicrovmRuntime**` | Output: runtime handle |
| `out_error` | `CAcrylMicrovmError*` | Output: error information |

#### Example

```c
CAcrylMicrovmRuntime* runtime = NULL;
CAcrylMicrovmError error = {0};

// Default configuration
if (acrylmicrovm_runtime_new(NULL, NULL, 0, &runtime, &error) != Ok) {
    fprintf(stderr, "Error: %s\n", error.message);
    acrylmicrovm_error_free(&error);
    return 1;
}

// Custom registries
AcrylMicrovmImageRegistry image_registries[] = {
  {
    .host = "ghcr.io",
    .transport = AcrylMicrovmRegistryTransportHttps,
    .skip_verify = 0,
    .search = 1,
    .username = NULL,
    .password = NULL,
    .bearer_token = NULL,
  },
  {
    .host = "registry.example.com",
    .transport = AcrylMicrovmRegistryTransportHttps,
    .skip_verify = 0,
    .search = 0,
    .username = "user",
    .password = "password",
    .bearer_token = NULL,
  },
};
if (acrylmicrovm_runtime_new("/var/lib/acrylmicrovm", image_registries, 2, &runtime, &error) != Ok) {
    // Handle error
}
```

---

#### acrylmicrovm_runtime_shutdown

Gracefully stop all running boxes.

```c
AcrylMicrovmErrorCode acrylmicrovm_runtime_shutdown(
    CAcrylMicrovmRuntime* runtime,
    int timeout,
    CAcrylMicrovmError* out_error
);
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `runtime` | `CAcrylMicrovmRuntime*` | Runtime instance |
| `timeout` | `int` | Seconds: 0=default(10), -1=infinite, >0=custom |
| `out_error` | `CAcrylMicrovmError*` | Output: error information |

---

#### acrylmicrovm_runtime_free

Free a runtime instance.

```c
void acrylmicrovm_runtime_free(CAcrylMicrovmRuntime* runtime);
```

Safe to call with NULL. Automatically frees all boxes.

---

### Box management

#### acrylmicrovm_create_box

Create and auto-start a box.

```c
AcrylMicrovmErrorCode acrylmicrovm_create_box(
    CAcrylMicrovmRuntime* runtime,
    CAcrylMicrovmOptions* opts,
    CBoxHandle** out_box,
    CAcrylMicrovmError* out_error
);
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `runtime` | `CAcrylMicrovmRuntime*` | Runtime instance |
| `opts` | `CAcrylMicrovmOptions*` | Box options created with `acrylmicrovm_options_new()` |
| `out_box` | `CBoxHandle**` | Output: box handle |
| `out_error` | `CAcrylMicrovmError*` | Output: error information |

#### Example

```c
CAcrylMicrovmOptions* opts = NULL;
if (acrylmicrovm_options_new("alpine:3.19", &opts, &error) != Ok) {
    // Handle error
}
// Half a CPU of aggregate time on Linux, with one guest vCPU.
acrylmicrovm_options_set_cpus(opts, 0.5);
acrylmicrovm_options_set_memory(opts, 512);
acrylmicrovm_options_set_network_enabled(opts);
CAdvancedBoxOptions* advanced = NULL;
if (acrylmicrovm_advanced_options_new(&advanced, &error) != Ok) {
    acrylmicrovm_options_free(opts);
    return 1;
}
const char* cap_add[] = {"NET_ADMIN"};
const char* cap_drop[] = {"NET_RAW"};
if (acrylmicrovm_advanced_options_set_capabilities_add(advanced, cap_add, 1) != Ok ||
    acrylmicrovm_advanced_options_set_capabilities_drop(advanced, cap_drop, 1) != Ok) {
    fprintf(stderr, "Invalid Linux capability list\n");
    acrylmicrovm_advanced_options_free(advanced);
    acrylmicrovm_options_free(opts);
    return 1;
}
acrylmicrovm_options_set_advanced(opts, advanced);
acrylmicrovm_advanced_options_free(advanced);

CBoxHandle* box = NULL;
if (acrylmicrovm_create_box(runtime, opts, &box, &error) != Ok) {
    fprintf(stderr, "Error: %s\n", error.message);
    acrylmicrovm_error_free(&error);
}
acrylmicrovm_options_free(opts);
```

---

#### acrylmicrovm_start_box

Start or restart a stopped box.

```c
AcrylMicrovmErrorCode acrylmicrovm_start_box(
    CBoxHandle* handle,
    CAcrylMicrovmError* out_error
);
```

---

#### acrylmicrovm_stop_box

Stop a running box.

```c
AcrylMicrovmErrorCode acrylmicrovm_stop_box(
    CBoxHandle* handle,
    CAcrylMicrovmError* out_error
);
```

**Note:** Consumes the handle - do not use after calling.

---

#### acrylmicrovm_remove

Remove a box.

```c
AcrylMicrovmErrorCode acrylmicrovm_remove(
    CAcrylMicrovmRuntime* runtime,
    const char* id_or_name,
    int force,
    CAcrylMicrovmError* out_error
);
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `id_or_name` | `const char*` | Box ID (full or prefix) or name |
| `force` | `int` | Non-zero to force remove running box |

---

#### acrylmicrovm_get

Reattach to an existing box.

```c
AcrylMicrovmErrorCode acrylmicrovm_get(
    CAcrylMicrovmRuntime* runtime,
    const char* id_or_name,
    CBoxHandle** out_handle,
    CAcrylMicrovmError* out_error
);
```

---

#### acrylmicrovm_box_id

Get box ID string from handle.

```c
char* acrylmicrovm_box_id(CBoxHandle* handle);
```

**Important:** Caller must free with `acrylmicrovm_free_string()`.

---

#### acrylmicrovm_box_free

Free a box handle.

```c
void acrylmicrovm_box_free(CBoxHandle* handle);
```

Safe to call with NULL. Use when you need to release a box handle without freeing the entire runtime.

---

### Network tunnels

Creation takes a complete system `sockaddr_storage` and a `AcrylMicrovmTunnelOptions`
containing `bool is_public`. Initialize the address with
`acrylmicrovm_socket_address_parse("127.0.0.1:3000", &target, &error)` (IPv6 uses
`"[::1]:3000"`). NULL options mean public. The parser performs no DNS resolution
and leaves output unchanged on error. Inputs are borrowed for the call; never cast
a smaller `sockaddr_in`/`sockaddr_in6` allocation to the storage parameter.
Targets require a nonzero port, loopback or guest IP, and zero IPv6 scope/flow
information. See the [C example](../../../sdks/c/README.md#network-tunnels).

```c
AcrylMicrovmErrorCode acrylmicrovm_box_network(
    CBoxHandle* handle,
    CBoxNetworkHandle** out_network,
    CAcrylMicrovmError* out_error
);
void acrylmicrovm_network_free(CBoxNetworkHandle* network);

AcrylMicrovmErrorCode acrylmicrovm_socket_address_parse(
    const char* address,
    struct sockaddr_storage* out_address,
    CAcrylMicrovmError* out_error
);

AcrylMicrovmErrorCode acrylmicrovm_network_tunnel(
    CBoxNetworkHandle* network,
    const struct sockaddr_storage* target,
    const AcrylMicrovmTunnelOptions* options,
    CBoxTunnelHandle** out_tunnel,
    CAcrylMicrovmError* out_error
);
AcrylMicrovmErrorCode acrylmicrovm_network_revoke_tunnel(
    CBoxNetworkHandle* network,
    uint16_t port,
    CAcrylMicrovmError* out_error
);
AcrylMicrovmErrorCode acrylmicrovm_tunnel_uri(
    CBoxTunnelHandle* tunnel,
    char** out_uri,
    CAcrylMicrovmError* out_error
);
AcrylMicrovmErrorCode acrylmicrovm_tunnel_connect(
    CBoxTunnelHandle* tunnel,
    int32_t* out_fd,
    CAcrylMicrovmError* out_error
);
void acrylmicrovm_tunnel_free(CBoxTunnelHandle* tunnel);

AcrylMicrovmErrorCode acrylmicrovm_tunnel_forward(
    CBoxTunnelHandle* tunnel,
    const AcrylMicrovmSocketAddress* listen,
    CTunnelForwarderHandle** out_forwarder,
    CAcrylMicrovmError* out_error
);
AcrylMicrovmErrorCode acrylmicrovm_tunnel_forwarder_address(
    CTunnelForwarderHandle* forwarder,
    char** out_address,
    CAcrylMicrovmError* out_error
);
AcrylMicrovmErrorCode acrylmicrovm_tunnel_forwarder_wait(
    CTunnelForwarderHandle* forwarder,
    CTunnelForwarderWaitCb cb,
    void* user_data,
    CAcrylMicrovmError* out_error
);
AcrylMicrovmErrorCode acrylmicrovm_tunnel_forwarder_close(
    CTunnelForwarderHandle* forwarder,
    CTunnelForwarderCloseCb cb,
    void* user_data,
    CAcrylMicrovmError* out_error
);
void acrylmicrovm_tunnel_forwarder_free(CTunnelForwarderHandle* forwarder);
```

Each `CBoxTunnelHandle` is one-shot. Choose `acrylmicrovm_tunnel_connect()` or
`acrylmicrovm_tunnel_forward()`; either consumes the prepared tunnel.
The target storage and typed options are described above. Read `uri`, `isPublic`,
and `token` with
`acrylmicrovm_tunnel_descriptor()`, before or after connect/forward consumes the tunnel,
freeing the result with `acrylmicrovm_free_string()`.
Only creation returns the full token; later preparations return a mask.
Forwarding opens connections against the existing declaration and cannot undo revocation.
`acrylmicrovm_tunnel_uri()` reports where a remotely served tunnel can be reached, as
an allocated string the caller frees with `acrylmicrovm_free_string()`. It writes NULL
for a local tunnel.

`acrylmicrovm_network_revoke_tunnel()` withdraws the public or private tunnel of `port`. It is
safe to repeat, leaves open connections running, and does nothing for a local
box.

`acrylmicrovm_tunnel_forward()` creates a TCP or Unix listener from the tunnel. Its
wait and close callbacks are posted exactly once through the parent runtime's
drain queue; freeing the caller handle requests non-blocking cancellation while
already accepted operations retain their callback state.
This differs from `acrylmicrovm_options_add_port()`, which creates a persistent,
local-only host listener that accepts repeated connections.

---

### Command execution

#### Live execution inventory and recovery

`acrylmicrovm_box_executions_list(box, callback, user_data, out_error)` queues a
read-only guest query. The callback owns its `CExecutionInfoList`; release it
with `acrylmicrovm_free_execution_info_list`. Each entry contains `execution_id`,
`argv`/`argc`, Unix-millisecond `started_at`, `tty`,
and `is_main_session`. Created/stopped boxes return an empty list without starting;
a paused box returns an invalid-state error.

Use a returned ID with `acrylmicrovm_box_executions_get` for control-only access,
or `acrylmicrovm_box_executions_attach` for stdin/output. Both return an owned
`CExecutionHandle` released with `acrylmicrovm_execution_free`; freeing a recovered
or attached handle does not kill the process. Only one output reader is allowed
per session. Attachment includes available buffered output.
Use `acrylmicrovm_box_executions_attach_read_only` when the client only reads: a
stdin attach that is freed sends the guest a stdin close, which a process
reading stdin sees as EOF.
See the [recovery guide](../../guides/ai-agent-integration.md#recovering-control-after-losing-the-handle)
for completed-session retention.

#### acrylmicrovm_execute

Start a command with optional streaming output and return an execution handle.

```c
typedef struct AcrylMicrovmCommand {
    const char* command;      // Required: command to execute
    const char* const* args;  // Argument array, or NULL
    int argc;                 // Number of entries in args
    const char* const* env_pairs; // [key0, value0, key1, value1, ...], or NULL
    int env_count;            // Number of strings in env_pairs
    const char* workdir;      // Working directory, or NULL
    const char* user;         // User spec (e.g., "nobody", "1000:1000"), or NULL
    double timeout_secs;      // Timeout in seconds (0.0 = no timeout)
    int tty;                  // 0 = no TTY, non-zero = TTY
} AcrylMicrovmCommand;

AcrylMicrovmErrorCode acrylmicrovm_execute(
    CBoxHandle* handle,
    const AcrylMicrovmCommand* cmd,
    void (*callback)(const char* text, int is_stderr, void* user_data),
    void* user_data,
    CExecutionHandle** out_execution,
    CAcrylMicrovmError* out_error
);
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `handle` | `CBoxHandle*` | Box handle |
| `cmd` | `const AcrylMicrovmCommand*` | Command descriptor |
| `callback` | function pointer | Optional streaming output callback |
| `user_data` | `void*` | User data passed to callback |
| `out_execution` | `CExecutionHandle**` | Output: execution handle |
| `out_error` | `CAcrylMicrovmError*` | Output: error information |

#### Callback signature

```c
void callback(const char* text, int is_stderr, void* user_data);
```

| Parameter | Description |
|-----------|-------------|
| `text` | Output text chunk |
| `is_stderr` | `0` for stdout, `1` for stderr |
| `user_data` | User data from `acrylmicrovm_execute` |

#### Example

```c
void output_handler(const char* text, int is_stderr, void* data) {
    FILE* stream = is_stderr ? stderr : stdout;
    fprintf(stream, "%s", text);
}

int exit_code = 0;
const char* args[] = {"-c", "print('hello')"};
AcrylMicrovmCommand cmd = {.command = "python", .args = args, .argc = 2};
CExecutionHandle* execution = NULL;
AcrylMicrovmErrorCode code = acrylmicrovm_execute(
    box,
    &cmd,
    output_handler,
    NULL,
    &execution,
    &error
);

if (code == Ok) {
    code = acrylmicrovm_execution_wait(execution, &exit_code, &error);
    acrylmicrovm_execution_free(execution);
}
if (code == Ok) {
    printf("Exit code: %d\n", exit_code);
}
```

---

#### Execution control

```c
AcrylMicrovmErrorCode acrylmicrovm_execution_write(CExecutionHandle* execution, const char* data, int len, CAcrylMicrovmError* out_error);
AcrylMicrovmErrorCode acrylmicrovm_execution_wait(CExecutionHandle* execution, int* out_exit_code, CAcrylMicrovmError* out_error);
AcrylMicrovmErrorCode acrylmicrovm_execution_kill(CExecutionHandle* execution, CAcrylMicrovmError* out_error);
AcrylMicrovmErrorCode acrylmicrovm_execution_resize_tty(CExecutionHandle* execution, int rows, int cols, CAcrylMicrovmError* out_error);
void acrylmicrovm_execution_free(CExecutionHandle* execution);
```

#### Example: command options

```c
const char* args[] = {"-c", "import os; print(os.getcwd())"};
const char* env[] = {"MY_VAR", "hello"};
AcrylMicrovmCommand cmd = {
    .command = "python",
    .args = args,
    .argc = 2,
    .env_pairs = env,
    .env_count = 2,
    .workdir = "/tmp",
    .user = "nobody",
    .timeout_secs = 30.0,
};

int exit_code = 0;
CExecutionHandle* execution = NULL;
AcrylMicrovmErrorCode code =
    acrylmicrovm_execute(box, &cmd, output_handler, NULL, &execution, &error);
if (code == Ok) {
    code = acrylmicrovm_execution_wait(execution, &exit_code, &error);
    acrylmicrovm_execution_free(execution);
}
```

---

### Discovery & introspection

#### acrylmicrovm_list_info

List all boxes.

```c
AcrylMicrovmErrorCode acrylmicrovm_list_info(
    CAcrylMicrovmRuntime* runtime,
    CBoxInfoListCb callback,
    void* user_data,
    CAcrylMicrovmError* out_error
);
```

`Ok` means the request was queued. The callback runs later when the caller
invokes `acrylmicrovm_runtime_drain()`. On success, the callback owns the non-NULL
list and must free it with `acrylmicrovm_free_box_info_list()`.

---

#### acrylmicrovm_get_info

Get single box info by ID or name.

```c
AcrylMicrovmErrorCode acrylmicrovm_get_info(
    CAcrylMicrovmRuntime* runtime,
    const char* id_or_name,
    CBoxInfoCb callback,
    void* user_data,
    CAcrylMicrovmError* out_error
);
```

---

#### acrylmicrovm_box_info

Get box info from handle.

```c
AcrylMicrovmErrorCode acrylmicrovm_box_info(
    CBoxHandle* handle,
    CBoxInfoCb callback,
    void* user_data,
    CAcrylMicrovmError* out_error
);
```

All three info functions use the same post-and-drain contract. `out_error`
reports only synchronous queueing failures. The error passed to a callback is
borrowed and valid only during that callback; do not call
`acrylmicrovm_error_free()` on it. `user_data` must remain valid until the callback
runs.

```c
typedef struct {
    int done;
} InfoRequest;

static void on_box_info(CBoxInfo* info, CAcrylMicrovmError* error, void* user_data) {
    InfoRequest* request = user_data;
    if (error->code != Ok) {
        fprintf(stderr, "info failed: %s\n",
                error->message ? error->message : "unknown error");
    } else {
        // The callback owns info on success.
        printf("Box %s status: %s\n", info->id, info->status);
        acrylmicrovm_free_box_info(info);
    }
    request->done = 1;
}

InfoRequest request = {0};
CAcrylMicrovmError error = {0};
if (acrylmicrovm_box_info(box, on_box_info, &request, &error) == Ok) {
    while (!request.done) {
        if (acrylmicrovm_runtime_drain(runtime, -1, &error) < 0) {
            fprintf(stderr, "drain failed: %s\n",
                    error.message ? error.message : "unknown error");
            acrylmicrovm_error_free(&error);
            break;
        }
    }
} else {
    fprintf(stderr, "queueing info failed: %s\n",
            error.message ? error.message : "unknown error");
    acrylmicrovm_error_free(&error);
}
```

A list callback has the equivalent ownership rule:

```c
static void on_box_list(CBoxInfoList* list, CAcrylMicrovmError* error,
                        void* user_data) {
    InfoRequest* request = user_data;
    if (error->code == Ok) {
        for (int i = 0; i < list->count; ++i) {
            printf("%s\n", list->items[i].id);
        }
        acrylmicrovm_free_box_info_list(list);
    }
    request->done = 1;
}

InfoRequest list_request = {0};
if (acrylmicrovm_list_info(runtime, on_box_list, &list_request, &error) == Ok) {
    while (!list_request.done) {
        if (acrylmicrovm_runtime_drain(runtime, -1, &error) < 0) {
            acrylmicrovm_error_free(&error);
            break;
        }
    }
} else {
    acrylmicrovm_error_free(&error);
}
```

`info->network` is an owned `CNetworkInfo*` freed with the rest of `CBoxInfo`.
It is `NULL` when network information is unavailable. Otherwise it contains a
`COutboundNetworkInfo`/`CInboundNetworkInfo` per direction (`outbound`, `inbound`), each with a
typed `mode` plus an `allow_net` string array and count, and a nullable
`CPublishedPortList*`. The deprecated top-level `mode`, `allow_net` and
`allow_net_count` fields keep their pre-split offsets and alias `outbound`. A `NULL` `published_ports` pointer means the current
handle does not know the lifecycle's publications, a non-`NULL` list with
`count == 0` means there are no active local publications, and a populated list
contains typed `CPublishedPort` entries with `guest_port`, `host_ip`,
`host_port`, and `protocol` fields. These nested values are borrowed from
`CBoxInfo` and must not be freed separately.

Because info resolution is asynchronous, the implementation may perform backend
I/O before posting the callback. Keep the runtime and any callback `user_data`
alive until the completion has been drained.

---

### Images

`acrylmicrovm_runtime_images()` returns a `CAcrylMicrovmImageHandle*` for the images a
runtime can boot from: the local cache on a runtime from
`acrylmicrovm_runtime_new()`, the server's catalog on one from
`acrylmicrovm_rest_runtime_new_with_options()`. List, get and remove run unchanged
against either; pull is local only and usage REST only. Free the handle with
`acrylmicrovm_image_free()`.

| Function | Local runtime | REST runtime |
|----------|---------------|--------------|
| `acrylmicrovm_image_pull()` | Pulls into the cache | `Unsupported`: creating a box pulls |
| `acrylmicrovm_image_list()` | One row per cached reference | Catalog alias and digest references |
| `acrylmicrovm_image_get()` | The cache entries under the name | The catalog entry with its versions |
| `acrylmicrovm_image_remove()` | Forgets the name; layers stay | Removes the catalog entry; `InvalidState` while a box can boot from it |
| `acrylmicrovm_image_usage()` | `Unsupported` | `count`, `limit`, `known_bytes` |
| `acrylmicrovm_image_create_alias()` | `Unsupported` | Creates a new alias; existing names conflict |
| `acrylmicrovm_image_delete_alias()` | `Unsupported` | Removes only the named alias |

```c
AcrylMicrovmErrorCode acrylmicrovm_image_get(
    CAcrylMicrovmImageHandle* handle,
    const char* name,
    CBoxImageGetCb cb,          // void (*)(CImageDetail*, CAcrylMicrovmError*, void*)
    void* user_data,
    CAcrylMicrovmError* out_error
);

AcrylMicrovmErrorCode acrylmicrovm_image_remove(
    CAcrylMicrovmImageHandle* handle,
    const char* name,
    CBoxImageRemoveCb cb,       // void (*)(CAcrylMicrovmError*, void*)
    void* user_data,
    CAcrylMicrovmError* out_error
);

AcrylMicrovmErrorCode acrylmicrovm_image_usage(
    CAcrylMicrovmImageHandle* handle,
    CBoxImageUsageCb cb,        // void (*)(CImageUsage*, CAcrylMicrovmError*, void*)
    void* user_data,
    CAcrylMicrovmError* out_error
);

AcrylMicrovmErrorCode acrylmicrovm_image_create_alias(
    CAcrylMicrovmImageHandle* handle,
    const char* name,
    const char* alias,
    const char* digest,         // "sha256:" and 64 lowercase hex digits
    CBoxImageGetCb cb,          // the image as it now is, as for a get
    void* user_data,
    CAcrylMicrovmError* out_error
);
```

These operations follow the post-and-drain contract of the info functions: `Ok`
means queued, and the callback runs in `acrylmicrovm_runtime_drain()`. `name` is
an image name such as `"docker.io/library/alpine"`, copied before the call
returns. A name the runtime does not hold reaches the callback as `NotFound`.
A reference with a tag or digest (`"quay.io/acme/app:v1"`) reaches it as
`InvalidArgument`, since a remove takes every alias of the name. Locally, a box
built from a removed image reads the image's configuration from the registry
when it next starts. `acrylmicrovm_image_create_alias()` copies `alias` and `digest` as it
does `name`; an alias that does not use OCI tag syntax or a malformed digest
reaches the callback as `InvalidArgument` before anything is sent.

Alias creation returns `AlreadyExists` through the callback for every existing
name, even when its digest matches. Explicit deletion uses the image-remove
callback shape and removes no versions:

```c
AcrylMicrovmErrorCode acrylmicrovm_image_delete_alias(
    CAcrylMicrovmImageHandle* handle, const char* name, const char* alias,
    CBoxImageRemoveCb cb, void* user_data, CAcrylMicrovmError* out_error
);
```

A catalog fills every field below; a local cache fills a version's `aliases` and
leaves its optional times unset and `history` empty.

| Type | Fields | Ownership |
|------|--------|-----------|
| `CImageDetail` | `name`, `aliases`/`aliases_count`, `curated` (non-zero for operator-provided images), `versions`/`versions_count` (newest first), `history`/`history_count` (alias registration history, newest first) | The callback owns it; free with `acrylmicrovm_free_image_detail()` |
| `CImageVersion` | `digest`, `size_bytes` (valid when `has_size` is non-zero), `source_ref`, `recorded_at` (Unix seconds), `aliases`/`aliases_count`, `last_used_at` and `upstream_missing_at` (Unix seconds, valid when `has_last_used_at` / `has_upstream_missing_at` is non-zero) | Owned by its `CImageDetail` |
| `CImageAliasEvent` | `alias`, `from_digest` (NULL for alias creation), `to_digest`, `actor` (NULL when a pull seeded the alias), `recorded_at` (Unix seconds) | Owned by its `CImageDetail` |
| `CImageUsage` | `count`, `limit`, `known_bytes` | Borrowed for the callback; nothing to free |

```c
static void on_detail(CImageDetail* detail, CAcrylMicrovmError* error,
                      void* user_data) {
    int* done = user_data;
    if (error->code == Ok) {
        for (int i = 0; i < detail->versions_count; ++i) {
            printf("%s %s\n", detail->name, detail->versions[i].digest);
        }
        acrylmicrovm_free_image_detail(detail);
    } else {
        fprintf(stderr, "get failed: %s\n", error->message);
    }
    *done = 1;
}

CAcrylMicrovmImageHandle* images = NULL;
if (acrylmicrovm_runtime_images(runtime, &images, &error) == Ok) {
    int done = 0;
    if (acrylmicrovm_image_get(images, "docker.io/library/alpine", on_detail,
                          &done, &error) == Ok) {
        while (!done && acrylmicrovm_runtime_drain(runtime, -1, &error) >= 0) {
        }
    }
    acrylmicrovm_image_free(images);
}
if (error.code != Ok) {
    acrylmicrovm_error_free(&error);
}
```

---

### Registries

`acrylmicrovm_image_registries()` returns a `CAcrylMicrovmRegistryHandle*` for the
logins a REST server presents when it pulls a private image for a box. A
runtime from `acrylmicrovm_runtime_new()` answers `Unsupported`: it pulls with the
logins in its `image_registries` option instead. Free the handle with
`acrylmicrovm_registry_free()`.

```c
AcrylMicrovmErrorCode acrylmicrovm_registry_list(
    CAcrylMicrovmRegistryHandle* handle,
    CBoxRegistryListCb cb,      // void (*)(CRegistryCredentialList*, CAcrylMicrovmError*, void*)
    void* user_data,
    CAcrylMicrovmError* out_error
);

AcrylMicrovmErrorCode acrylmicrovm_registry_create(
    CAcrylMicrovmRegistryHandle* handle,
    const char* registry_host,      // e.g. "ghcr.io"
    const char* repository_prefix,  // e.g. "acme/"; NULL for the whole registry
    const char* username,
    const char* password,
    CBoxRegistryCreateCb cb,    // void (*)(CRegistryCredential*, CAcrylMicrovmError*, void*)
    void* user_data,
    CAcrylMicrovmError* out_error
);

AcrylMicrovmErrorCode acrylmicrovm_registry_remove(
    CAcrylMicrovmRegistryHandle* handle,
    const char* id,
    CBoxRegistryRemoveCb cb,    // void (*)(CAcrylMicrovmError*, void*)
    void* user_data,
    CAcrylMicrovmError* out_error
);
```

All three follow the post-and-drain contract of the image functions. The
strings are copied before the call returns, and the password is sent once and
never returned. The callback receives `AlreadyExists` for a second login for
the same registry and prefix, `InvalidState` (naming the boxes) for removing a
login a box still pulls through, and `NotFound` for an unknown id. An id that
is not a UUID reaches it as `InvalidArgument` without a request.

| Type | Fields | Ownership |
|------|--------|-----------|
| `CRegistryCredential` | `id`, `registry_host`, `repository_prefix` (empty for the whole registry), `username`, `created_by` (NULL when unknown), `created_at` (Unix seconds); no password | From a create callback, owned; free with `acrylmicrovm_free_registry_credential()` |
| `CRegistryCredentialList` | `items`/`count` | The callback owns it; free with `acrylmicrovm_free_registry_credential_list()` |

Registry strings are non-null, except `created_by` when unknown. If any
server-returned field contains an interior NUL byte, create and list callbacks
receive `Internal` and no result. The SDK rejects the response without changing
credential IDs; list conversion frees any entries already converted.

---

### Guest SSH control

These operations take a `CBoxHandle*` and follow the existing Native API
callback convention.

| Function | Additional input | Callback |
|----------|------------------|----------|
| `acrylmicrovm_box_ssh_configure` | `const char* config_json` | `CBoxSshConfigureCb` |
| `acrylmicrovm_box_ssh_enable` | None | `CBoxSshEnableCb` |
| `acrylmicrovm_box_ssh_status` | None | `CBoxSshStatusCb` |
| `acrylmicrovm_box_ssh_disable` | None | `CBoxSshDisableCb` |

Enable requires saved configuration and preserves healthy connections. Enable
and disable work locally and over REST, require a running box, and never start
it. A valid handle to a configured or stopped box fails the callback with
`InvalidState` (HTTP 409 over REST), without changing saved SSH intent. Disable is
safe to repeat while running; call it while running to persist the disabled switch.

All four callbacks receive `CSshStatus*`. Its corresponding release function
is `acrylmicrovm_free_ssh_status()`; the existing [memory management rules](#memory-management)
apply.

| `CSshStatus` field | C type |
|-------------------|--------|
| `enabled` | `int` (non-zero when enabled) |
| `generation` | `uint64_t` |
| `listen_address` | `char*` |
| `host_public_key` | `char*` |
| `host_key_fingerprint` | `char*` |

`config_json` must be UTF-8 JSON and only needs to remain valid for the call.
Malformed JSON or non-UTF-8 input returns `InvalidArgument` with
`invalid SSH configuration`, without echoing rejected values. Operation
failures are reported through the callback.

The configuration fields are `listen_address`, `host_private_key`, and
`accounts` of `login`, `authorized_keys`, and optional `ca` with `public_key`
and `principal`.

---

### Metrics

#### acrylmicrovm_runtime_metrics

Get runtime-wide metrics.

```c
AcrylMicrovmErrorCode acrylmicrovm_runtime_metrics(
    CAcrylMicrovmRuntime* runtime,
    CRuntimeMetrics* out_metrics,
    CAcrylMicrovmError* out_error
);
```

---

#### acrylmicrovm_box_metrics

Get per-box metrics.

```c
AcrylMicrovmErrorCode acrylmicrovm_box_metrics(
    CBoxHandle* handle,
    CBoxMetrics* out_metrics,
    CAcrylMicrovmError* out_error
);
```

---

## Memory management

### Rules

1. **All allocated strings must be freed**
   - `acrylmicrovm_box_id()` → `acrylmicrovm_free_string()`

2. **Error structs must be freed**
   - Caller-owned `CAcrylMicrovmError` output → `acrylmicrovm_error_free()`
   - Callback error pointers are borrowed and must not be freed

3. **Results must be freed**
   - `CAcrylMicrovmExecResult` → `acrylmicrovm_result_free()`
   - `CBoxInfo` → `acrylmicrovm_free_box_info()`
   - `CBoxInfoList` → `acrylmicrovm_free_box_info_list()`
   - `CImagePullResult` → `acrylmicrovm_free_image_pull_result()`
   - `CImageInfoList` → `acrylmicrovm_free_image_info_list()`
   - `CImageDetail` → `acrylmicrovm_free_image_detail()`

4. **All cleanup functions are NULL-safe**

### Functions

#### acrylmicrovm_free_string

Free a string allocated by ACRYL microVMs.

```c
void acrylmicrovm_free_string(char* str);
```

---

#### acrylmicrovm_error_free

Free error struct (message only - struct itself is stack-allocated).

```c
void acrylmicrovm_error_free(CAcrylMicrovmError* error);
```

---

#### acrylmicrovm_box_free

Free a box handle.

```c
void acrylmicrovm_box_free(CBoxHandle* handle);
```

Safe to call with NULL.

---

## Thread safety

| Component | Thread Safety |
|-----------|---------------|
| `CAcrylMicrovmRuntime` | Thread-safe |
| `CBoxHandle` | **NOT** thread-safe - do not share across threads |
| `CAcrylMicrovmSimple` | **NOT** thread-safe - do not share across threads |
| Callbacks | Invoked on the thread calling `acrylmicrovm_runtime_drain()` |

### Safe multi-threaded usage

```c
// CORRECT: Share runtime, create per-thread boxes
void* thread_func(void* arg) {
    CAcrylMicrovmRuntime* runtime = (CAcrylMicrovmRuntime*)arg;
    CAcrylMicrovmError error = {0};
    CAcrylMicrovmOptions* opts = NULL;
    CBoxHandle* box = NULL;

    // Each thread creates its own box
    acrylmicrovm_options_new("alpine:3.19", &opts, &error);
    acrylmicrovm_create_box(runtime, opts, &box, &error);
    acrylmicrovm_options_free(opts);
    // Use box in this thread only
    acrylmicrovm_stop_box(box, &error);
    return NULL;
}

CAcrylMicrovmRuntime* runtime;
acrylmicrovm_runtime_new(NULL, NULL, 0, &runtime, &error);

pthread_t threads[4];
for (int i = 0; i < 4; i++) {
    pthread_create(&threads[i], NULL, thread_func, runtime);
}
```

---

## Platform requirements

| Platform | Architecture | Status | Requirements |
|----------|-------------|--------|--------------|
| macOS | ARM64 (Apple Silicon) | Supported | macOS 11.0+, Hypervisor.framework |
| macOS | x86_64 (Intel) | **Not supported** | N/A |
| Linux | x86_64 | Supported | KVM enabled |
| Linux | ARM64 (aarch64) | Supported | KVM enabled |
| Windows | Any | Via WSL2 | WSL2 with KVM |

---

## Migration from v0.1.x

### Error handling change

**v0.1.x (old):**
```c
char* error = NULL;
CAcrylMicrovmRuntime* runtime = acrylmicrovm_runtime_new(NULL, &error);
if (!runtime) {
    fprintf(stderr, "Error: %s\n", error);
    acrylmicrovm_free_string(error);
    return 1;
}
```

**v0.2.0 (new):**
```c
CAcrylMicrovmRuntime* runtime = NULL;
CAcrylMicrovmError error = {0};
AcrylMicrovmErrorCode code = acrylmicrovm_runtime_new(NULL, NULL, 0, &runtime, &error);
if (code != Ok) {
    fprintf(stderr, "Error %d: %s\n", error.code, error.message);
    acrylmicrovm_error_free(&error);
    return 1;
}
```

### Execute change

**v0.1.x:**
```c
int exit_code = old_execute_api_returning_exit_code(...);
if (exit_code < 0) {
    // Error
}
```

**v0.2.0:**
```c
int exit_code = 0;
const char* args[] = {"hello"};
AcrylMicrovmCommand cmd = {.command = "echo", .args = args, .argc = 1};
CExecutionHandle* execution = NULL;
AcrylMicrovmErrorCode code =
    acrylmicrovm_execute(box, &cmd, callback, NULL, &execution, &error);
if (code == Ok) {
    code = acrylmicrovm_execution_wait(execution, &exit_code, &error);
    acrylmicrovm_execution_free(execution);
}
if (code != Ok) {
    // Error
}
```

### Migration checklist

- [ ] Replace `char* error = NULL` with `CAcrylMicrovmError error = {0}`
- [ ] Initialize output pointers to NULL (e.g., `CAcrylMicrovmRuntime* runtime = NULL`)
- [ ] Use output parameters for synchronous calls and callbacks for
      asynchronous calls
- [ ] Replace return value checks with `AcrylMicrovmErrorCode` checks
- [ ] Replace `acrylmicrovm_free_string(error)` with `acrylmicrovm_error_free(&error)`
- [ ] Create boxes with `CAcrylMicrovmOptions`

---

## API summary

| Function | Description |
|----------|-------------|
| `acrylmicrovm_version()` | Get version string |
| `acrylmicrovm_runtime_new()` | Create runtime |
| `acrylmicrovm_runtime_shutdown()` | Graceful shutdown |
| `acrylmicrovm_runtime_free()` | Free runtime |
| `acrylmicrovm_runtime_metrics()` | Get runtime metrics |
| `acrylmicrovm_create_box()` | Create box |
| `acrylmicrovm_start_box()` | Start/restart box |
| `acrylmicrovm_stop_box()` | Stop box |
| `acrylmicrovm_remove()` | Remove box |
| `acrylmicrovm_get()` | Reattach to box |
| `acrylmicrovm_box_id()` | Get box ID |
| `acrylmicrovm_box_free()` | Free box handle |
| `acrylmicrovm_box_info()` | Queue box info lookup |
| `acrylmicrovm_box_metrics()` | Get box metrics |
| `acrylmicrovm_execute()` | Execute command |
| `acrylmicrovm_list_info()` | Queue box list lookup |
| `acrylmicrovm_get_info()` | Queue box info lookup by ID |
| `acrylmicrovm_runtime_images()` | Get the image handle |
| `acrylmicrovm_image_pull()` | Queue an image pull (local runtimes) |
| `acrylmicrovm_image_list()` | Queue an image list |
| `acrylmicrovm_image_get()` | Queue a read of an image name's builds |
| `acrylmicrovm_image_remove()` | Queue removal of an image name |
| `acrylmicrovm_image_usage()` | Queue a read of the image allowance (REST runtimes) |
| `acrylmicrovm_image_create_alias()` | Create a catalog alias; existing names conflict (REST runtimes) |
| `acrylmicrovm_image_delete_alias()` | Delete one catalog alias (REST runtimes) |
| `acrylmicrovm_image_free()` | Free the image handle |
| `acrylmicrovm_free_image_detail()` | Free an image detail |
| `acrylmicrovm_image_registries()` | Get the registry login handle (REST runtimes) |
| `acrylmicrovm_registry_list()` | Queue a list of registry logins |
| `acrylmicrovm_registry_create()` | Queue adding a registry login |
| `acrylmicrovm_registry_remove()` | Queue removing a registry login |
| `acrylmicrovm_registry_free()` | Free the registry login handle |
| `acrylmicrovm_free_registry_credential()` | Free a registry login |
| `acrylmicrovm_free_registry_credential_list()` | Free a registry login list |
| `acrylmicrovm_simple_new()` | Create simple box |
| `acrylmicrovm_simple_run()` | Run command (simple) |
| `acrylmicrovm_simple_free()` | Free simple box |
| `acrylmicrovm_result_free()` | Free exec result |
| `acrylmicrovm_free_string()` | Free string |
| `acrylmicrovm_error_free()` | Free error |

---

## Common patterns

### Streaming output

```c
void output_callback(const char* text, int is_stderr, void* user_data) {
    FILE* stream = is_stderr ? stderr : stdout;
    fprintf(stream, "%s", text);
}

int exit_code = 0;
const char* args[] = {"-c", "print('hello')"};
AcrylMicrovmCommand cmd = {.command = "python", .args = args, .argc = 2};
CExecutionHandle* execution = NULL;
if (acrylmicrovm_execute(box, &cmd, output_callback, NULL, &execution, &error) == Ok) {
    acrylmicrovm_execution_wait(execution, &exit_code, &error);
    acrylmicrovm_execution_free(execution);
}
```

### Reattach to box

```c
// Get box ID
char* box_id = acrylmicrovm_box_id(box);

// Later, in different process:
CBoxHandle* box2 = NULL;
acrylmicrovm_get(runtime, box_id, &box2, &error);

acrylmicrovm_free_string(box_id);
```

### Get box info

```c
InfoRequest request = {0};
AcrylMicrovmErrorCode code =
    acrylmicrovm_box_info(box, on_box_info, &request, &error);
if (code == Ok) {
    while (!request.done) {
        if (acrylmicrovm_runtime_drain(runtime, -1, &error) < 0) {
            acrylmicrovm_error_free(&error);
            break;
        }
    }
}
```

---

## Common mistakes

### Uninitialized error struct

```c
CAcrylMicrovmError error;       // Wrong: uninitialized
CAcrylMicrovmError error = {0}; // Correct: zero-initialized
```

### Forgetting to free error

```c
if (code != Ok) {
    printf("Error: %s\n", error.message);
    return 1;                       // Wrong: memory leak
}

if (code != Ok) {
    printf("Error: %s\n", error.message);
    acrylmicrovm_error_free(&error);     // Correct
    return 1;
}
```

### Forgetting to free result structs

```c
static void leaking_list_callback(CBoxInfoList* list, CAcrylMicrovmError* error,
                                  void* user_data) {
    // Wrong: successful list ownership was transferred here but never freed.
}

static void list_callback(CBoxInfoList* list, CAcrylMicrovmError* error,
                          void* user_data) {
    if (error->code == Ok) {
        acrylmicrovm_free_box_info_list(list);  // Correct
    }
}
```

---

## Build & link

### CMake

```cmake
cmake_minimum_required(VERSION 3.15)
project(my_app)

set(ACRYLMICROVM_INCLUDE "/path/to/acrylmicrovm/sdks/c/include")
set(ACRYLMICROVM_LIB_DIR "/path/to/acrylmicrovm/target/release")

include_directories(${ACRYLMICROVM_INCLUDE})

add_executable(my_app main.c)
target_link_libraries(my_app ${ACRYLMICROVM_LIB_DIR}/libacrylmicrovm.dylib)
```

### Direct compilation

```bash
# macOS
gcc -o myapp myapp.c \
    -I/path/to/acrylmicrovm/sdks/c/include \
    -L/path/to/acrylmicrovm/target/release \
    -lacrylmicrovm

export DYLD_LIBRARY_PATH=/path/to/acrylmicrovm/target/release:$DYLD_LIBRARY_PATH
./myapp

# Linux
gcc -o myapp myapp.c \
    -I/path/to/acrylmicrovm/sdks/c/include \
    -L/path/to/acrylmicrovm/target/release \
    -lacrylmicrovm

export LD_LIBRARY_PATH=/path/to/acrylmicrovm/target/release:$LD_LIBRARY_PATH
./myapp
```

---

## See also

- **[C SDK README](../../../sdks/c/README.md)** - Full SDK documentation
- **[C Quick Start](../../getting-started/quickstart-c.md)** - 5-minute guide
- **[C Examples](../../../examples/c/)** - Working examples
- **[Concepts](../../concepts/README.md)** - How ACRYL microVMs works

### CPU ABI compatibility

CPU arguments and `CBoxInfo.cpus` use `double`; rebuild callers with the matching header and library.
Integer allocations preserve existing behavior; fractional allocations require a Linux execution host and round guest vCPUs upward.
See [fractional CPUs](../../guides/resource-limits.md#fractional-cpus-linux) for enforcement and error semantics.
