#!/bin/bash

# Get all examples in the current directory (i.e., all Rust source files)
files=(*.rs)

testCounter=0
failedCounter=0
failed=false

# Arrays to store names of the tests that failed in debug and release modes
failedArrDebug=()
failedArrRelease=()

# We build examples first to generate build artifacts so we can quickly run tests
echo "Building examples... [debug]"
timeout 3m cargo build --features test-parser
if [ $? -eq 1 ]; then
    echo "Failed to build examples [debug]"
    exit 1
fi
echo "Building examples... [release]"
timeout 3m cargo build --release --features test-parser
if [ $? -eq 1 ]; then
    echo "Failed to build examples [release]"
    exit 1
fi

# Loop through each .rs file (representing individual tests)
for file in "${files[@]}"; do

    # Strip the ".rs" extension from the filename to use it as the test name
    strippedName="${file%.rs}"
    ((testCounter++))
    echo "Executing test ${testCounter}: ${strippedName} [debug]"

    # Run the test in debug mode using `cargo run` and timeout after 1 minute.
    # The output of the `cargo run` command is piped into `test-parser` to check for test success or failure.
    timeout 1m cargo run --bin "$strippedName" --features test-parser | test-parser -s TEST-SUCCESS -f TEST-FAIL

    if [ $? -eq 1 ]; then
        # If the test failed, log it, mark the failure, and increment failure counter
        echo "The last command failed with exit code 1."
        failedArrDebug+=("$strippedName")
        failed=true
        ((failedCounter++))
    fi

    ((testCounter++))
    echo "Executing test ${testCounter}: ${strippedName} [release]"

    # Run the test in release mode using `cargo run` with the --release flag
    timeout 1m cargo run --bin "$strippedName" --features test-parser --release | test-parser -s TEST-SUCCESS -f TEST-FAIL

    if [ $? -eq 1 ]; then
        echo "The last command failed with exit code 1."
        failedArrRelease+=("$strippedName")
        failed=true
        ((failedCounter++))
    fi
done

echo -e "\n========END OF TESTS SUMMARY========"
((successCounter = testCounter - failedCounter))
echo -e "${successCounter} / ${testCounter} tests passed\n"

# If there were failed tests, display them and exit with a non-zero code to indicate failure
if [ "$failed" = true ]; then
    echo "Some tests failed: "
    for failure in "${failedArrDebug[@]}"; do
        echo "${failure} [debug]"
    done
    for failure in "${failedArrRelease[@]}"; do
        echo "${failure} [release]"
    done
    exit 1
else
    echo "All tests passed!"
    exit 0
fi
