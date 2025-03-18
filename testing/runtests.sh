#!/bin/sh
case "$1" in
  ""|"-i")
      notfound=1
      status=0
      failed=""
      for test in *; do
        command="$test/${test}Test"
        if [ -x $command ]; then
          notfound=0
          $command
          if [ $? -ne 0 ]; then
            status=$((status+1))
            failed="$failed $test"
          fi
          printf "test finished for $test"
          [ "$1" = "-i" ] && (echo ", press enter to continue"; read) || echo
        fi
      done
      if [ $status -ne 0 ]; then
        echo "the following tests failed:$failed"
        exit $status
      elif [ $notfound -ne 0 ]; then
          echo "no tests found"
          exit 1
      fi
    ;;
  *)
    if [ ! -x "$1/$1Test" ]; then
    echo "No such test exists: $1" >&2
      exit 1
    fi
    $1/$1Test
    exit $?
    ;;
esac
