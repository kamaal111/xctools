use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

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
    cmd.assert().failure().stderr(predicate::str::contains("required"));
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
    cmd.assert().failure().stderr(predicate::str::contains("required"));
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

#[test]
fn test_bump_version_command_help() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&["bump-version", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Bump version of Xcode project"))
        .stdout(predicate::str::contains("--build-number"))
        .stdout(predicate::str::contains("--version-number"));
}

#[test]
fn test_bump_version_command_missing_required_args() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.arg("bump-version");
    cmd.assert().failure().stderr(predicate::str::contains("required"));
}

#[test]
fn test_bump_version_command_no_pbxproj() {
    let tmp = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.current_dir(tmp.path())
        .args(&["bump-version", "--build-number", "42"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No project.pbxproj found"));
}

#[test]
fn test_bump_version_command_build_number_only() {
    let tmp = tempdir().unwrap();
    let pbxproj_path = tmp.path().join("project.pbxproj");
    std::fs::write(
        &pbxproj_path,
        "CURRENT_PROJECT_VERSION = 1;\nMARKETING_VERSION = 1.0.0;\nOtherKey = value;",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.current_dir(tmp.path())
        .args(&["bump-version", "--build-number", "42"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully updated project.pbxproj"))
        .stdout(predicate::str::contains("Build number set to: 42"))
        .stdout(predicate::str::contains("Version number set to: UNSET"));

    let content = std::fs::read_to_string(&pbxproj_path).unwrap();

    assert!(content.contains("CURRENT_PROJECT_VERSION = 42;"));
    assert!(content.contains("MARKETING_VERSION = 1.0.0;")); // Should remain unchanged
}

#[test]
fn test_bump_version_command_version_number_only() {
    let tmp = tempdir().unwrap();
    let pbxproj_path = tmp.path().join("project.pbxproj");
    std::fs::write(
        &pbxproj_path,
        "CURRENT_PROJECT_VERSION = 5;\nMARKETING_VERSION = 1.0.0;\nOtherKey = value;",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.current_dir(tmp.path())
        .args(&["bump-version", "--version-number", "2.1.3"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully updated project.pbxproj"))
        .stdout(predicate::str::contains("Build number set to: UNSET"))
        .stdout(predicate::str::contains("Version number set to: 2.1.3"));

    let content = std::fs::read_to_string(&pbxproj_path).unwrap();

    assert!(content.contains("CURRENT_PROJECT_VERSION = 5;")); // Should remain unchanged
    assert!(content.contains("MARKETING_VERSION = 2.1.3;"));
}

#[test]
fn test_bump_version_command_both_values() {
    let tmp = tempdir().unwrap();
    let pbxproj_path = tmp.path().join("project.pbxproj");
    std::fs::write(
        &pbxproj_path,
        "CURRENT_PROJECT_VERSION = 1;\n    MARKETING_VERSION = 1.0.0;\nOtherKey = value;",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.current_dir(tmp.path())
        .args(&["bump-version", "--build-number", "100", "--version-number", "3.2.1"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully updated project.pbxproj"))
        .stdout(predicate::str::contains("Build number set to: 100"))
        .stdout(predicate::str::contains("Version number set to: 3.2.1"));

    let content = std::fs::read_to_string(&pbxproj_path).unwrap();

    assert!(content.contains("CURRENT_PROJECT_VERSION = 100;"));
    assert!(content.contains("MARKETING_VERSION = 3.2.1;"));
}

#[test]
fn test_bump_version_command_invalid_version() {
    let tmp = tempdir().unwrap();
    let pbxproj_path = tmp.path().join("project.pbxproj");
    std::fs::write(&pbxproj_path, "CURRENT_PROJECT_VERSION = 1;").unwrap();

    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.current_dir(tmp.path())
        .args(&["bump-version", "--version-number", "invalid.version"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'invalid.version'"));
}

#[test]
fn test_bump_version_command_with_nested_pbxproj() {
    let tmp = tempdir().unwrap();

    // Create a nested directory structure like a real Xcode project
    let project_dir = tmp.path().join("MyApp.xcodeproj");
    std::fs::create_dir_all(&project_dir).unwrap();

    let pbxproj_path = project_dir.join("project.pbxproj");
    std::fs::write(
        &pbxproj_path,
        "CURRENT_PROJECT_VERSION = 3;\nMARKETING_VERSION = 0.1.0;",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.current_dir(tmp.path())
        .args(&["bump-version", "--build-number", "999"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully updated project.pbxproj"))
        .stdout(predicate::str::contains("Build number set to: 999"));

    let content = std::fs::read_to_string(&pbxproj_path).unwrap();

    assert!(content.contains("CURRENT_PROJECT_VERSION = 999;"));
    assert!(content.contains("MARKETING_VERSION = 0.1.0;")); // Should remain unchanged
}
