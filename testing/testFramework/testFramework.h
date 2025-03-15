#include <string>

class testFramework
{
    public:
        /**
         * @brief default constructor
        */
        testFramework();
        /**
         * @brief construct test framework with paths, testname and test folder
         * @param simBinaryLocation is the location of the simulation
         * @param rootPath is the location of the location of the root of the repository
         * @param testName is the name of the test
         * @param is the instructionType, the overal category folder it resides under
        */
        testFramework(std::string simBinaryLocation, std::string rootPath, std::string testName, std::string instrType);
        /**
         * @brief deconstructor, cleans up the files created during the test
        */
        ~testFramework();

        bool run();
    
    private:
        /**
         * @brief compare line by line through the simResult and expectedResult
        */
        void parseResult();
        /**
         * @brief parses the .dis disassembly file and generates a mem image file
        */
        void generateMemImage();
        /**
         * @brief finds the paths of the of the specified file in your $PATH variable
         * @param fileName is a regex pattern for the file you want to find in your $PATH
        */
        std::string getPath(std::string fileName);
        std::string m_testName;
        bool pass = false;

        std::string m_simResultFilename;
        std::string m_expectedResultFilename;
        std::string m_instrType;
        std::string m_memImageLocation;
        std::string m_simBinaryLocation;
        std::string m_rootPath;
        std::string m_assemblerPath;
        std::string m_objdumpPath;
        std::string m_assemblyFileLocation;
        std::string m_objFileLocation;
        std::string m_disassemblyFileLocation;
};