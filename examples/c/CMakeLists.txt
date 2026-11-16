cmake_minimum_required(VERSION 3.15)
project(acrylmicrovm_c_examples C)

set(CMAKE_C_STANDARD 11)

# Find the ACRYL microVMs library
# Expected location: ../../target/release/libacrylmicrovm.{a,dylib,so}
set(ACRYLMICROVM_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../..")
set(ACRYLMICROVM_INCLUDE_DIR "${ACRYLMICROVM_ROOT}/sdks/c/include")
set(ACRYLMICROVM_LIB_DIR "${ACRYLMICROVM_ROOT}/target/release")

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
    message(FATAL_ERROR "ACRYL microVMs library not found at ${ACRYLMICROVM_LIB}. Please run: cargo build --release -p acrylmicrovm-c")
endif()

# Check if header exists
if(NOT EXISTS "${ACRYLMICROVM_INCLUDE_DIR}/acrylmicrovm.h")
    message(FATAL_ERROR "ACRYL microVMs header not found at ${ACRYLMICROVM_INCLUDE_DIR}/acrylmicrovm.h. Please run: cargo build --release -p acrylmicrovm-c")
endif()

# Include directories
include_directories(${ACRYLMICROVM_INCLUDE_DIR})

# Example executables
set(EXAMPLE_TARGETS
    execute
    shutdown
    simple_api_demo
    01_lifecycle
    02_list_boxes
    03_streaming_output
    04_error_handling
    05_metrics
)

foreach(TARGET ${EXAMPLE_TARGETS})
    add_executable(${TARGET} ${TARGET}.c)
    target_link_libraries(${TARGET} ${ACRYLMICROVM_LIB})
endforeach()

# Set RPATH so the executables can find the dylib
if(APPLE)
    set_target_properties(${EXAMPLE_TARGETS} PROPERTIES
        BUILD_RPATH "${ACRYLMICROVM_LIB_DIR}"
        INSTALL_RPATH "@executable_path/../lib"
    )
elseif(UNIX)
    set_target_properties(${EXAMPLE_TARGETS} PROPERTIES
        BUILD_RPATH "${ACRYLMICROVM_LIB_DIR}"
        INSTALL_RPATH "$ORIGIN/../lib"
    )
endif()

# Print instructions
message(STATUS "ACRYL microVMs C Examples Configuration:")
message(STATUS "  Include dir: ${ACRYLMICROVM_INCLUDE_DIR}")
message(STATUS "  Library: ${ACRYLMICROVM_LIB}")
message(STATUS "")
message(STATUS "To build and run:")
message(STATUS "  mkdir -p build && cd build")
message(STATUS "  cmake ..")
message(STATUS "  make")
message(STATUS "  ./execute")
