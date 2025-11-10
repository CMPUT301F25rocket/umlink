# Test Data for umlink

This directory contains all test data for the umlink project.

**Note:** Integration tests that run automatically with `cargo test` are located in the
`tests/` directory at the project root (following Rust conventions). This `test_data/`
directory contains the Java classes and Mermaid files used by those integration tests.

## Directory Structure

```
test_data/
├── java/           # Java source files for test classes
├── class/          # Compiled .class files for testing
├── input/          # Sample mermaid diagram files
├── compile.sh      # Script to compile Java source files
└── README.md       # This file
```

## Directory Contents

### `java/` - Java Source Files

Contains Java source code organized by package:

- **`com/example/`** - Simple test classes that can be compiled standalone
  - `Skip.java` - Annotation with RUNTIME retention for testing skip functionality
  - `SkipClass.java` - Annotation with CLASS retention for testing retention policies
  - `SkippedClass.java` - Class marked with @Skip annotation
  - `TestClass.java` - Test class with some members marked @Skip
  - `TestClassRetention.java` - Test class with members marked @SkipClass

- **`com/rocket/radar/`** - Android project classes (stubs for documentation)
  - `MainActivity.java` - Main Android activity (stub)
  - `notifications/Notification.java` - Notification data model
  - `notifications/NotificationAdapter.java` - RecyclerView adapter (stub with inner classes)
  - `notifications/NotificationRepository.java` - Firebase repository (stub)
  - `qr/QRGenerator.java` - QR code generator (stub)

**Note:** The Android-related classes in `com.rocket.radar.*` are simplified stubs that document
the original class structure but cannot be compiled without the Android SDK. The actual `.class`
files used for testing are from the original Android project and are kept in the `class/` directory.

### `class/` - Compiled Class Files

Contains all `.class` files used for integration testing:

- Simple test classes (compiled from `com.example.*`)
- Android classes (from original Android project - cannot be recompiled without Android SDK)

These `.class` files are the actual test data used by umlink to generate UML diagrams.

### `input/` - Sample Mermaid Diagrams

Contains sample `.mmd` (Mermaid) diagram files for testing:

- `test.mmd` - Main test diagram with Android classes
- `test_skip.mmd` - Tests the @Skip annotation functionality
- `test_cardinality.mmd` - Tests cardinality/multiplicity in relationships
- `test_class_retention.mmd` - Tests CLASS retention policy annotations

## Compilation

To recompile the `com.example.*` classes from source:

```bash
cd test_data
./compile.sh
```

This will compile all Java files in `java/com/example/` into `class/`.

**Note:** The Android classes cannot be recompiled without the Android SDK and its dependencies
(androidx, Firebase, etc.). The existing `.class` files are used as-is for testing.

## Running Tests

Integration tests run automatically with:

```bash
cargo test
```

To manually run umlink with the test data:

```bash
# From the project root
cargo build

# Generate a linked diagram
./target/debug/umlink test_data/input/test.mmd -i test_data/class -o output

# Test skip annotation (com.example.Skip)
./target/debug/umlink test_data/input/test_skip.mmd -i test_data/class -o output -s com.example.Skip

# Test class retention annotation (com.example.SkipClass)
./target/debug/umlink test_data/input/test_class_retention.mmd -i test_data/class -o output -s com.example.SkipClass
```

## Test Philosophy

This test structure follows the principle that **compiled `.class` files should always be
accompanied by their Java source code**. This makes the test data transparent and maintainable:

- ✓ Anyone can see what the class structure looks like without decompiling
- ✓ Simple classes can be recompiled if needed
- ✓ Documentation of the original source is preserved
- ✓ No opaque binary files in the repository
