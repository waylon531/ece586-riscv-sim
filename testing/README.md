# Prerequisites
This framework requires a few things
- RISC V toolchain. This can be cloned and installed from [here](https://github.com/riscv-collab/riscv-gnu-toolchain)
  - The toolchain MUST be added to your path, for example: if you have the toolchain installed to the default location then append
  
    `export PATH="/opt/riscv/bin/riscv64-unknown-linux-gnu-as:$PATH"` and 

    `export PATH="/opt/riscv/bin/riscv64-unknown-linux-gnu-objdump:$PATH"`

    To the bottom of your .bashrc file and then run

    `$ source .bashrc`
- GCC build tools, make etc...
- Cmake, at least version 3.23.1
# Building and Running
To build the tests, at the `testing/` directory run:
- `$ cmake -S . -B build` to generate the makefiles.
- `$ cmake --build build` to build the tests.

This builds test binaries in each of the following folders
- `loadStore/` load and store focused tests
- `branchJump/` branch, jump, auipc, and lui focused tests
- `integer/` integer register and immediate arithmetic and logic ops focused test

Inside one of those folders will be a binary called `<foldername>Test`, and to
run the test execute `$ ./<foldername>Test`.

# Adding Tests
## The Overhead
The test framework uses gtest as a driver so we will be following gtest 
conventions. Gtest is an industry standard unit testing driver and framework. It
is typically used for software unit tests, but I have found luck with it for 
creating a nice interface for verilog testbenches (given a custom framework that 
runs and parses those testbenches). More on gtest can be found here:
* https://github.com/google/googletest
* https://google.github.io/googletest/

Inside each of the folders above will be a file called `<folername>Test.cpp`.
There should be at least one test that you can use as an example, but I will
cover it here.

1. Add a function called `TEST(<foldername>, some_descriptive_name)`
   It wont break anything but please keep the convention of the file using underscores and not camelcase.
2. Inside that function, create a test framework object `testFramework framework(simBinaryLocation, rootPath, "some_descriptive_name", "<foldername>");`.
   If you dont keep this convention it will break the pathing and your test wont 
   compile or run correctly. You must also keep your test name `some_descriptive_name` 
   consistent, or your test will break.
3. Currently the framework only returns true if the test passes. Add `EXPECT_TRUE(framework.run());`
   To the end of your test function to check if your test passes and add it to 
   the final test results.
## The Real Meat of the Test
The part that actually does the testing will be your assembly and expected 
results files. Inside each `load/`, `store/`, `integer/`, etc... folder (that
replaces the placeholder `<foldername>`) are two other folders called
* `testResources/assembly`
* `testResources/expected`

Inside `testResources/assembly` will be the assembly code for your test that the
simulator will execute. Inside `testResources/expected` will be the expected 
register and Program Counter values. Your assembly code file **MUST** be called `some_descriptive_name.s`
and your expected results file **MUST** be called `some_descriptive_name.txt`. If 
your files are not named that, then your test will break. 

# Failing Tests
All tests generate a mem image file and results files in `testResources/memImages`
and `testResources/results` respectively. These files are cleaned up at the end
of a passing test. Failed tests do not clean up their mem images or results 
files so that they can be inspected later to investigate why the test failed. 