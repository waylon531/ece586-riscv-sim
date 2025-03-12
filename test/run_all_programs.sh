#!/bin/bash

FAILURES=0

for program in progams/*.mem; do
    echo "--------------------"
    echo "Running $program"
    cargo run -- --suppress-status "$program"
    FAILURE=$((FAILURE + $?))
    echo "--------------------"
done

exit $FAILURES
