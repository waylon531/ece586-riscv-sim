#include <fstream>
#include <iostream>
#include <filesystem>
#include <string>
#include <vector>
#include <cstdlib>

#include "testGenerator.h"

testGenerator::testGenerator()
{

}
testGenerator::testGenerator(std::string path, std::string testName, std::string instrType)
{
    m_path = path;
    m_testName = testName;

    std::filesystem::path folderPath = path;

    if (std::filesystem::exists(folderPath) && std::filesystem::is_directory(folderPath))
    {
        for (const auto& entry : std::filesystem::directory_iterator(folderPath))
        {
            if (std::filesystem::is_regular_file(entry.status()) && (entry.path().extension() == ".s"))
            {
                m_tests.push_back(entry.path().stem().string());
            }
        }
    }

    std::ofstream file(m_testName);
    if (!file) 
    {
        std::cerr << "Error: Could not create the file." << std::endl;
        std::exit(EXIT_FAILURE);
    }
    file << "// This is a generated file. Do not add to git."<<std::endl;
    file << m_fileHeader;
    file << m_fileConsts;

    for(uint32_t iter = 0; iter < m_tests.size(); iter++)
    {
        file << "TEST("<<instrType<<"Test, " << m_tests.at(iter)<<")"<<std::endl;
        file << "{"<<std::endl;
        file << "    testFramework framework(simBinaryLocation, rootPath, \""<<m_tests.at(iter)<<"\", \""<<instrType<<"\");"<<std::endl;
        file << "    EXPECT_TRUE(framework.run());"<<std::endl;
        file << "}"<<std::endl;
    }

    file.close();
}
testGenerator::~testGenerator()
{

}