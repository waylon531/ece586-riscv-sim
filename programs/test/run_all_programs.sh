#!/bin/bash

FAILURES=0

for program in programs/*.mem; do
    echo "--------------------"
    echo "Running $program"
    cargo run -- --suppress-status "$program"
    FAILURES=$((FAILURES + $?))
    echo "--------------------"
done

exit $FAILURES
