#include <string>

class testFramework
{
    public:
        testFramework();
        testFramework(std::string testName, std::string instrType);
        ~testFramework();

        bool run();
    
    private:
        /**
         * @brief compare line by line through the simResult and expectedResult
        */
        void parseResult();
        std::string testName;
        bool pass = false;

        std::string simResultFilename;
        std::string expectedResultFilename;
        std::string m_instrType;
};