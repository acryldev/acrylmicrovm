cmake_minimum_required(VERSION 3.15)
project(acrylmicrovm_c_tests C)

set(CMAKE_C_STANDARD 11)

# Find the ACRYL microVMs library
set(ACRYLMICROVM_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../..")
set(ACRYLMICROVM_INCLUDE_DIR "${ACRYLMICROVM_ROOT}/sdks/c/include")
set(ACRYLMICROVM_LIB_DIR "${ACRYLMICROVM_ROOT}/target/debug" CACHE PATH "Directory containing the native SDK library")

# Detect platform and set library name
if(APPLE)
    set(ACRYLMICROVM_LIB "${ACRYLMICROVM_LIB_DIR}/libacrylmicrovm.dylib")
elseif(UNIX)
    set(ACRYLMICROVM_LIB "${ACRYLMICROVM_LIB_DIR}/libacrylmicrovm.so")
else()
    message(FATAL_ERROR "Unsupported platform")
endif()

# Check if library exists
if(NOT EXISTS "${ACRYLMICROVM_LIB}")
    message(FATAL_ERROR "ACRYL microVMs library not found at ${ACRYLMICROVM_LIB}. Please run: make dev:c")
endif()

# Check if header exists
if(NOT EXISTS "${ACRYLMICROVM_INCLUDE_DIR}/acrylmicrovm.h")
    message(FATAL_ERROR "ACRYL microVMs header not found at ${ACRYLMICROVM_INCLUDE_DIR}/acrylmicrovm.h. Please run: make dev:c")
endif()

# Include directories
include_directories(${ACRYLMICROVM_INCLUDE_DIR})

# Test executables.
# Note: lifecycle/execute/streaming/etc. tests were removed when the C SDK
# moved to the post-and-drain callback API; coverage of those code paths now
# lives in the Go SDK + runner integration suite.
set(TEST_TARGETS
    test_agents
    test_simple_api
    test_null_callback
    test_socket_address
    test_volume_ffi
    test_rest
    test_info
    test_images
    test_registries
    test_ssh
    test_ssh_integration
)

foreach(TARGET ${TEST_TARGETS})
    add_executable(${TARGET} ${TARGET}.c)
    target_link_libraries(${TARGET} ${ACRYLMICROVM_LIB})
    target_compile_definitions(${TARGET} PRIVATE _XOPEN_SOURCE=700)

    # Set RPATH so the executable can find the dylib
    if(APPLE)
        set_target_properties(${TARGET} PROPERTIES
            BUILD_RPATH "${ACRYLMICROVM_LIB_DIR}"
            INSTALL_RPATH "@executable_path/../lib"
        )
    elseif(UNIX)
        set_target_properties(${TARGET} PROPERTIES
            BUILD_RPATH "${ACRYLMICROVM_LIB_DIR}"
            INSTALL_RPATH "$ORIGIN/../lib"
        )
    endif()
endforeach()

# Enable testing
enable_testing()

# Register tests
foreach(TARGET ${TEST_TARGETS})
    add_test(NAME ${TARGET} COMMAND ${TARGET})
endforeach()

# Label integration tests (require VM)
set_tests_properties(test_agents PROPERTIES LABELS "unit;ffi")
set_tests_properties(test_simple_api PROPERTIES LABELS "integration")
# test_null_callback only exercises the FFI argument-validation path;
# no VM is required, but we still link the runtime dylib.
set_tests_properties(test_null_callback PROPERTIES LABELS "unit;ffi")
set_tests_properties(test_socket_address PROPERTIES LABELS "unit;ffi")
# test_volume_ffi only exercises signatures and argument validation, so it
# needs no VM either.
set_tests_properties(test_volume_ffi PROPERTIES LABELS "unit;ffi")
# REST runtime construction does no I/O (lazy connect), so it is a
# true unit test — runs under `make test:unit:c` (ctest -L unit).
set_tests_properties(test_rest PROPERTIES LABELS "unit")
set_tests_properties(test_info PROPERTIES LABELS "unit")
# test_images serves the REST side from a forked loopback server and reads the
# local side from an empty cache, so it needs no VM either.
set_tests_properties(test_images PROPERTIES LABELS "unit")
# test_registries is the same: a forked loopback server and a local runtime
# that refuses the handle.
set_tests_properties(test_registries PROPERTIES LABELS "unit")
# test_ssh is the same: a forked loopback REST server carries configure,
# status, disable, and the refusal path, so it needs no VM either.
set_tests_properties(test_ssh PROPERTIES LABELS "unit")

set_tests_properties(test_ssh_integration PROPERTIES LABELS "integration" TIMEOUT 900)

message(STATUS "ACRYL microVMs C Tests Configuration:")
message(STATUS "  Include dir: ${ACRYLMICROVM_INCLUDE_DIR}")
message(STATUS "  Library: ${ACRYLMICROVM_LIB}")
message(STATUS "  Test targets: ${TEST_TARGETS}")
message(STATUS "")
message(STATUS "Prerequisite: make dev:c")
message(STATUS "")
message(STATUS "To build and run tests:")
message(STATUS "  mkdir -p build && cd build")
message(STATUS "  cmake ..")
message(STATUS "  make")
message(STATUS "  ctest --verbose")
