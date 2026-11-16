# ACRYL microVMs C SDK

C bindings for the ACRYL microVMs runtime, providing a stable C API for integrating ACRYL microVMs into C/C++ applications.

The [agent API](../../docs/reference/agents.md#c-api-and-ownership) covers
Box-owned sessions, turns, callbacks and handle ownership.

**C Standard:** C11-compatible compiler (GCC/Clang)

**Upgrade notice:** existing local boxes with secrets must be recreated with explicit
`headers`, including boxes managed by `acrylmicrovm serve`. See the
[required-headers migration notes](../../docs/reference/configuration.md#secrets).

## Managed checkpoints

Execution setters are rejected even when they set defaults, such as
`acrylmicrovm_options_set_detach(opts, 0)`, before or after checkpoint selection.
Null, empty or invalid UTF-8 checkpoint input returns `InvalidArgument` from
create; it never falls back to the constructor's image. A synchronous refusal
leaves the options handle owned by the caller.

A checkpoint is a source for a new box, like an image: pass it to the ordinary
create call. The checkpoint handle only manages checkpoints. Archive import is a
separate operation for local `.acrylmicrovm` files and is unchanged. The CLI form is
`acrylmicrovm create --checkpoint ID [--name NAME]`.

The source must be stopped before checkpoint creation; stop it and wait for `stopped` first.
Checkpoint creation rejects other states and never stops the source automatically.

`acrylmicrovm_checkpoint_create`, `list`, `get` and `remove` require a
REST runtime and use its existing post-and-drain event queue. Local runtimes
deliver an unsupported-operation error through the same callback queue.
To create a Box from a checkpoint, call `acrylmicrovm_options_set_checkpoint` on the
options and pass them to `acrylmicrovm_create_box`. Only the name may accompany it.
Input strings are copied before return. Metadata callbacks own `CCheckpointInfo`
or `CCheckpointList`; release them with `acrylmicrovm_free_checkpoint_info` or
`acrylmicrovm_free_checkpoint_list`. List entries belong to the list. Error
pointers are borrowed during callback dispatch.

Cloud capture may initially report `creating`; wait for `ready` before use.
A Box created from a checkpoint inherits all saved execution settings; any other
option fails the create with an invalid-argument error.
Force removal ignores absence and never bypasses active use.
Storage belongs to the server. Existing native archive
export/import remains available.
`acrylmicrovm_box_name` returns an owned name string, released with
`acrylmicrovm_free_string`, or null for an unnamed Box.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Overview](#api-overview)
  - [Simple API](#simple-api)
  - [Native API](#native-api)
    - [Network Tunnels](#network-tunnels)
  - [Error Handling](#error-handling)
- [Complete API Reference](#complete-api-reference)
- [Examples](#examples)
- [Memory Management](#memory-management)
- [Threading & Safety](#threading--safety)
- [Platform Support](#platform-support)
- [Migration Guide](#migration-guide)
- [Troubleshooting](#troubleshooting)
- [Architecture](#architecture)
- [Links](#links)

---

## Overview

The C SDK provides two API styles:

1. **Simple API** (`acrylmicrovm_simple_*`) - Convenience layer for common use cases
   - No runtime setup required
   - Auto-managed runtime
   - Buffered command results
   - Automatic cleanup

2. **Native API** (`acrylmicrovm_*`) - Full-featured, flexible interface
   - Typed `CAcrylMicrovmOptions` configuration
   - Streaming output callbacks
   - Fine-grained control
   - Advanced features (volumes, networking, etc.)

Both APIs support:
- ✅ Structured error handling (error codes + messages)
- ✅ OCI container images
- ✅ Hardware-accelerated VMs (KVM/Hypervisor.framework)
- ✅ Command execution with streaming output
- ✅ Box lifecycle management
- ✅ Performance metrics
- ✅ Multi-box management

---

## Features

Secrets for outbound HTTPS can use `acrylmicrovm_options_add_secret_with_headers()`.
Pass parallel header-name and template arrays, such as `"x-goog-api-key"` and
`"{value}"`. At least one entry is required; zero entries are rejected at creation. Templates must
contain exactly one `{value}`; empty strings are rejected at box creation.
The existing `acrylmicrovm_options_add_secret()` remains ABI-compatible, but its
headerless secret is rejected at creation. Use the plural function above.

### Core Features
- **C-compatible FFI bindings** (`cdylib`, `staticlib`)
- **Auto-generated header file** (`include/acrylmicrovm.h`)
- **Structured error handling** - Error codes + detailed messages
- **Simple convenience API** - Auto-cleanup
- **Streaming output support** - Real-time callbacks
- **Typed configuration** - Builder-style options and typed result structs

### Advanced Features
- **Box lifecycle management** - Create, start, stop, restart, remove
- **Persistent boxes** - Cross-process reattachment
- **Performance metrics** - Runtime and per-box statistics
- **Multiple boxes** - Concurrent container management
- **Prefix lookup** - Find boxes by ID prefix

---

## Installation

### Prerequisites

**macOS:**
- Apple Silicon (ARM64) or Intel x86_64
- macOS 11.0+ (Big Sur or later)
- Xcode Command Line Tools

**Linux:**
- x86_64 or ARM64 architecture
- KVM support (check: `kvm-ok` or `lsmod | grep kvm`)
- GCC or Clang

### Building from Source

```bash
# From repository root
git clone https://github.com/acrylmicrovm/acrylmicrovm.git
cd acrylmicrovm

# Initialize submodules (REQUIRED!)
git submodule update --init --recursive

# Build C SDK
cargo build --release -p acrylmicrovm-c

# Outputs:
# - target/release/libacrylmicrovm.{dylib,so}     (shared library)
# - target/release/libacrylmicrovm.a              (static library)
# - sdks/c/include/acrylmicrovm.h                 (auto-generated header)
```

### Option 1: Direct Linking (Development)

```bash
# Copy library and header to your project
cp target/release/libacrylmicrovm.{dylib,so} /path/to/your/project/lib/
cp sdks/c/include/acrylmicrovm.h /path/to/your/project/include/

# Compile your program
gcc -I/path/to/include -L/path/to/lib -lacrylmicrovm your_program.c -o your_program

# macOS: Set runtime library path
install_name_tool -add_rpath /path/to/lib your_program

# Linux: Set LD_LIBRARY_PATH
export LD_LIBRARY_PATH=/path/to/lib:$LD_LIBRARY_PATH
./your_program
```

### Option 2: CMake (Recommended)

See `examples/c/CMakeLists.txt` for a complete example.

```cmake
cmake_minimum_required(VERSION 3.15)
project(my_acrylmicrovm_app C)

set(ACRYLMICROVM_ROOT "/path/to/acrylmicrovm")
set(ACRYLMICROVM_INCLUDE_DIR "${ACRYLMICROVM_ROOT}/sdks/c/include")
set(ACRYLMICROVM_LIB_DIR "${ACRYLMICROVM_ROOT}/target/release")

include_directories(${ACRYLMICROVM_INCLUDE_DIR})

add_executable(my_app main.c)
target_link_libraries(my_app ${ACRYLMICROVM_LIB_DIR}/libacrylmicrovm.dylib)

# Set RPATH
if(APPLE)
    set_target_properties(my_app PROPERTIES
        BUILD_RPATH "${ACRYLMICROVM_LIB_DIR}"
    )
endif()
```

---

## Quick Start

### Simple API (Recommended for Most Use Cases)

```c
#include <stdio.h>
#include "acrylmicrovm.h"

int main() {
    // Create a box with no runtime management
    CAcrylMicrovmSimple* box;
    CAcrylMicrovmError error = {0};

    if (acrylmicrovm_simple_new("python:slim", 0, 0, &box, &error) != Ok) {
        fprintf(stderr, "Error %d: %s\n", error.code, error.message);
        acrylmicrovm_error_free(&error);
        return 1;
    }

    // Run a command
    const char* args[] = {"-c", "print('Hello!')", NULL};
    CAcrylMicrovmExecResult* result;

    if (acrylmicrovm_simple_run(box, "python", args, 2, &result, &error) == Ok) {
        printf("Output: %s\n", result->stdout_text);
        printf("Exit code: %d\n", result->exit_code);
        acrylmicrovm_result_free(result);
    }

    // Cleanup (auto-stop and remove)
    acrylmicrovm_simple_free(box);
    return 0;
}
```

### Native API (For Advanced Use Cases)

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
    acrylmicrovm_options_set_network_inbound_disabled(opts);

    CAdvancedBoxOptions* advanced = NULL;
    if (acrylmicrovm_advanced_options_new(&advanced, &error) != Ok) {
        acrylmicrovm_options_free(opts);
        acrylmicrovm_runtime_free(runtime);
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
    AcrylMicrovmCommand cmd = {
        .command = "/bin/ls",
        .args = args,
        .argc = 2,
    };
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

    // Cleanup (runtime frees all boxes)
    acrylmicrovm_runtime_free(runtime);
    return 0;
}
```

### Runtime Image Management

`acrylmicrovm_runtime_images()` returns the images a runtime can boot from: the
local cache on a runtime from `acrylmicrovm_runtime_new()`, the server's catalog on
a REST runtime. Pull is local only (a REST runtime pulls when a box is
created) and usage is REST only. Each operation queues its work and calls back
from `acrylmicrovm_runtime_drain()`:

```c
static void on_list(CImageInfoList* list, CAcrylMicrovmError* error, void* done) {
    if (error->code == Ok) {
        printf("Images: %d\n", list->count);
        acrylmicrovm_free_image_info_list(list);
    }
    *(int*)done = 1;
}

static void on_removed(CAcrylMicrovmError* error, void* done) {
    if (error->code != Ok) {
        fprintf(stderr, "remove failed: %s\n", error->message);
    }
    *(int*)done = 1;
}

CAcrylMicrovmImageHandle* images = NULL;
if (acrylmicrovm_runtime_images(runtime, &images, &error) == Ok) {
    int listed = 0;
    if (acrylmicrovm_image_list(images, on_list, &listed, &error) == Ok) {
        while (!listed && acrylmicrovm_runtime_drain(runtime, -1, &error) >= 0) {
        }
    }

    // A name, not a reference: "docker.io/library/alpine:3.20" is refused.
    int removed = 0;
    if (acrylmicrovm_image_remove(images, "docker.io/library/alpine", on_removed,
                             &removed, &error) == Ok) {
        while (!removed && acrylmicrovm_runtime_drain(runtime, -1, &error) >= 0) {
        }
    }

    acrylmicrovm_image_free(images);
}
```

`acrylmicrovm_image_get()` and `acrylmicrovm_image_create_alias()` deliver a `CImageDetail`
(aliases, versions newest first, and alias history) and `acrylmicrovm_image_usage()` a
`CImageUsage`. See
[Images in the C reference](../../docs/reference/c/README.md#images) for each
operation on each runtime.

### Private Registry Logins

On a REST runtime, `acrylmicrovm_image_registries()` returns the logins the
server pulls private images with; a local runtime answers `Unsupported` and
takes its logins from `image_registries` instead. The password is copied,
sent once, and never returned.

```c
static void on_created(CRegistryCredential* login, CAcrylMicrovmError* error,
                       void* user_data) {
    if (error->code == Ok) {
        printf("added %s for %s/%s\n", login->id, login->registry_host,
               login->repository_prefix);
        acrylmicrovm_free_registry_credential(login);
    }
    *(int*)user_data = 1;
}

CAcrylMicrovmRegistryHandle* registries = NULL;
if (acrylmicrovm_image_registries(runtime, &registries, &error) == Ok) {
    int done = 0;
    if (acrylmicrovm_registry_create(registries, "ghcr.io", "acme/", "acme-bot",
                                getenv("GHCR_TOKEN"), on_created, &done,
                                &error) == Ok) {
        while (!done && acrylmicrovm_runtime_drain(runtime, -1, &error) >= 0) {
        }
    }
    acrylmicrovm_registry_free(registries);
}
```

See [Registries in the C reference](../../docs/reference/c/README.md#registries).

### Guest SSH Control

`acrylmicrovm_box_ssh_configure()`, `acrylmicrovm_box_ssh_status()`,
`acrylmicrovm_box_ssh_enable()`, and `acrylmicrovm_box_ssh_disable()` configure, query,
enable saved configuration, and disable guest SSH through
the Native API. See the [C SSH reference](../../docs/reference/c/README.md#guest-ssh-control)
for configuration fields, callbacks, and `CSshStatus`.

---

## API Overview

### Simple API

The Simple API provides a streamlined interface for common use cases with automatic resource management.

#### Key Functions

```c
// Create and auto-start a box
AcrylMicrovmErrorCode acrylmicrovm_simple_new(
    const char* image,          // "python:slim", "alpine:3.19", etc.
    double cpus,                // 0 = default; fractions require a Linux execution host
    int memory_mib,             // 0 = default (512)
    CAcrylMicrovmSimple** out_box,
    CAcrylMicrovmError* out_error
);

// Run command and get buffered result
AcrylMicrovmErrorCode acrylmicrovm_simple_run(
    CAcrylMicrovmSimple* box,
    const char* command,
    const char** args,          // NULL-terminated array
    int argc,
    CAcrylMicrovmExecResult** out_result,
    CAcrylMicrovmError* out_error
);

// Free result (stdout, stderr, exit code)
void acrylmicrovm_result_free(CAcrylMicrovmExecResult* result);

// Auto-cleanup (stop + remove)
void acrylmicrovm_simple_free(CAcrylMicrovmSimple* box);
```

#### When to Use Simple API
- ✅ Quick prototypes and scripts
- ✅ Single-box applications
- ✅ Buffered output is acceptable
- ✅ Standard resource limits (2 CPUs, 512 MB)

#### When to Use Native API Instead
- ❌ Need streaming output callbacks
- ❌ Custom volumes or networking
- ❌ Multi-box orchestration
- ❌ Advanced configuration (custom box options)

### Native API

The Native API provides full control and advanced features.

#### Runtime Management

```c
// Get version
const char* acrylmicrovm_version(void);

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

// Create runtime with options
AcrylMicrovmErrorCode acrylmicrovm_runtime_new(
    const char* home_dir,            // NULL = ~/.acryl/microvm
    const AcrylMicrovmImageRegistry* image_registries, // NULL = default registries
    int image_registries_count,
    CAcrylMicrovmRuntime** out_runtime,
    CAcrylMicrovmError* out_error
);

// Graceful shutdown
AcrylMicrovmErrorCode acrylmicrovm_runtime_shutdown(
    CAcrylMicrovmRuntime* runtime,
    int timeout,  // 0=default(10s), -1=infinite
    CAcrylMicrovmError* out_error
);

// Runtime-wide metrics
AcrylMicrovmErrorCode acrylmicrovm_runtime_metrics(
    CAcrylMicrovmRuntime* runtime,
    CRuntimeMetrics* out_metrics,
    CAcrylMicrovmError* out_error
);

// Free runtime (auto-frees all boxes)
void acrylmicrovm_runtime_free(CAcrylMicrovmRuntime* runtime);
```

#### Box Lifecycle

Nested virtualization is a creation option:
`acrylmicrovm_advanced_options_set_nested_virtualization(advanced, 1)` requires runtime
experimental authorization. Attach the advanced options before create; runtime
saves the existing BoxConfig. `acrylmicrovm_start_box` retains its original signature
and uses saved configuration. Passing `0` selects an ordinary new Box.

```c
// Create box (auto-started)
AcrylMicrovmErrorCode acrylmicrovm_create_box(
    CAcrylMicrovmRuntime* runtime,
    CAcrylMicrovmOptions* opts,
    CBoxHandle** out_box,
    CAcrylMicrovmError* out_error
);

// Start/restart a stopped box
AcrylMicrovmErrorCode acrylmicrovm_start_box(
    CBoxHandle* handle,
    CAcrylMicrovmError* out_error
);

// Stop box; a local runtime also removes it unless its options called
// acrylmicrovm_options_set_auto_delete_interval(opts, 0)
AcrylMicrovmErrorCode acrylmicrovm_stop_box(
    CBoxHandle* handle,
    CAcrylMicrovmError* out_error
);

// Remove box
AcrylMicrovmErrorCode acrylmicrovm_remove(
    CAcrylMicrovmRuntime* runtime,
    const char* id_or_name,
    int force,  // 1=remove even if running
    CAcrylMicrovmError* out_error
);

// Reattach to existing box
AcrylMicrovmErrorCode acrylmicrovm_get(
    CAcrylMicrovmRuntime* runtime,
    const char* id_or_name,      // Full ID or prefix
    CBoxHandle** out_handle,
    CAcrylMicrovmError* out_error
);

// Get box ID (caller must free with acrylmicrovm_free_string)
char* acrylmicrovm_box_id(CBoxHandle* handle);
```

#### Command Execution

```c
typedef struct AcrylMicrovmCommand {
    const char* command;      // Required
    const char* const* args;  // Argument array, or NULL
    int argc;
    const char* const* env_pairs; // [key0, value0, key1, value1, ...], or NULL
    int env_count;
    const char* workdir;      // Working directory, or NULL
    const char* user;         // User spec (e.g., "nobody", "1000:1000"), or NULL
    double timeout_secs;      // 0.0 = no timeout
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

AcrylMicrovmErrorCode acrylmicrovm_execution_write(CExecutionHandle* execution, const char* data, int len, CAcrylMicrovmError* out_error);
AcrylMicrovmErrorCode acrylmicrovm_execution_wait(CExecutionHandle* execution, int* out_exit_code, CAcrylMicrovmError* out_error);
AcrylMicrovmErrorCode acrylmicrovm_execution_kill(CExecutionHandle* execution, CAcrylMicrovmError* out_error);
AcrylMicrovmErrorCode acrylmicrovm_execution_resize_tty(CExecutionHandle* execution, int rows, int cols, CAcrylMicrovmError* out_error);
void acrylmicrovm_execution_free(CExecutionHandle* execution);
```

**Example: structured command with options**

```c
const char* env[] = {"MY_VAR", "hello"};
AcrylMicrovmCommand cmd = {
    .command = "pwd",
    .args = NULL,
    .argc = 0,
    .env_pairs = env,
    .env_count = 2,
    .workdir = "/tmp",
    .user = "nobody",
    .timeout_secs = 30.0,
};
CExecutionHandle* execution = NULL;
int exit_code;
if (acrylmicrovm_execute(box, &cmd, my_callback, NULL, &execution, &error) == Ok) {
    acrylmicrovm_execution_wait(execution, &exit_code, &error);
    acrylmicrovm_execution_free(execution);
}
```

#### Live Execution Discovery and Recovery

`acrylmicrovm_box_executions_list(box, cb, user_data, &error)` queues a live guest query.
Drain the runtime as for other async operations. On success the callback owns
the `CExecutionInfoList*` and must free it with
`acrylmicrovm_free_execution_info_list`. Each record has `execution_id` (canonical
guest ID), `argv`/`argc`, `started_at` (Unix milliseconds), `tty`, and
`is_main_session`. The main process is included; completed executions disappear.
Listing never boots a box; a stopped/unstarted box returns an empty list, and
a paused box returns an invalid-state error.

`acrylmicrovm_execution_id(execution)` returns an owned ID string; release it with
`acrylmicrovm_free_string`. `acrylmicrovm_box_executions_get(box, id, &execution, &error)`
synchronously recovers control without taking stdin/stdout/stderr. The existing
wait, kill, signal, and TTY resize callback APIs work on that handle, even while
another client reads output. Freeing recovered controls does not kill the
process.

`acrylmicrovm_box_executions_attach(box, id, &execution, &error)` synchronously opens
live streams for an existing ID without starting the box. Register stdout,
stderr, and exit callbacks as for `acrylmicrovm_box_exec`. If a reader is already
active, attach reports a conflict without stealing its output. A new client can
attach after the former reader disconnects. Attachment includes available buffered
output; it does not guarantee a complete historical log.
Freeing an attached handle releases its reader without killing the process.

`acrylmicrovm_box_executions_attach_read_only` is the same attach without a stdin
channel. Prefer it when the client only reads: freeing a stdin attach sends a
stdin close to the guest, which a process reading stdin sees as EOF and may act
on.

#### Network Tunnels

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
```

Each `CBoxTunnelHandle` is one-shot. Choose `acrylmicrovm_tunnel_connect()` or
`acrylmicrovm_tunnel_forward()`; either consumes the prepared tunnel.
`acrylmicrovm_tunnel_uri()` reports where a remotely served tunnel can be reached, as
an allocated string the caller frees with `acrylmicrovm_free_string()`. It writes NULL
for a local tunnel.

Create a target with the system's `sockaddr_storage` and the numeric address parser:

```c
struct sockaddr_storage target;
CAcrylMicrovmError error = {0};
AcrylMicrovmErrorCode rc = acrylmicrovm_socket_address_parse("127.0.0.1:3000", &target, &error);
if (rc != Ok) {
    /* Inspect error.message, then release it with acrylmicrovm_error_free(). */
    return rc;
}
AcrylMicrovmTunnelOptions options = { .is_public = false };
CBoxTunnelHandle* tunnel = NULL;
rc = acrylmicrovm_network_tunnel(network, &target, &options, &tunnel, &error);
```

The parser accepts numeric `IPv4:port` and `[IPv6]:port`, performs no DNS lookup,
and leaves output unchanged on error. Storage belongs to the caller and needs no
cleanup. Pass a complete `sockaddr_storage`, never a cast pointer to a smaller
`sockaddr_in` or `sockaddr_in6`. Tunnel targets must have a loopback or guest IP,
a nonzero port, and zero IPv6 scope/flow information. Address and options are borrowed
only for the call. NULL `options` defaults to public; set `is_public = false` for
private cloud access. This replaces the custom target and JSON options arguments;
`AcrylMicrovmSocketAddress` remains the separate TCP/Unix listener input to `forward`.
Read `uri`, `isPublic`, and `token`
with `acrylmicrovm_tunnel_descriptor()`, before or after connect/forward consumes the
handle, freeing its JSON with `acrylmicrovm_free_string()`. Only creation returns a full token; later preparations
return a mask. Forwarding reuses the declaration and authenticates CONNECT with
the runtime's API key or JWT; it never recreates a revoked tunnel.

This differs from `acrylmicrovm_options_add_port()`, which creates a persistent,
local-only host listener that accepts repeated connections.

#### Domains

On a hosted runtime the organization can route a name of its own
(`demo.apps.example`) to a box port; see
[Domains](../../docs/concepts/networking.md#domains). These calls block until
the server answers.

```c
AcrylMicrovmErrorCode acrylmicrovm_runtime_domains(
    CAcrylMicrovmRuntime* runtime,
    CAcrylMicrovmDomainHandle** out_handle,
    CAcrylMicrovmError* out_error
);
AcrylMicrovmErrorCode acrylmicrovm_domain_list(
    CAcrylMicrovmDomainHandle* handle,
    CDomainList** out_list,
    CAcrylMicrovmError* out_error
);
AcrylMicrovmErrorCode acrylmicrovm_domain_bind(
    CAcrylMicrovmDomainHandle* handle,
    const char* domain,
    const CDomainTarget* target,
    CDomain** out_domain,
    CAcrylMicrovmError* out_error
);
AcrylMicrovmErrorCode acrylmicrovm_domain_unbind(
    CAcrylMicrovmDomainHandle* handle,
    const char* domain,
    CAcrylMicrovmError* out_error
);
void acrylmicrovm_free_domain_handle(CAcrylMicrovmDomainHandle* handle);
void acrylmicrovm_free_domain(CDomain* domain);
void acrylmicrovm_free_domain_list(CDomainList* list);
```

A target is `{ .type = AcrylMicrovmDomainTargetBox, .box = "web-a", .port = 3000 }`;
`box` takes an id or name. In a returned `CDomain`, `hostname` is the full hostname
and there is no short `name`. Box targets have `target.box` and `target.port`;
reserved `AcrylMicrovmDomainTargetApp` targets have `target.app`. Inactive reference
pointers are NULL; both are NULL while unbound. Initialize unused input fields
to zero. App binding currently returns `app_target_not_supported`.
`acrylmicrovm_domain_bind()` and `acrylmicrovm_domain_unbind()` require an owned full hostname.
Request new names in Dashboard > Domains before binding. A local runtime answers
`Unsupported`. The changed domain structures require rebuilding C consumers.

Release the management handle with `acrylmicrovm_free_domain_handle()`, the result
of `acrylmicrovm_domain_bind()` with `acrylmicrovm_free_domain()`, and the result of
`acrylmicrovm_domain_list()` with `acrylmicrovm_free_domain_list()`. The list owns its
entries; do not free them individually. These functions release local SDK
resources and leave domain bindings intact. Use `acrylmicrovm_domain_unbind()` to
stop routing a domain.

#### Discovery & Introspection

```c
// List all boxes
AcrylMicrovmErrorCode acrylmicrovm_list_info(
    CAcrylMicrovmRuntime* runtime,
    CBoxInfoListCb cb,
    void* user_data,
    CAcrylMicrovmError* out_error
);

// Get specific box info
AcrylMicrovmErrorCode acrylmicrovm_get_info(
    CAcrylMicrovmRuntime* runtime,
    const char* id_or_name,
    CBoxInfoCb cb,
    void* user_data,
    CAcrylMicrovmError* out_error
);

// Get box info from handle
AcrylMicrovmErrorCode acrylmicrovm_box_info(
    CBoxHandle* handle,
    CBoxInfoCb cb,
    void* user_data,
    CAcrylMicrovmError* out_error
);

// Per-box metrics
AcrylMicrovmErrorCode acrylmicrovm_box_metrics(
    CBoxHandle* handle,
    CBoxMetrics* out_metrics,
    CAcrylMicrovmError* out_error
);
```

`CBoxInfo.network` is an owned `CNetworkInfo*` and is `NULL` when network
metadata is unavailable. `CNetworkInfo` exposes a `COutboundNetworkInfo`/`CInboundNetworkInfo` per
direction (`outbound`, `inbound`) — each with `mode`, the `allow_net` string
array and count — plus a nullable `CPublishedPortList*`. The deprecated
top-level `mode`, `allow_net` and `allow_net_count` fields remain at their
pre-split offsets and alias `outbound`, so callers built against the old header
keep working; they are borrowed views and must not be freed separately. A `NULL`
`published_ports` pointer means the current handle does not know the bindings,
a non-`NULL` list with `count == 0` means there are no active publications, and
a populated list contains typed `CPublishedPort` entries with `guest_port`,
`host_ip`, `host_port`, and `protocol`.

The network pointer and every nested array and string are borrowed from their
enclosing `CBoxInfo`; do not free them separately. Info callbacks own their
result and must call `acrylmicrovm_free_box_info()` or
`acrylmicrovm_free_box_info_list()` to release the complete object graph. All info
operations complete asynchronously through `acrylmicrovm_runtime_drain()`.

### Error Handling

The C SDK introduces structured error handling with error codes and detailed messages.

#### Error Codes

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

#### Error Struct

```c
typedef struct CAcrylMicrovmError {
    AcrylMicrovmErrorCode code;  // Error code for programmatic handling
    char* message;           // Detailed message (NULL if none)
} CAcrylMicrovmError;
```

#### Error Handling Patterns

**Pattern 1: Basic Check**

```c
CAcrylMicrovmError error = {0};
AcrylMicrovmErrorCode code = acrylmicrovm_simple_new("alpine:3.19", 0, 0, &box, &error);

if (code != Ok) {
    fprintf(stderr, "Error %d: %s\n", error.code, error.message);
    acrylmicrovm_error_free(&error);
    return 1;
}

// Success path
acrylmicrovm_simple_free(box);
```

**Pattern 2: Switch on Error Code**

```c
CAcrylMicrovmError error = {0};
AcrylMicrovmErrorCode code = acrylmicrovm_get(runtime, "box-id", &box, &error);

switch (code) {
    case Ok:
        // Success
        break;
    case InvalidArgument:
        printf("Invalid argument: %s\n", error.message);
        break;
    case NotFound:
        printf("Resource not found: %s\n", error.message);
        break;
    default:
        printf("Error %d: %s\n", error.code, error.message);
}

acrylmicrovm_error_free(&error);
```

**Pattern 3: Retry Logic**

```c
int retries = 3;
for (int i = 0; i < retries; i++) {
    code = acrylmicrovm_simple_new(..., &error);
    if (code == Ok) {
        break;  // Success
    }

    printf("Retry %d/%d failed: %s\n", i+1, retries, error.message);
    acrylmicrovm_error_free(&error);

    if (code == InvalidArgument || code == Unsupported) {
        break;  // Non-retryable errors
    }
    sleep(1);  // Backoff
}
```

---

## Complete API Reference

For the full API reference with detailed parameter tables and code examples, see:

**[C SDK API Reference](docs/reference/c/README.md)**

---

## Examples

The `examples/c/` directory contains 8 examples:

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

### Building and Running Examples

```bash
cd examples/c
mkdir -p build && cd build
cmake ..
make

# Run any example
./simple_api_demo
./01_lifecycle
./05_metrics
```

---

## Memory Management

### Rules

1. **All allocated strings must be freed**
   - `acrylmicrovm_box_id()` → `acrylmicrovm_free_string()`

2. **Error structs must be freed**
   - `CAcrylMicrovmError` → `acrylmicrovm_error_free()`

3. **Results must be freed**
   - `CAcrylMicrovmExecResult` → `acrylmicrovm_result_free()`
   - `CBoxInfo` → `acrylmicrovm_free_box_info()`
   - `CBoxInfoList` → `acrylmicrovm_free_box_info_list()`
   - `CImagePullResult` → `acrylmicrovm_free_image_pull_result()`
   - `CImageInfoList` → `acrylmicrovm_free_image_info_list()`
   - `CImageDetail` → `acrylmicrovm_free_image_detail()`

4. **Handles have specific free functions**
   - `CAcrylMicrovmRuntime` → `acrylmicrovm_runtime_free()` (auto-frees all boxes)
   - `CBoxHandle` → `acrylmicrovm_box_free()`
   - `CAcrylMicrovmSimple` → `acrylmicrovm_simple_free()`

5. **All cleanup functions are NULL-safe**

### Common Patterns

**String output:**
```c
char* id = acrylmicrovm_box_id(box);
printf("ID: %s\n", id);
acrylmicrovm_free_string(id);  // MUST free
```

**Error handling:**
```c
CAcrylMicrovmError error = {0};
AcrylMicrovmErrorCode code = acrylmicrovm_simple_new(..., &error);
if (code != Ok) {
    fprintf(stderr, "%s\n", error.message);
    acrylmicrovm_error_free(&error);  // MUST free
}
```

**Execution results:**
```c
CAcrylMicrovmExecResult* result;
acrylmicrovm_simple_run(..., &result, &error);
printf("Output: %s\n", result->stdout_text);
acrylmicrovm_result_free(result);  // MUST free
```

### Memory Leak Detection

Use valgrind (Linux) or Instruments (macOS) to detect leaks:

```bash
# Linux
valgrind --leak-check=full ./my_app

# macOS
leaks -atExit -- ./my_app
```

---

## Threading & Safety

### Thread Safety

- ✅ **`CAcrylMicrovmRuntime` is thread-safe** - Multiple threads can call runtime functions concurrently
- ⚠️ **`CBoxHandle` is NOT thread-safe** - Don't share box handles across threads
- ⚠️ **`CAcrylMicrovmSimple` is NOT thread-safe** - Don't share simple boxes across threads

### Best Practices

**Safe: One runtime, multiple threads**
```c
CAcrylMicrovmRuntime* runtime = NULL;
CAcrylMicrovmError error = {0};
acrylmicrovm_runtime_new(NULL, NULL, 0, &runtime, &error);

// Thread 1
CBoxHandle* box1 = NULL;
CAcrylMicrovmOptions* opts1 = NULL;
acrylmicrovm_options_new("alpine:3.19", &opts1, &error);
acrylmicrovm_create_box(runtime, opts1, &box1, &error);
acrylmicrovm_options_free(opts1);

// Thread 2
CBoxHandle* box2 = NULL;
CAcrylMicrovmOptions* opts2 = NULL;
acrylmicrovm_options_new("alpine:3.19", &opts2, &error);
acrylmicrovm_create_box(runtime, opts2, &box2, &error);
acrylmicrovm_options_free(opts2);
```

**Unsafe: Sharing box handle across threads**
```c
CBoxHandle* box = NULL;
acrylmicrovm_create_box(runtime, opts, &box, &error);

// Thread 1
acrylmicrovm_execute(box, &cmd1, ...);  // UNSAFE

// Thread 2
acrylmicrovm_execute(box, &cmd2, ...);  // UNSAFE
```

**Safe: Per-thread boxes**
```c
void* thread_func(void* arg) {
    CAcrylMicrovmRuntime* runtime = (CAcrylMicrovmRuntime*)arg;
    CBoxHandle* box = NULL;
    CAcrylMicrovmError error = {0};
    CAcrylMicrovmOptions* opts = NULL;
    acrylmicrovm_options_new("alpine:3.19", &opts, &error);
    acrylmicrovm_create_box(runtime, opts, &box, &error);
    acrylmicrovm_options_free(opts);
    int exit_code = 0;
    const char* args[] = {"hello"};
    AcrylMicrovmCommand cmd = {.command = "/bin/echo", .args = args, .argc = 1};
    CExecutionHandle* execution = NULL;
    if (acrylmicrovm_execute(box, &cmd, NULL, NULL, &execution, &error) == Ok) {
        acrylmicrovm_execution_wait(execution, &exit_code, &error);
        acrylmicrovm_execution_free(execution);
    }
    acrylmicrovm_stop_box(box, &error);
    return NULL;
}
```

### Callback Execution

Callbacks are invoked on the **calling thread**. Do not block in callbacks.

---

## Platform Support

### Supported Platforms

| Platform | Architecture | Status | Requirements |
|----------|-------------|--------|--------------|
| macOS    | ARM64 (Apple Silicon) | ✅ Full support | macOS 11.0+, Hypervisor.framework |
| macOS    | x86_64 (Intel) | ❌ Not supported | N/A |
| Linux    | x86_64 | ✅ Full support | KVM enabled |
| Linux    | ARM64 (aarch64) | ✅ Full support | KVM enabled |
| Windows  | Any | ❌ Not supported | Use WSL2 |

### Platform-Specific Notes

**macOS:**
- Requires Hypervisor.framework (built-in on macOS 11.0+)
- Intel Macs are not supported
- Dylib search paths: use `install_name_tool` or `DYLD_LIBRARY_PATH`

**Linux:**
- Requires KVM kernel module: `sudo modprobe kvm kvm_intel` (or `kvm_amd`)
- Check support: `kvm-ok` or `lsmod | grep kvm`
- Library search paths: use `LD_LIBRARY_PATH` or `ldconfig`

**Windows:**
- Use WSL2 (Windows Subsystem for Linux 2)
- Follow Linux instructions inside WSL2

---

## Migration Guide

### Unreleased

**Breaking Changes:**
- `acrylmicrovm_options_add_volume` is renamed `acrylmicrovm_options_add_bind_mount`, and
  the new `acrylmicrovm_options_add_managed_volume` mounts a managed volume by its
  server-assigned id or its name. The two mount origins are distinct: a host
  path is local-runtime only, a managed volume REST-runtime only.
- `acrylmicrovm_volume_create` takes a new `const char *name` as its second
  argument, before the callback. Pass `NULL` for an unnamed volume. A non-NULL
  name that is not valid UTF-8 returns `InvalidArgument` rather than silently
  creating an unnamed volume.
- `CVolumeInfo` gains a `name` field between `id` and `created_at`, changing
  the struct layout. Recompile any consumer that reads it; the field is a
  heap-owned string released by the existing
  `acrylmicrovm_free_volume_info` / `acrylmicrovm_free_volume_info_list`.
- `acrylmicrovm_options_add_port` signature changed. Parameters are now host-first
  (matching `PortSpec`, the other SDKs, and Docker's `host:guest` convention),
  ports are `uint16_t`, the new `AcrylMicrovmPortProtocol` enum and a nullable
  `host_ip` bind address were added, and the function returns
  `AcrylMicrovmErrorCode` instead of `void`.
- `acrylmicrovm_box_info` intentionally breaks the C ABI: metadata lookup is now
  asynchronous, so the `CBoxInfo **out_info` argument was replaced by a
  `CBoxInfoCb` callback and `void *user_data`. `Ok` means the request was
  queued; call `acrylmicrovm_runtime_drain()` to dispatch the callback. On success,
  the callback owns the `CBoxInfo *` and must release it with
  `acrylmicrovm_free_box_info()`.
- `acrylmicrovm_copy_into`, `acrylmicrovm_copy_out` and `acrylmicrovm_copy_out_start` no
  longer flatten a directory source. The C bindings - and the Go SDK built on
  them - copied a directory's contents where the Rust, Python and Node SDKs
  copied the directory itself. `acrylmicrovm_copy_into` and `acrylmicrovm_copy_out` now
  land a directory as `docker cp` does: it becomes a destination that does not
  exist, and lands under its own name in one that does. End the source path
  with `/.` to copy only the contents either way, as before.
- `acrylmicrovm_copy_in_start` takes a `AcrylMicrovmCopySourceKind source_kind` as its
  third argument instead of `bool source_is_dir`. Use
  `AcrylMicrovmCopySourceKindDir` for a directory tree,
  `AcrylMicrovmCopySourceKindFile` for a single file, or
  `AcrylMicrovmCopySourceKindUnknown` when the caller cannot tell (older
  clients). The guest lands every upload by the archive's first entry, so
  the kind does not change where it lands; `...Dir` makes a non-recursive
  copy fail before anything is sent.

Before:
```c
acrylmicrovm_options_add_volume(opts, "/host/data", "/data", 0);
acrylmicrovm_volume_create(handle, on_created, user_data, &err);
```

After:
```c
/* host bind (local runtime) */
acrylmicrovm_options_add_bind_mount(opts, "/host/data", "/data", 0);
/* managed volume by id or name (REST runtime) */
acrylmicrovm_options_add_managed_volume(opts, "my-data", "/data", 0);

acrylmicrovm_volume_create(handle, "my-data", on_created, user_data, &err);
acrylmicrovm_volume_create(handle, NULL, on_created, user_data, &err);  /* unnamed */
```

Before:
```c
acrylmicrovm_options_add_port(opts, 80, 8080);  /* guest, host */
```

After:
```c
acrylmicrovm_options_add_port(opts, 8080, 80, AcrylMicrovmPortProtocolTcp, NULL);  /* host, guest */
```

Before:
```c
CBoxInfo *info = NULL;
acrylmicrovm_box_info(box, &info, &error);
```

After:
```c
acrylmicrovm_box_info(box, on_info, user_data, &error);
while (!request_done) {
    acrylmicrovm_runtime_drain(runtime, -1, &error);
}
```

Before:
```c
acrylmicrovm_copy_in_start(handle, "/dst", true, copy_cb, user_data, &error);
```

After:
```c
/* dir tree */
acrylmicrovm_copy_in_start(handle, "/dst", AcrylMicrovmCopySourceKindDir, copy_cb, user_data, &error);
/* one file */
acrylmicrovm_copy_in_start(handle, "/dst", AcrylMicrovmCopySourceKindFile, copy_cb, user_data, &error);
/* old client, shape unknown */
acrylmicrovm_copy_in_start(handle, "/dst", AcrylMicrovmCopySourceKindUnknown, copy_cb, user_data, &error);
```

### From 0.1.x to 0.2.0

**Breaking Changes:**
- Simple API added (new feature, backward compatible)
- Error handling enhanced (new `CAcrylMicrovmError` struct, backward compatible with old API)

**No code changes required** if using old API. Existing programs will continue to work.

**Recommended migrations:**

**1. Simple use cases → Simple API**

Before (0.1.x):
```c
char* error = NULL;
CAcrylMicrovmRuntime* runtime = acrylmicrovm_runtime_new(NULL, &error);
CBoxHandle* box = acrylmicrovm_create_box(runtime, opts, &error);
old_execute_api(box, "/bin/echo", args, NULL, NULL, &error);
acrylmicrovm_stop_box(box, &error);
acrylmicrovm_runtime_free(runtime);
```

After (0.2.0):
```c
CAcrylMicrovmSimple* box;
CAcrylMicrovmError error = {0};

acrylmicrovm_simple_new("alpine:3.19", 0, 0, &box, &error);
const char* args[] = {"hello", NULL};
CAcrylMicrovmExecResult* result;
acrylmicrovm_simple_run(box, "/bin/echo", args, 1, &result, &error);
acrylmicrovm_result_free(result);
acrylmicrovm_simple_free(box);
```

**2. Error handling → Structured errors**

Before:
```c
char* error = NULL;
if (!runtime) {
    fprintf(stderr, "Error: %s\n", error);
    // Parse error string to understand type
}
```

After:
```c
CAcrylMicrovmError error = {0};
if (code != Ok) {
    switch (error.code) {
        case NotFound:
            // Handle not found
            break;
        case InvalidArgument:
            // Handle invalid argument
            break;
    }
    acrylmicrovm_error_free(&error);
}
```

---

## Troubleshooting

### Library Not Found

**Error:** `dyld: Library not loaded: @rpath/libacrylmicrovm.dylib`

**Solution:**
```bash
# macOS: Add RPATH to executable
install_name_tool -add_rpath /path/to/lib my_app

# Linux: Set LD_LIBRARY_PATH
export LD_LIBRARY_PATH=/path/to/lib:$LD_LIBRARY_PATH
```

### Box Creation Fails

**Error:** `Failed to create box: Image error: ...`

**Solutions:**
1. Check internet connection (for image pull)
2. Verify image name: `"alpine:3.19"` (not `alpine:3.19` without quotes)
3. Check disk space: `df -h ~/.acryl/microvm`
4. Enable debug logs: `RUST_LOG=debug ./my_app`

### KVM Not Available (Linux)

**Error:** `UnsupportedEngine` or `kvm: Permission denied`

**Solutions:**
```bash
# Check KVM support
kvm-ok

# Load KVM module
sudo modprobe kvm kvm_intel  # or kvm_amd

# Add user to kvm group
sudo usermod -aG kvm $USER
newgrp kvm
```

### Crash on Apple Intel Mac

**Error:** Segmentation fault or `UnsupportedEngine`

**Solution:** Intel Macs are not supported. Use ARM64 Mac or Linux.

### Memory Leaks

**Run valgrind:**
```bash
valgrind --leak-check=full --show-leak-kinds=all ./my_app
```

**Common causes:**
- Not freeing strings: `acrylmicrovm_box_id()`
- Not freeing errors: `acrylmicrovm_error_free()`
- Not freeing results: `acrylmicrovm_result_free()` and typed list/info structs

### High Memory Usage

**Check box count:**
```c
CRuntimeMetrics metrics = {0};
CAcrylMicrovmError error = {0};
acrylmicrovm_runtime_metrics(runtime, &metrics, &error);
printf("Running boxes: %d\n", metrics.num_running_boxes);
```

**Reduce memory per box:**
```c
// Simple API: Can't configure (uses defaults)
// Use native API instead:
CAcrylMicrovmOptions* opts = NULL;
acrylmicrovm_options_new("alpine:3.19", &opts, &error);
acrylmicrovm_options_set_memory(opts, 256);
```

### Command Hangs

**Possible causes:**
1. Command waiting for input (use non-interactive commands)
2. Large output without callback (output buffer full)
3. Deadlock in callback function

**Solutions:**
- Use streaming callback for large output
- Don't block in callbacks
- Set command timeout (future feature)

---

## Architecture

The C SDK is a thin wrapper around the Rust `acrylmicrovm` crate:

```text
sdks/c/src/lib.rs
  ↓ (exports C ABI)
sdks/c/src/runtime.rs    (runtime management)
sdks/c/src/box_handle.rs (box lifecycle)
sdks/c/src/exec.rs       (command execution)
sdks/c/src/images.rs     (image operations)
sdks/c/src/info.rs       (box info)
sdks/c/src/metrics.rs    (metrics)
sdks/c/src/copy.rs       (file copy)
  ↓ (wraps)
acrylmicrovm/src/runtime/
```

- Built as the `acrylmicrovm-c` crate to produce `cdylib`/`staticlib`
- Header auto-generated from Rust code using `cbindgen`
- Typed structs are used for complex inputs and outputs
- Maintains same functionality as Rust API

### Development

**Rebuilding Header:**
```bash
cargo build -p acrylmicrovm-c
# Outputs: sdks/c/include/acrylmicrovm.h
```

**Adding New Functions:**
1. Add implementation to the appropriate file in `sdks/c/src/`
2. Add the exported C symbol in the same domain module
3. Rebuild: `cargo build -p acrylmicrovm-c`
4. Header is automatically updated

**Testing:**
See `sdks/c/tests/` for the test suite.

- `make test:unit:ffi` runs the Rust-side FFI tests; `make test:unit:c` and `make test:integration:c` build and run the C tests. `FILTER=<pattern>` narrows each.
- Rust FFI tests borrowing runtime out-pointers should use `runtime.as_ref().expect(...)` at the borrow site, making the non-null precondition explicit to static analysis. Keep the allocation alive through the reference's last use, then free the runtime; a null check alone does not establish pointer validity.
- `make test:unit:ffi FILTER=checkpoint` exercises checkpoint create/list/get/import/remove through loopback HTTP and the real post-and-drain queue, including callback-owned metadata, Box handles and errors. Native binding coverage runs these Rust FFI tests alongside the C executable tests; no VM is needed.
- `make test:unit:c FILTER=agents` obtains a live Box handle from a loopback REST server, checks successful namespace allocation and release, and verifies agent-operation errors without starting a VM. Null-handle checks alone cannot cover namespace ownership.
- C integration tests pin `alpine:3.19`, which avoids drifting to the latest Alpine release line; the tag still receives patch updates, so it does not guarantee an immutable userland. `test_ssh_integration.c` pulls it through the same registry mirrors the Python and Node suites use.
- `FREE_STR_CALLS` counts string frees across the whole test process. A Rust test that frees a counted FFI payload, or reads the counter, must hold `FREE_STR_LOCK` for as long as it does so, even if it never reads the counter. Without the lock, plain `cargo test` threads count its frees in another test's delta. nextest's process-per-test runs hide this.

---

## License

Apache-2.0

---

## Links

- **[C SDK API Reference](docs/reference/c/README.md)** - Complete function reference
- **[C Quick Start](docs/getting-started/quickstart-c.md)** - 5-minute guide
- **Examples:** `examples/c/`
- **Tests:** `sdks/c/tests/`
