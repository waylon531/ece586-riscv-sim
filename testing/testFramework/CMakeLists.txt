cmake_minimum_required(VERSION 3.23.1)
project(testFramework VERSION 1.0)
add_library(${PROJECT_NAME} testFramework.h testFramework.cpp)
# Make headers available to those that include this library
target_include_directories(${PROJECT_NAME} PUBLIC ${PROJECT_SOURCE_DIR})