#!/bin/bash

# Compilation script for umlink test Java files
# This script compiles Java source files from tests/java/ into tests/class/

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
JAVA_DIR="$SCRIPT_DIR/java"
CLASS_DIR="$SCRIPT_DIR/class"

echo "Compiling Java test files..."
echo "Source directory: $JAVA_DIR"
echo "Output directory: $CLASS_DIR"
echo ""

# Create output directory if it doesn't exist
mkdir -p "$CLASS_DIR"

# Find all .java files in the com.example package (these are simple and don't require Android SDK)
EXAMPLE_FILES=$(find "$JAVA_DIR/com/example" -name "*.java" 2>/dev/null || true)

if [ -n "$EXAMPLE_FILES" ]; then
    echo "Compiling com.example package files..."
    javac -d "$CLASS_DIR" $EXAMPLE_FILES
    echo "✓ Successfully compiled com.example package"
    echo ""
fi

# The Android-related classes (com.rocket.radar.*) cannot be compiled without Android SDK
# They are provided as stubs for documentation purposes only
ROCKET_FILES=$(find "$JAVA_DIR/com/rocket" -name "*.java" 2>/dev/null || true)

if [ -n "$ROCKET_FILES" ]; then
    echo "Note: Android-related classes (com.rocket.radar.*) cannot be compiled without Android SDK."
    echo "These are stub files for documentation purposes. The actual .class files in tests/class/"
    echo "were compiled from the original Android project and are used as-is for testing."
    echo ""
    echo "Android stub files found (not compiled):"
    echo "$ROCKET_FILES" | while read -r file; do
        echo "  - $(basename "$file")"
    done
    echo ""
fi

echo "Compilation complete!"
echo ""
echo "To verify compiled files:"
echo "  ls -R $CLASS_DIR"
