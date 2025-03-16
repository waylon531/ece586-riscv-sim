cmake_minimum_required(VERSION 3.23.1)

# Set the project name and version
project(testGenerator VERSION 1.0.0  LANGUAGES CXX)

# Add the executable target and list the source files.
add_executable(${PROJECT_NAME} main.cpp testGenerator.cpp)
target_compile_options(${PROJECT_NAME} PRIVATE -c -g -std=c++17 -Wall -W -Werror -pedantic)
set_target_properties(${PROJECT_NAME} PROPERTIES RUNTIME_OUTPUT_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR})

# Optionally, specify the C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)
