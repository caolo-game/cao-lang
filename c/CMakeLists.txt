cmake_minimum_required(VERSION 3.18)

project(
    cao_lang
    VERSION 0.1.0
    LANGUAGES C CXX)

include(ExternalProject)

option(CAOLO_ENABLE_TESTING "Enable testing" OFF)

add_library(cao_lang INTERFACE)
target_include_directories(cao_lang INTERFACE ${CMAKE_SOURCE_DIR}/include)
if(WIN32)
    target_link_libraries(
        cao_lang
        INTERFACE debug "${CMAKE_SOURCE_DIR}/../target/debug/cao_lang_c.lib"
                  optimized
                  "${CMAKE_SOURCE_DIR}/../target/release/cao_lang_c.lib")
else()
    target_link_libraries(
        cao_lang
        INTERFACE debug "${CMAKE_SOURCE_DIR}/../target/debug/libcao_lang_c.a"
                  optimized
                  "${CMAKE_SOURCE_DIR}/../target/release/libcao_lang_c.a")
endif()

ExternalProject_Add(
    cao_lang_rs
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo build
    COMMAND cargo build --release
    BINARY_DIR "${CMAKE_SOURCE_DIR}"
    INSTALL_COMMAND ""
    LOG_BUILD ON)

add_dependencies(cao_lang cao_lang_rs)

if(CAOLO_ENABLE_TESTING)
    enable_testing()
    list(APPEND CMAKE_MODULE_PATH ${CMAKE_CURRENT_LIST_DIR}/cmake)
    set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

    add_subdirectory(test)
endif()
