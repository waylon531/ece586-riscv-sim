#include <string>
#include <vector>

class testGenerator
{
    public:
        testGenerator();
        /**
         * @param path to the assembly folder
         * @param testName is path/to/test/location/testName.cpp
         * @param instrType of what kind of test it is, ie loadStore, BranchJump, or integer
        */
        testGenerator(std::string path, std::string testName, std::string instrType);
        ~testGenerator();

    private:
        std::string m_testName;
        std::string m_path;
        const std::string m_fileHeader = "#include <string>\n"
                                       "#include <iostream>\n"
                                       "#include <cstdlib>\n"
                                       "#include <gtest/gtest.h>\n"
                                       "#include \"testFramework.h\"\n"
                                       "\n";
        const std::string m_fileConsts = "const std::string homedir = std::getenv(\"HOME\");\n"
                                       "const std::string simBinaryLocation = homedir + \"/ece586-riscv-sim/target/release/ece586-riscv-sim\";\n"
                                       "const std::string rootPath = homedir + \"/ece586-riscv-sim/\";\n"
                                       "\n";

        std::vector<std::string> m_tests;

};