#include <string>
#include <fstream>
#include <cstdlib>
#include <iostream>
#include <cassert>
#include "testFramework.h"

// pre-req is cmake installed. Simple sudo apt install cmake should work
// in testing dir
// cmake -S . -B build
// cmake --build build
// go to load directory
// ./loadTest
// executes entire test suite

testFramework::testFramework()
{

}

testFramework::testFramework(std::string testName, std::string instrType)
{
    m_testName = testName;
    m_instrType = instrType;

    m_simResultFilename = "~/ece586-riscv-sim/testing/" + m_instrType + "/simResult_" + m_testName + ".txt";
    m_expectedResultFilename = "testResources/expected/expected_" + m_testName + ".txt";
    m_memImageLocation = "~/ece586-riscv-sim/testing/" + m_instrType + "/testResources/memImages/" + m_testName + ".mem";
}

testFramework::~testFramework()
{
    if(pass == true)
    {
        // clean up if test passed
        std::string temp = "rm -rf " + m_simResultFilename;
        system(temp.c_str());
    }
}

bool testFramework::run()
{
    std::string cmd = m_simBinaryLocation + " "+ m_memImageLocation + " --dump-to " + m_simResultFilename;
    system(cmd.c_str());
    parseResult();
    return pass;
}

void testFramework::parseResult()
{
    // Open the two files
    std::ifstream simResult(m_simResultFilename.c_str(), std::ifstream::in);
    std::ifstream expectedResult(m_expectedResultFilename.c_str(), std::ifstream::in);
    std::string simResultLine;
    std::string expectedResultLine;

    // Check if the files were opened successfully
    std::cout<<"simResult: "<<simResult.good()<<std::endl;
    std::cout<<"expectedResult: "<<expectedResult.good()<<std::endl;
    if (!simResult.good()) 
    {
        std::cerr << "Error opening simResult\n";
        assert(false);
    }
    if (!expectedResult.good()) 
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

    simResult.close();
    expectedResult.close();
}