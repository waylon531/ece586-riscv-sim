#include <string>
#include <fstream>
#include <cstdlib>
#include <iostream>
#include <cassert>
#include "testFramework.h"

//cmake -S . -B build
//cmake --build build

testFramework::testFramework()
{

}

testFramework::testFramework(std::string testName)
{
    this->testName = testName;

    simResultFilename = "simResult_" + testName + ".txt";
    expectedResultFilename = "testResources/expected/expected_" + testName + ".txt";
}

testFramework::~testFramework()
{
    if(pass == true)
    {
        // clean up if test passed
        std::string temp = "rm -rf " + simResultFilename;
        system(temp.c_str());
    }
}

bool testFramework::run()
{
    std::string cmd = "./riscvSim testResources/memImages/" + testName + ".mem > " + simResultFilename;
    system(cmd.c_str());
    parseResult();
    return pass;
}

void testFramework::parseResult()
{
    // Open the two files
    std::ifstream simResult(simResultFilename.c_str(), std::ifstream::in);
    std::ifstream expectedResult(expectedResultFilename.c_str(), std::ifstream::in);
    std::string simResultLine;
    std::string expectedResultLine;

    // Check if the files were opened successfully
    if (!simResult) 
    {
        std::cerr << "Error opening simResult\n";
        assert(false);
    }
    if (!expectedResult) 
    {
        std::cerr << "Error opening expectedResult\n";
        assert(false);
    }

    while(std::getline(simResult, simResultLine) && std::getline(expectedResult, expectedResultLine))
    {
        if(simResultLine != expectedResultLine)
        {
            break;
        }
        else
        {
            pass = true;
        }
    }
}