cmake_minimum_required(VERSION 3.18)

project(
    cao_lang
    VERSION 0.1.0
    LANGUAGES C CXX)

include(ExternalProject)

option(CAOLO_ENABLE_TESTING "Enable testing" OFF)

add_library(cao_lang INTERFACE)
target_include_directories(cao_lang
                           INTERFACE ${CMAKE_CURRENT_SOURCE_DIR}/c/include)

# Honor the cargo dir env variable if present
set(CAO_TARGET_DIR $ENV{CARGO_TARGET_DIR})
if(NOT EXISTS ${CAO_TARGET_DIR}) 
    set(CAO_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/target")
endif()

if(WIN32)
    target_link_libraries(
        cao_lang
        INTERFACE debug
                  "${CAO_TARGET_DIR}/debug/cao_lang_c.lib"
                  optimized
                  "${CAO_TARGET_DIR}/release/cao_lang_c.lib")
else()
    target_link_libraries(
        cao_lang
        INTERFACE debug
                  "${CAO_TARGET_DIR}/debug/libcao_lang_c.a"
                  optimized
                  "${CAO_TARGET_DIR}/release/libcao_lang_c.a")
endif()

ExternalProject_Add(
    cao_lang_rs
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND $<IF:$<CONFIG:Debug>,cargo build --package ccao-lang,cargo build --package ccao-lang --release >
    BINARY_DIR "${CMAKE_CURRENT_SOURCE_DIR}"
    INSTALL_COMMAND ""
    LOG_BUILD ON
    BUILD_BYPRODUCTS
        "${CAO_TARGET_DIR}/release/libcao_lang_c.a"
        "${CAO_TARGET_DIR}/debug/libcao_lang_c.a"
        "${CAO_TARGET_DIR}/release/cao_lang_c.lib"
        "${CAO_TARGET_DIR}/debug/cao_lang_c.lib")

add_dependencies(cao_lang cao_lang_rs)

if(CAOLO_ENABLE_TESTING)
    enable_testing()
    list(APPEND CMAKE_MODULE_PATH ${CMAKE_CURRENT_LIST_DIR}/c/cmake)
    set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

    add_subdirectory(c/test)
endif()
