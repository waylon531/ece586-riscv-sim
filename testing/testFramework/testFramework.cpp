#include <string>
#include <fstream>
#include <cstdlib>
#include <iostream>
#include <cassert>
#include <cstdlib>
#include <sstream>
#include <regex>
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
    m_assemblyFileLocation = rootPath + "testing/" + m_instrType + "/testResources/assembly/" + m_testName + ".s";
    m_objFileLocation = rootPath + "testing/" + m_instrType + "/testResources/assembly/" + m_testName + ".out";
    m_disassemblyFileLocation = rootPath + "testing/" + m_instrType + "/testResources/assembly/" + m_testName + ".dis";

    generateMemImage();
}

testFramework::~testFramework()
{
    if(pass == true)
    {
        // clean up if test passed
        std::string temp = "rm -rf " + m_simResultFilename;
        system(temp.c_str());

        temp = "rm -rf " + m_memImageLocation;
        system(temp.c_str());

        temp = "rm -rf " + m_objFileLocation;
        system(temp.c_str());

        temp = "rm -rf " + m_disassemblyFileLocation;
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
// /opt/riscv/bin/riscv64-unknown-linux-gnu-as /home/matthew/ece586-riscv-sim/testing/load/testResources/assembly/load_byte_unsigned.s -o /home/matthew/Downloads/a.out
void testFramework::generateMemImage()
{
    m_assemblerPath = getPath("riscv64-unknown-linux-gnu-as");
    m_objdumpPath = getPath("riscv64-unknown-linux-gnu-objdump");
    std::string assemblyCmd = m_assemblerPath + " -march=rv32i -mabi=ilp32 " + m_assemblyFileLocation + " -o " + m_objFileLocation;
    std::string disassemblyCmd = m_objdumpPath + " -d " + m_objFileLocation + " > " + m_disassemblyFileLocation;

    system(assemblyCmd.c_str());
    system(disassemblyCmd.c_str());

    std::ifstream disassembly(m_disassemblyFileLocation.c_str(), std::ifstream::in);
    std::ofstream memImage(m_memImageLocation.c_str(), std::ofstream::out);

    // Check if the files were opened successfully
    if (!disassembly.good()) 
    {
        std::cerr << "Error opening disassembly file: "<< m_disassemblyFileLocation <<std::endl;
        assert(false);
    }
    if (!memImage.good()) 
    {
        std::cerr << "Error opening memImage file: "<< m_memImageLocation <<std::endl;
        assert(false);
    }

    std::string line;

    // This regex matches lines that start with optional whitespace,
    // followed by an address (digits) and a colon,
    // then some whitespace and a hexadecimal value.
    std::regex pattern("^\\s*(\\d+):\\s+([0-9a-fA-F]+)");

    while(std::getline(disassembly, line))
    {
        std::smatch match;
        // if line is empty, skip
        if(line.empty())
        {
            continue;
        }

        if (std::regex_search(line, match, pattern))
        {
            // match[1] contains the address (e.g., "0" or "4")
            // match[2] contains the hexadecimal code (e.g., "0ff00293")
            memImage << match[1] << ":   " << match[2] << "\n";
        }
    }
}

// /opt/riscv/bin/riscv64-unknown-linux-gnu-as -ahld prog.s
// rvobjdump -d prog.o > prog.dis
// matthew@sonOfAnton:~/ece586-riscv-sim/testing/load/testResources/assembly$ /opt/riscv/bin/riscv64-unknown-linux-gnu-as -march=rv32i -mabi=ilp32 load_byte_unsigned.s
// matthew@sonOfAnton:~/ece586-riscv-sim/testing/load/testResources/assembly$ /opt/riscv/bin/riscv64-unknown-linux-gnu-objdump -d a.out 
std::string testFramework::getPath(std::string filename)
{   
    std::string pathEnv = std::getenv("PATH");
    std::string dir;
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