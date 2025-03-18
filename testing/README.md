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
- This framework tests the *release* build of the program in `target/release/`. You need to run
  `cargo build --release`
  to make sure you're testing the latest version
  
# Building and Running
To generate the tests, in the `testing/testGenerator/` folder run:
- `$ cmake -S . -B build` to generate the makefiles.
- `$ cmake --build build` to build testGenerator
- `$ ./testGenerator` to generate the test .cpp files

To build the tests, at the `testing/` directory run:
- `$ cmake -S . -B build` to generate the makefiles.
- `$ cmake --build build` to build the tests.

This builds test binaries in each of the following folders
- `loadStore/` load and store focused tests
- `branchJump/` branch, jump, auipc, and lui focused tests
- `integer/` integer register and immediate arithmetic and logic ops focused test

Inside one of those folders will be a binary called `<foldername>Test`, and to
run the test execute `$ ./<foldername>Test`.

Alternatively, you can use the provided scripts like so:

- `$ ./buildtests.sh`

to generate and build all the tests, and

- `$ ./runTests.sh`

to run them. This will by default run all available tests; you can specify `-i` if you want to pause after each one, or you can specify the folder name of the group of tests you wish to run.

# Adding Tests
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

If you've added a new group of test (ie a folder), need to reference your tests in the `main.cpp` file in `testing/testGenerator`. You can do this manually, or you can run the provided script:

- `$ ./addtest.sh [groupname]`

Use `./addtest.sh -d [groupname]` to remove a test.

After you have created your tests re-run the test generator, inside
`testing/testGenerator` run `./testGenerator`. 

# Failing Tests
All tests generate a mem image file and results files in `testResources/memImages`
and `testResources/results` respectively. These files are cleaned up at the end
of a passing test. Failed tests do not clean up their mem images or results 
files so that they can be inspected later to investigate why the test failed. 
