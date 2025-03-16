#include "testGenerator.h"

int main()
{
    testGenerator loadStore("../loadStore/testResources/assembly", "../loadStore/loadStoreTest.cpp", "loadStore");
    testGenerator integer("../integer/testResources/assembly", "../integer/integerTest.cpp", "integer");
    testGenerator branchJump("../branchJump/testResources/assembly", "../branchJump/branchJumpTest.cpp", "branchJump");
    
    return 0;
}