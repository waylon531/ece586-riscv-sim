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
        std::string m_testName;
        bool pass = false;

        std::string m_simResultFilename;
        std::string m_expectedResultFilename;
        std::string m_instrType;
        std::string m_memImageLocation;
        const std::string m_simBinaryLocation = "~/ece586-riscv-sim/target/release/ece586-riscv-sim";
};