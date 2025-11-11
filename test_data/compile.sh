#!/bin/bash

# Compilation script for umlink test Java files
# This script compiles Java source files from test_data/java/ into test_data/class/

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

# Find all .java files in the com.example package 
EXAMPLE_FILES=$(find "$JAVA_DIR/com/example" -name "*.java" 2>/dev/null || true)

if [ -n "$EXAMPLE_FILES" ]; then
    echo "Compiling com.example package files..."
    javac -d "$CLASS_DIR" $EXAMPLE_FILES
    echo "✓ Successfully compiled com.example package"
    echo ""
fi

echo "Compilation complete!"
echo ""
echo "To verify compiled files:"
echo "  ls -R $CLASS_DIR"
