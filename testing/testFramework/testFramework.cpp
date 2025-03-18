#include <string>
#include <fstream>
#include <cstdlib>
#include <iostream>
#include <cstdlib>
#include <sstream>
#include <regex>
#include <cstdlib>
#include "testFramework.h"

testFramework::testFramework()
{
    
}

testFramework::testFramework(std::string simBinaryLocation, std::string rootPath, std::string testName, std::string instrType)
{
    m_testName = testName;
    m_instrType = instrType;
    m_simBinaryLocation = simBinaryLocation;
    m_rootPath = rootPath;

    m_simResultFilename = m_rootPath + "testing/" + m_instrType + "/testResources/results/" + m_testName + ".txt";
    m_expectedResultFilename = m_rootPath + "testing/" + m_instrType +"/testResources/expected/" + m_testName + ".txt";
    m_memImageLocation = rootPath + "testing/" + m_instrType + "/testResources/memImages/" + m_testName + ".mem";
    m_assemblyFileLocation = rootPath + "testing/" + m_instrType + "/testResources/assembly/" + m_testName + ".s";
    m_objFileLocation = rootPath + "testing/" + m_instrType + "/testResources/assembly/" + m_testName + ".out";
    m_disassemblyFileLocation = rootPath + "testing/" + m_instrType + "/testResources/assembly/" + m_testName + ".dis";

    generateMemImage();
}

testFramework::testFramework(std::string testName, std::string instrType)
{
    const std::string homedir = std::getenv("HOME");
    const std::string simBinaryLocation = homedir + "/ece586-riscv-sim/target/release/ece586-riscv-sim";
    const std::string rootPath = homedir + "/ece586-riscv-sim/";

    m_testName = testName;
    m_instrType = instrType;
    m_simBinaryLocation = simBinaryLocation;
    m_rootPath = rootPath;

    m_simResultFilename = m_rootPath + "testing/" + m_instrType + "/testResources/results/" + m_testName + ".txt";
    m_expectedResultFilename = m_rootPath + "testing/" + m_instrType +"/testResources/expected/" + m_testName + ".txt";
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
    std::string cmd = m_simBinaryLocation + " "+ m_memImageLocation + " --dump-to " + m_simResultFilename + " --quiet -s 65536";
    system(cmd.c_str());
    parseResult();
    return pass;
}
// me when c+ has no builtin way to uppercase or lowercase strings
bool ichar_equals(char a, char b)
{
    return std::tolower(static_cast<unsigned char>(a)) ==
           std::tolower(static_cast<unsigned char>(b));
}
bool iequals(const std::string& a, const std::string& b)
{
    return std::equal(a.begin(), a.end(), b.begin(), b.end(), ichar_equals);
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
        std::exit(EXIT_FAILURE);
    }
    if (!expectedResult.good()) 
    {
        std::cerr << "Error opening expectedResult\n";
        std::cerr << "Is the the test result name correctly?\n";
        std::exit(EXIT_FAILURE);
    }

    while(std::getline(simResult, simResultLine) && std::getline(expectedResult, expectedResultLine))
    {
        if(!iequals(simResultLine,expectedResultLine))
        {
            pass = false;
            std::cout<<"Expected results is: "<<expectedResultLine<<std::endl;
            std::cout<<"Result from sim is: "<<simResultLine<<std::endl;
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
    //m_assemblerPath = "riscv64-unknown-elf-as";
    //m_objdumpPath = "riscv64-unknown-elf-objdump";
    //std::string riscvPath = getPath("([^:]*riscv/bin)(?=:|$)");
    // export RISCV_PATH=/opt/riscv/bin/
    std::string riscvPath = std::getenv("RISCV_PATH");
    std::string assemblyCmd = riscvPath + "riscv*as -march=rv32i -mabi=ilp32 " + m_assemblyFileLocation + " -o " + m_objFileLocation;
    std::string disassemblyCmd = riscvPath + "riscv*objdump -d " + m_objFileLocation + " > " + m_disassemblyFileLocation;

    system(assemblyCmd.c_str());
    system(disassemblyCmd.c_str());

    std::ifstream disassembly(m_disassemblyFileLocation.c_str(), std::ifstream::in);
    std::ofstream memImage(m_memImageLocation.c_str(), std::ofstream::out);

    // Check if the files were opened successfully
    if (!disassembly.good()) 
    {
        std::cerr << "Error opening disassembly file: "<< m_disassemblyFileLocation <<std::endl;
        std::exit(EXIT_FAILURE);
    }
    if (!memImage.good()) 
    {
        std::cerr << "Error opening memImage file: "<< m_memImageLocation <<std::endl;
        std::exit(EXIT_FAILURE);
    }

    std::string line;

    // This regex matches lines that start with optional whitespace,
    // followed by an address (digits) and a colon,
    // then some whitespace and a hexadecimal value.
    std::regex pattern("^\\s*([0-9a-fA-F]+:)\\s+([0-9a-fA-F]{8})");

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
            memImage << match[1] << "   " << match[2] << "\n";
        }
    }
}

std::string testFramework::getPath(std::string fileName)
{       
    std::string pathEnv = std::getenv("PATH");
    std::string dir = "";

    if(pathEnv.empty())
    {
        std::cerr<<"PATH environment variable not found"<<std::endl;
        std::exit(EXIT_FAILURE); // kill execution
    }
    std::regex pattern(fileName);
    std::smatch match;
    if(std::regex_search(pathEnv, match, pattern))
    {
        dir = match[1];
    }

    return dir;
}
