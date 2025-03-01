#include <string>

class testFramework
{
    public:
        testFramework();
        testFramework(std::string simBinaryLocation, std::string rootPath, std::string testName, std::string instrType);
        ~testFramework();

        bool run();
    
    private:
        /**
         * @brief compare line by line through the simResult and expectedResult
        */
        void parseResult();
        void generateMemImage();
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