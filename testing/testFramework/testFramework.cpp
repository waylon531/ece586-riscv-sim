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

testFramework::testFramework(std::string simBinaryLocation, std::string rootPath, std::string testName, std::string instrType)
{
    m_testName = testName;
    m_instrType = instrType;
    m_simBinaryLocation = simBinaryLocation;
    m_rootPath = rootPath;

    m_simResultFilename = m_rootPath + "testing/" + m_instrType + "/simResult_" + m_testName + ".txt";
    m_expectedResultFilename = m_rootPath + "testing/" + m_instrType +"/testResources/expected/expected_" + m_testName + ".txt";
    m_memImageLocation = rootPath + "testing/" + m_instrType + "/testResources/memImages/" + m_testName + ".mem";
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