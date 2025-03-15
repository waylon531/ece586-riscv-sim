#include <string>
#include <iostream>
#include <cstdlib>
#include <gtest/gtest.h>
#include "testFramework.h"

const std::string homedir = std::getenv("HOME");
const std::string simBinaryLocation = homedir + "/ece586-riscv-sim/target/release/ece586-riscv-sim";
const std::string rootPath = homedir + "/ece586-riscv-sim/";

TEST(loadStoreTest, load_byte_unsigned)
{
    testFramework framework(simBinaryLocation, rootPath, "load_byte_unsigned", "loadStore");

    // true if output matches expected
    EXPECT_TRUE(framework.run());
}

TEST(loadStoreTest, load_empty_byte_unsigned)
{
    testFramework framework(simBinaryLocation, rootPath, "load_empty_byte_unsigned", "loadStore");

    // true if output matches expected
    EXPECT_TRUE(framework.run());
}