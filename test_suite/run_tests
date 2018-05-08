#!/bin/bash

TEST_DIR=suite
EXECUTABLE=cannoli
RUST_CRATE=sandbox
PYTHON=python3

run_test() {
   f=$1

   # if the test.skip file exists then we won't run the test
   if [[ -e "$f"/test.skip ]]; then
      printf "\n\033[33;93m"
      echo "'$f' contains 'test.skip' and will be skipped..."
      printf "\033[0m"
      return
   fi

   # remove all out files first
   rm $TEST_DIR/test*/*.out

   printf "\n\033[33;93m"
   echo "Testing: '$f'"
   printf "\033[0m"

   printf "\033[33;94m"
   echo -n "INFO: "
   cat "$f"/info.txt
   printf "\033[0m"

   $PYTHON "$f"/test.py > "$f"/expected.out

   ./$EXECUTABLE "$f"/test.py > /dev/null
   STATUS=$?
   if [[ $STATUS -ne 0 ]]; then
      printf "\033[33;31m"
      printf "[COMPILER ERROR]\n"
      printf "\033[0m"
      return
   fi

   mv "$f"/main.rs $RUST_CRATE/src/
   cd $RUST_CRATE
   cargo build --release 2>/dev/null
   STATUS=$?
   if [[ $STATUS -ne 0 ]]; then
      printf "\033[33;31m"
      printf "$f build failed\n\n"
      printf "\033[0m"
      return
   fi
   cd ..

   $RUST_CRATE/target/release/$RUST_CRATE > "$f"/test.out
   STATUS=$?
   if [[ $STATUS -eq 0 ]]; then
      printf "\033[33;32m"
      printf "Test output normally.\n"
      printf "\033[0m"
   else
      printf "\033[33;31m"
      printf "$f exited with a non-zero exit status.\n"
      printf "\033[0m"
   fi

   diff "$f"/expected.out "$f"/test.out > /dev/null
   STATUS=$?
   if [[ $STATUS -eq 0 ]]; then
      printf "diff:\033[33;32m"
      printf "[SUCCESS]\n"
      printf "\033[0m"
   else
      printf "diff:\033[33;31m"
      printf "[FAILED]\n"
      printf "\033[0m"
   fi
}

# Run tests or single test if specified
mkdir -p $RUST_CRATE/src
if [[ -z "$1" ]]; then
   for f in $TEST_DIR/*; do
      if [[ -d $f ]]; then
         run_test "$f"
      fi
   done
else
   run_test "$TEST_DIR/$1"
fi

