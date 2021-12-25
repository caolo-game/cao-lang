include(FetchContent)

FetchContent_Declare(
  googletest
  URL "https://github.com/google/googletest/archive/release-1.11.0.tar.gz")

# Prevent overriding the parent project's compiler/linker settings on Windows
set(gtest_force_shared_crt
    ON
    CACHE BOOL "" FORCE)

set(GTEST_CREATE_SHARED_LIBRARY OFF)
set(GTEST_LINKED_AS_SHARED_LIBRARY OFF)
FetchContent_MakeAvailable(googletest)

FetchContent_GetProperties(googletest)
if(NOT googletest_POPULATED)
  FetchContent_Populate(googletest)
  add_subdirectory(${googletest_SOURCE_DIR} ${googletest_BINARY_DIR})
endif()
