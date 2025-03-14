#include <iostream>
#include <cstdlib>
#include <gtest/gtest.h>
#include "testFramework.h"

const std::string homedir = std::getenv("HOME");
const std::string simBinaryLocation = "~/ece586-riscv-sim/target/release/ece586-riscv-sim";
const std::string rootPath = homedir + "/ece586-riscv-sim/";

TEST(branchJumpTest, register_add)
{
    testFramework framework(simBinaryLocation, rootPath, "branch_on_equal", "branchJump");

    // true if output matches expected
    EXPECT_TRUE(framework.run());
}