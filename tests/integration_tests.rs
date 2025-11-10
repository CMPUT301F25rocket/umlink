use std::process::Command;
use std::path::Path;
use std::fs;

/// Helper function to get the path to the umlink binary
fn get_umlink_binary() -> String {
    let exe_suffix = if cfg!(windows) { ".exe" } else { "" };
    format!("./target/debug/umlink{}", exe_suffix)
}

/// Helper function to ensure test output directory exists
fn setup_test_output_dir() -> std::io::Result<()> {
    fs::create_dir_all("test_output")
}

/// Helper function to run umlink and return output
fn run_umlink(args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    Command::new(get_umlink_binary())
        .args(args)
        .output()
}

#[test]
fn test_basic_diagram_generation() {
    setup_test_output_dir().expect("Failed to create test output directory");

    let output = run_umlink(&[
        "test_data/input/test.mmd",
        "-i", "test_data/class",
        "-o", "test_output",
    ]).expect("Failed to execute umlink");

    assert!(
        output.status.success(),
        "umlink exited with non-zero status: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that output file was created
    let output_file = Path::new("test_output/test.mmd");
    assert!(
        output_file.exists(),
        "Output file was not created: {:?}",
        output_file
    );

    // Verify output contains expected classes
    let content = fs::read_to_string(output_file)
        .expect("Failed to read output file");

    assert!(content.contains("class MainActivity"), "Output should contain MainActivity class");
    assert!(content.contains("class Notification"), "Output should contain Notification class");
    assert!(content.contains("class NotificationAdapter"), "Output should contain NotificationAdapter class");
    assert!(content.contains("class NotificationRepository"), "Output should contain NotificationRepository class");
    assert!(content.contains("class QRGenerator"), "Output should contain QRGenerator class");
}

#[test]
fn test_skip_annotation() {
    setup_test_output_dir().expect("Failed to create test output directory");

    let output = run_umlink(&[
        "test_data/input/test_skip.mmd",
        "-i", "test_data/class",
        "-o", "test_output",
        "-s", "com.example.Skip",
    ]).expect("Failed to execute umlink");

    assert!(
        output.status.success(),
        "umlink exited with non-zero status: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_file = Path::new("test_output/test_skip.mmd");
    assert!(output_file.exists(), "Output file was not created");

    let content = fs::read_to_string(output_file)
        .expect("Failed to read output file");

    // Should contain TestClass
    assert!(content.contains("class TestClass"), "Output should contain TestClass");

    // SkippedClass itself is marked with @Skip annotation, so the class definition should be skipped
    // However, the relationship may still reference it
    assert!(!content.contains("class SkippedClass"),
        "Output should not contain SkippedClass definition (entire class is marked with @Skip)");

    // Should contain visible fields/methods from TestClass
    assert!(content.contains("visibleField"), "Output should contain visibleField");
    assert!(content.contains("visibleMethod"), "Output should contain visibleMethod");

    // Should NOT contain hidden fields/methods marked with @Skip
    assert!(!content.contains("hiddenField"), "Output should not contain hiddenField (marked with @Skip)");
    assert!(!content.contains("hiddenMethod"), "Output should not contain hiddenMethod (marked with @Skip)");
}

#[test]
fn test_class_retention_annotation() {
    setup_test_output_dir().expect("Failed to create test output directory");

    let output = run_umlink(&[
        "test_data/input/test_class_retention.mmd",
        "-i", "test_data/class",
        "-o", "test_output",
        "-s", "com.example.SkipClass",
    ]).expect("Failed to execute umlink");

    assert!(
        output.status.success(),
        "umlink exited with non-zero status: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_file = Path::new("test_output/test_class_retention.mmd");
    assert!(output_file.exists(), "Output file was not created");

    let content = fs::read_to_string(output_file)
        .expect("Failed to read output file");

    // Should contain TestClassRetention
    assert!(content.contains("class TestClassRetention"), "Output should contain TestClassRetention");

    // Should contain visible fields/methods
    assert!(content.contains("visibleField"), "Output should contain visibleField");
    assert!(content.contains("visibleMethod"), "Output should contain visibleMethod");

    // Should NOT contain hidden fields/methods marked with @SkipClass (CLASS retention)
    assert!(!content.contains("hiddenFieldWithClassRetention"),
        "Output should not contain hiddenFieldWithClassRetention (marked with @SkipClass)");
    assert!(!content.contains("hiddenMethodWithClassRetention"),
        "Output should not contain hiddenMethodWithClassRetention (marked with @SkipClass)");
}

#[test]
fn test_cardinality_preservation() {
    setup_test_output_dir().expect("Failed to create test output directory");

    let output = run_umlink(&[
        "test_data/input/test_cardinality.mmd",
        "-i", "test_data/class",
        "-o", "test_output",
    ]).expect("Failed to execute umlink");

    assert!(
        output.status.success(),
        "umlink exited with non-zero status: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output_file = Path::new("test_output/test_cardinality.mmd");
    assert!(output_file.exists(), "Output file was not created");

    // Just verify it runs successfully and produces output
    // Cardinality-specific assertions would require parsing the mermaid output
}

#[test]
fn test_yaml_frontmatter_preservation() {
    setup_test_output_dir().expect("Failed to create test output directory");

    let output = run_umlink(&[
        "test_data/input/test.mmd",
        "-i", "test_data/class",
        "-o", "test_output",
    ]).expect("Failed to execute umlink");

    assert!(output.status.success(), "umlink should run successfully");

    let output_file = Path::new("test_output/test.mmd");
    let content = fs::read_to_string(output_file)
        .expect("Failed to read output file");

    // Should preserve YAML frontmatter
    assert!(content.starts_with("---"), "Output should start with YAML frontmatter");
    assert!(content.contains("title:"), "Output should contain YAML title field");
    assert!(content.contains("classDiagram"), "Output should contain classDiagram directive");
}
