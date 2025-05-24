use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Build Xcode project"));
}

#[test]
fn test_build_command_help() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&["build", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Build Xcode project"))
        .stdout(predicate::str::contains("--schema"))
        .stdout(predicate::str::contains("--destination"))
        .stdout(predicate::str::contains("--project"))
        .stdout(predicate::str::contains("--workspace"));
}

#[test]
fn test_build_command_missing_required_args() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.arg("build");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_build_command_missing_project_or_workspace() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "build",
        "--schema",
        "TestXcodeApp",
        "--destination",
        "iOS Simulator,name=iPhone 15 Pro",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_build_command_with_both_project_and_workspace() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "build",
        "--schema",
        "TestXcodeApp",
        "--destination",
        "iOS Simulator,name=iPhone 15 Pro",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
        "--workspace",
        "TestXcodeApp/TestXcodeApp.xcworkspace",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn test_build_command_invalid_configuration() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "build",
        "--schema",
        "TestXcodeApp",
        "--destination",
        "iOS Simulator,name=iPhone 15 Pro",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
        "--configuration",
        "invalid",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'invalid'"));
}

#[test]
fn test_build_command_argument_parsing() {
    // Test that valid arguments are parsed correctly without actually running xcodebuild
    // We'll check that there are no argument parsing errors in stderr
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "build",
        "--schema",
        "TestXcodeApp",
        "--destination",
        "iOS Simulator,name=iPhone 15 Pro",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
        "--configuration",
        "debug",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}
