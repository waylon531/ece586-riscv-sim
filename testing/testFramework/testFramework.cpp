#include <string>
#include <fstream>
#include <cstdlib>
#include <iostream>
#include <cassert>
#include <cstdlib>
#include <filesystem>
#include <sstream>
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
    generateMemImage();
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

void testFramework::generateMemImage()
{
    m_assemblerPath = getPath("riscv64-unknown-linux-gnu-as");
    std::cout<<"m_assemblerPath: "<<m_assemblerPath<<std::endl;
    m_objdumpPath = getPath("riscv64-unknown-linux-gnu-objdump");
    std::cout<<"m_objdumpPath: "<<m_objdumpPath<<std::endl;
}

// /opt/riscv/bin/riscv64-unknown-linux-gnu-as -ahld prog.s
// rvobjdump -d prog.o > prog.dis
// matthew@sonOfAnton:~/ece586-riscv-sim/testing/load/testResources/assembly$ /opt/riscv/bin/riscv64-unknown-linux-gnu-as -march=rv32i -mabi=ilp32 load_byte_unsigned.s
// matthew@sonOfAnton:~/ece586-riscv-sim/testing/load/testResources/assembly$ /opt/riscv/bin/riscv64-unknown-linux-gnu-objdump -d a.out 
std::string testFramework::getPath(std::string filename)
{   
    std::string pathEnv = std::getenv("PATH");
    std::string dir;
    std::filesystem::path filePath;
    bool found = false;

    if(pathEnv.empty())
    {
        std::cerr<<"PATH enviroment variable not found"<<std::endl;
        assert(false); // kill execution
    }

    std::istringstream iss(pathEnv);
    while (std::getline(iss, dir, ':'))
    {   
        if(dir.find(filename) != std::string::npos)
        {
            found = true;
            break;
        }
    }

    if (!found) 
    {
        std::cout << "Required binary '" << filename << "' not found in PATH." << std::endl;
        std::cout << "Please add it to PATH (probably in your .bashrc file) and try again"<<std::endl;
        std::cout << "Hint, add something that should look like: export PATH=\"/opt/riscv/bin/riscv64-unknown-linux-gnu-as:$PATH\" to the end of your .bashrc file" << std::endl;
        assert(false);
    }

    return dir;
}