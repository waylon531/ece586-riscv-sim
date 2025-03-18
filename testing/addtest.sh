#!/bin/sh
SOURCEFILE="main.cpp"
SOURCEDIR="testGenerator"
set -eo pipefail
[ "$1" == "-d" ] && T=$2 || T=$1
[ -e "$SOURCEFILE" ] || SOURCEFILE="$SOURCEDIR/$SOURCEFILE"
errout() {
  echo $1 >&2
  exit 1
}
window() {
  ceol=$(tput el)
  [ -z "$1" ] && n=5 || n=$1
  cursmov=$(tput cuu $n)
  buff=""
  c=0
  echo "========================================"
  while read line; do
    [ $c -gt $n ] && echo "$(tput init)========================================$(tput cuu 1)"
    if [ $c -gt $((n-1)) ]; then
      buff=$(printf '%b' "$buff" | sed '1d')
      printf '%s\n' "$cursmov$buff"
    fi
    line="$line$ceol"
    echo $line
    [ "$c" -eq 0 ] && buff="$line" || buff="$buff\n$line"
    c=$((c+1))
  done
  echo
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
  echo Successfully added test $T. Attempting build.
  unbuffer ./buildtests.sh 2>&1 | window 8 || errout "Failed to build your tests."
  echo Successfully rebuilt testGenerator with your new tests.
fi
)
if [ $? -ne 0 ]; then
  echo Failed to $op test $T
  exit 1
fi
