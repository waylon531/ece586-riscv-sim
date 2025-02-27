#include <string>
#include <gtest/gtest.h>
#include "testFramework.h"

TEST(loadTest, load_byte_unsigned)
{
    testFramework framework("load_byte_unsigned", "load");

    // true if output matches expected
    EXPECT_TRUE(framework.run());
}