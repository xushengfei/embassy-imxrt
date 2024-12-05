#!/bin/bash

# Get all .rs files in the current directory
files=(*.rs)

testCounter=1
failed=false
failedArr=()

# Loop through each file and add its name without the .rs extension to the array
for file in "${files[@]}"; do
    strippedName="${file%.rs}"
    echo "Executing test ${testCounter}: ${strippedName}"
    ((testCounter++))

    cargo run --bin "$strippedName" | test-parser -s TEST-SUCCESS -f TEST-FAIL

    if [ $? -eq 1 ]; then
        echo "The last command failed with exit code 1."
        failedArr+=("$strippedName")
        failed=true
    fi
done

echo -e "\n========END OF TESTS SUMMARY========"
if [ "$failed" = true ]; then
    echo "Some tests failed: "
    for failure in "${failedArr[@]}"; do
        echo "${failure}"
    done
    exit 1
else
    echo "All tests passed!"
    exit 0
fi
