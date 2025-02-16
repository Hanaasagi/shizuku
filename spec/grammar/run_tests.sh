#!/usr/bin/bash

set -e

# Usage:
#   ./run_tests.sh [FILES...]
#
# If no arguments are provided, the script tests all `.szk` files in the `tests/` directory.
# If specific files are provided as arguments, only those files will be tested.
#
# Example:
#   ./run_tests.sh                # Run all tests in `tests/`
#   ./run_tests.sh file1.szk      # Test a single file
#   ./run_tests.sh file1.szk file2.szk  # Test multiple files
#
# Exit codes:
#   0 - All tests passed
#   1 - At least one test failed or parser is missing

PARSER="./build/parser"

if [[ ! -x "$PARSER" ]]; then
    echo "ERROR: $PARSER not found or not executable."
    echo "ERROR: You need to run \`make\` first."
    exit 1
fi

SHIZUKU="${SHIZUKU:-szk}"

FILES=("$@")
if [[ ${#FILES[@]} -eq 0 ]]; then
    mapfile -t FILES < <(ls tests/*.szk 2>/dev/null)

    if [[ ${#FILES[@]} -eq 0 ]]; then
        echo "ERROR: No test files found in 'tests/'"
        exit 1
    fi
fi

echo "Running ${#FILES[@]} tests..."

ANY_FAILURE=0

for FILE in "${FILES[@]}"; do
    if [[ ! -f "$FILE" ]]; then
        echo "WARNING: Skipping '$FILE' (file not found)"
        continue
    fi

    error=$("$PARSER" < "$FILE" 2>&1)
    parserret=$?

    if [[ "$parserret" -ne 0 ]]; then
        echo "[❌] FAIL: $FILE: grammar error (exit code: $parserret)"
        [[ "$parserret" -eq 1 ]] && echo "$error"
        ANY_FAILURE=1
    fi
done

if [[ "$ANY_FAILURE" -eq 0 ]]; then
    echo "[✅] All tests passed."
else
    exit 1
fi
