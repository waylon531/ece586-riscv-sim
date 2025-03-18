#!/bin/sh
SOURCEFILE="main.cpp"
SOURCEDIR="testGenerator"
set -e
[ "$1" == "-d" ] && T=$2 || T=$1
[ -e "$SOURCEFILE" ] || SOURCEFILE="$SOURCEDIR/$SOURCEFILE"
errout() {
  echo $1 >&2
  exit 1
}
(
[ -e "$SOURCEFILE" ] || errout "Could not find $SOURCEFILE"
[ ! -z "$T" ] || errout "No test specified."
[ -d "$T/" ] || [ -d "../$T/" ] || errout "Could not find your tests."
[ -d "$T/testResources/assembly" ] || [ -d "../$T/testResources/assembly" ] || errout "Could not find your assembly files."
[ -d "$T/testResources/expected" ] || [ -d "../$T/testResources/expected" ] || errout "Could not find your expected results."

if [ "$1" == "-d" ]; then
  OP="remove"
  grep "## TEST: $T" $SOURCEFILE >/dev/null || errout "Could not find test $T in $SOURCEFILE"
  sed -i '' "/## TEST: $T/,+1d" $SOURCEFILE
  echo Successfully deleted test $T
else
  grep "## TEST: $T" $SOURCEFILE >/dev/null && errout "Test $T already in $SOURCEFILE"
  OP="add"
  sed -i '' "1,/{/s/{/{\n    \/\/\#\# TEST: $T\n    testGenerator $T(\"..\/$T\/testResources\/assembly\", \"..\/$T\/$TTest.cpp\", \"$T\");/" $SOURCEFILE
  echo Successfully added test $T
fi
)
if [ $? -ne 0 ]; then
  echo Failed to $op test $T
  exit 1
fi
