#!/bin/bash
export RUSTFLAGS=-Awarnings
FAILURES=0

FAILED=""
for program in programs/test/*.mem; do
    echo "--------------------"
    echo "Running $program"
    cargo run -- --suppress-status "$program" 2> /tmp/errors
    EXIT=$?
    if [[ $EXIT -ne 0 ]]; then
      FAILED="$FAILED $program"
    fi
    FAILURES=$((FAILURES + $EXIT))
    echo "--------------------"
done
echo "Failing tests: $FAILED"
cat /tmp/errors
exit $FAILURES
