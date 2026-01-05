#!/bin/bash

set -eux

PYTHON="/usr/bin/env python3"
# chdir to containing directory
cd "$(dirname "$0")"

# use a venv if available
if [ -f "./venv/bin/python3" ]; then
    PYTHON="./venv/bin/python3";
fi

# Copy the vdep json over.
$PYTHON ../mkvdepjson.py local | tee Snippets-java/vendordeps/ReduxLib_local.json Snippets-cpp/vendordeps/ReduxLib_local.json

# Compile and publish vendordep
(cd .. && rm -rf build/docs && rm -rf build/doxygen && ./gradlew -PreleaseMode javadoc doxygen publish)

# Check that every file has a copyright header
$PYTHON check_copyright.py ..
# Run the java/cpp/copyright lint checks
$PYTHON javadoc.py "../build/docs/javadoc"
# Compile java
(cd "Snippets-java" && ./gradlew build && ./gradlew clean)

$PYTHON doxygen.py "../build/doxygen/html"
# Compile cpp
(cd "Snippets-cpp" && ./gradlew frcUserProgramReleaseExecutable && ./gradlew clean)



