find_package(Threads)
find_package(GoogleTest REQUIRED)
include(GoogleTest)

add_executable(cao_lang_tests test_compile.cpp)

set_property(TARGET cao_lang_tests PROPERTY CXX_STANDARD 17)
set_property(TARGET cao_lang_tests PROPERTY CXX_STANDARD_REQUIRED ON)

target_link_libraries(cao_lang_tests cao_lang gtest gtest_main)

gtest_discover_tests(cao_lang_tests)

if(NOT WIN32 AND NOT APPLE)
    target_link_options(cao_lang_tests PRIVATE -Wl,--no-as-needed -ldl -pthread)
endif()
