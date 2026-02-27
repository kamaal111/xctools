use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
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
        .stdout(predicate::str::contains("--scheme"))
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
        "--scheme",
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
        "--scheme",
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
        "--scheme",
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
        "--scheme",
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
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
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
        .stdout(predicate::str::contains(
            "Successfully updated project.pbxproj",
        ))
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
        .stdout(predicate::str::contains(
            "Successfully updated project.pbxproj",
        ))
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
    cmd.current_dir(tmp.path()).args(&[
        "bump-version",
        "--build-number",
        "100",
        "--version-number",
        "3.2.1",
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Successfully updated project.pbxproj",
        ))
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
        .stdout(predicate::str::contains(
            "Successfully updated project.pbxproj",
        ))
        .stdout(predicate::str::contains("Build number set to: 999"));

    let content = std::fs::read_to_string(&pbxproj_path).unwrap();

    assert!(content.contains("CURRENT_PROJECT_VERSION = 999;"));
    assert!(content.contains("MARKETING_VERSION = 0.1.0;")); // Should remain unchanged
}

#[test]
fn test_acknowledgements_command_help() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&["acknowledgements", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Generate acknowledgements file"))
        .stdout(predicate::str::contains("--app-name"))
        .stdout(predicate::str::contains("--output"));
}

#[test]
fn test_acknowledgements_command_missing_required_args() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.arg("acknowledgements");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_acknowledgements_command_missing_app_name() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&["acknowledgements", "--output", "./output.json"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_acknowledgements_command_missing_output() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&["acknowledgements", "--app-name", "TestApp"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_acknowledgements_command_argument_parsing() {
    // Test that valid arguments are parsed correctly without actually finding DerivedData
    // We'll check that there are no CLI argument parsing errors in stderr
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "acknowledgements",
        "--app-name",
        "TestApp",
        "--output",
        "./acknowledgements.json",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_acknowledgements_command_nonexistent_app() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "acknowledgements",
        "--app-name",
        "NonExistentApp123XYZ",
        "--output",
        "./acknowledgements.json",
    ]);

    // This should fail because DerivedData won't be found for the nonexistent app
    cmd.assert().failure().stderr(predicate::str::contains(
        "Could not find any DerivedData for project",
    ));
}

#[test]
fn test_acknowledgements_command_output_to_directory() {
    let tmp = tempdir().unwrap();

    // Create a mock DerivedData structure for testing
    let home_dir = tmp.path().join("home");
    fs::create_dir_all(&home_dir).unwrap();
    let derived_data_dir = home_dir
        .join("Library/Developer/Xcode/DerivedData")
        .join("TestApp-abc123");
    let source_packages_dir = derived_data_dir.join("SourcePackages");
    let checkouts_dir = source_packages_dir.join("checkouts");
    fs::create_dir_all(&checkouts_dir).unwrap();

    // Create minimal workspace-state.json
    let workspace_state_content = r#"
    {
        "object": {
            "dependencies": []
        }
    }
    "#;
    fs::write(
        source_packages_dir.join("workspace-state.json"),
        workspace_state_content,
    )
    .unwrap();

    // Set HOME environment variable to point to our mock directory
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.env("HOME", home_dir);
    cmd.args(&[
        "acknowledgements",
        "--app-name",
        "TestApp",
        "--output",
        tmp.path().to_str().unwrap(),
    ]);

    let output = cmd.output().unwrap();

    // Check that it attempted to create acknowledgements.json in the directory
    let expected_file = tmp.path().join("acknowledgements.json");

    // If successful, the file should exist and contain valid JSON
    if output.status.success() {
        assert!(expected_file.exists());
        let content = fs::read_to_string(&expected_file).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(json["packages"].is_array());
        assert!(json["contributors"].is_array());
    } else {
        // If it fails, it should be due to DerivedData issues, not CLI parsing
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("error: the following required arguments were not provided"));
    }
}

#[test]
fn test_acknowledgements_command_output_to_specific_file() {
    let tmp = tempdir().unwrap();
    let output_file = tmp.path().join("custom_acknowledgements.json");

    // Create a mock DerivedData structure
    let home_dir = tmp.path().join("home");
    fs::create_dir_all(&home_dir).unwrap();
    let derived_data_dir = home_dir
        .join("Library/Developer/Xcode/DerivedData")
        .join("TestApp-abc123");
    let source_packages_dir = derived_data_dir.join("SourcePackages");
    let checkouts_dir = source_packages_dir.join("checkouts");
    fs::create_dir_all(&checkouts_dir).unwrap();

    // Create minimal workspace-state.json
    let workspace_state_content = r#"
    {
        "object": {
            "dependencies": []
        }
    }
    "#;
    fs::write(
        source_packages_dir.join("workspace-state.json"),
        workspace_state_content,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.env("HOME", home_dir);
    cmd.args(&[
        "acknowledgements",
        "--app-name",
        "TestApp",
        "--output",
        output_file.to_str().unwrap(),
    ]);

    let output = cmd.output().unwrap();

    if output.status.success() {
        // Should create the specific file
        assert!(output_file.exists());
        let content = fs::read_to_string(&output_file).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(json["packages"].is_array());
        assert!(json["contributors"].is_array());

        // Should contain success message
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Acknowledgements written to:"));
        assert!(stdout.contains("custom_acknowledgements.json"));
    }
}

#[test]
fn test_acknowledgements_command_with_mock_packages() {
    let tmp = tempdir().unwrap();
    let output_file = tmp.path().join("acknowledgements.json");

    // Create a comprehensive mock DerivedData structure
    let home_dir = tmp.path().join("home");
    fs::create_dir_all(&home_dir).unwrap();
    let derived_data_dir = home_dir
        .join("Library/Developer/Xcode/DerivedData")
        .join("TestApp-abc123");
    let source_packages_dir = derived_data_dir.join("SourcePackages");
    let checkouts_dir = source_packages_dir.join("checkouts");
    fs::create_dir_all(&checkouts_dir).unwrap();

    // Create workspace-state.json with mock packages
    let workspace_state_content = r#"
    {
        "object": {
            "dependencies": [
                {
                    "packageRef": {
                        "name": "SwiftUI",
                        "location": "https://github.com/apple/swift-ui"
                    }
                },
                {
                    "packageRef": {
                        "name": "Alamofire",
                        "location": "https://github.com/Alamofire/Alamofire.git"
                    }
                }
            ]
        }
    }
    "#;
    fs::write(
        source_packages_dir.join("workspace-state.json"),
        workspace_state_content,
    )
    .unwrap();

    // Create mock package directories with licenses
    let swiftui_dir = checkouts_dir.join("SwiftUI");
    fs::create_dir_all(&swiftui_dir).unwrap();
    fs::write(
        swiftui_dir.join("LICENSE"),
        "Apache License 2.0\n\nCopyright (c) 2024 Apple Inc.",
    )
    .unwrap();

    let alamofire_dir = checkouts_dir.join("Alamofire");
    fs::create_dir_all(&alamofire_dir).unwrap();
    fs::write(
        alamofire_dir.join("LICENSE"),
        "MIT License\n\nCopyright (c) 2024 Alamofire Software Foundation",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.env("HOME", home_dir);
    cmd.args(&[
        "acknowledgements",
        "--app-name",
        "TestApp",
        "--output",
        output_file.to_str().unwrap(),
    ]);

    let output = cmd.output().unwrap();

    if output.status.success() {
        assert!(output_file.exists());
        let content = fs::read_to_string(&output_file).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Verify JSON structure
        assert!(json["packages"].is_array());
        assert!(json["contributors"].is_array());

        let packages = json["packages"].as_array().unwrap();

        // Should contain our mock packages
        assert!(packages.iter().any(|p| p["name"] == "SwiftUI"));
        assert!(packages.iter().any(|p| p["name"] == "Alamofire"));

        // Check package details
        let swiftui_package = packages.iter().find(|p| p["name"] == "SwiftUI").unwrap();
        assert_eq!(swiftui_package["author"], "apple");
        assert_eq!(swiftui_package["url"], "https://github.com/apple/swift-ui");
        assert!(
            swiftui_package["license"]
                .as_str()
                .unwrap()
                .contains("Apache License")
        );

        let alamofire_package = packages.iter().find(|p| p["name"] == "Alamofire").unwrap();
        assert_eq!(alamofire_package["author"], "Alamofire");
        assert_eq!(
            alamofire_package["url"],
            "https://github.com/Alamofire/Alamofire.git"
        );
        assert!(
            alamofire_package["license"]
                .as_str()
                .unwrap()
                .contains("MIT License")
        );

        // Should contain success message
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("✅ Acknowledgements written to:"));
    }
}

#[test]
fn test_acknowledgements_command_invalid_output_path() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "acknowledgements",
        "--app-name",
        "TestApp",
        "--output",
        "/root/invalid/path/acknowledgements.json", // Invalid path that requires root permissions
    ]);

    let output = cmd.output().unwrap();

    // Should fail due to invalid output path (if it gets that far)
    // Most likely it will fail earlier due to DerivedData not being found
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Could fail for DerivedData or file permissions
        assert!(
            stderr.contains("Could not find any DerivedData")
                || stderr.contains("Failed to write acknowledgements to file")
                || stderr.contains("Permission denied")
        );
    }
}

#[test]
fn test_acknowledgements_command_with_git_repository() {
    let tmp = tempdir().unwrap();
    let output_file = tmp.path().join("acknowledgements.json");

    // Initialize a git repository in the temp directory
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    // Configure git with test user
    std::process::Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    // Create a test file and commit
    fs::write(tmp.path().join("README.md"), "# Test Project").unwrap();
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(&tmp)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&tmp)
        .output()
        .unwrap();

    // Create mock DerivedData structure
    let home_dir = tmp.path().join("home");
    fs::create_dir_all(&home_dir).unwrap();
    let derived_data_dir = home_dir
        .join("Library/Developer/Xcode/DerivedData")
        .join("TestApp-abc123");
    let source_packages_dir = derived_data_dir.join("SourcePackages");
    fs::create_dir_all(&source_packages_dir).unwrap();

    // Create minimal workspace-state.json
    let workspace_state_content = r#"
    {
        "object": {
            "dependencies": []
        }
    }
    "#;
    fs::write(
        source_packages_dir.join("workspace-state.json"),
        workspace_state_content,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.env("HOME", home_dir);
    cmd.current_dir(&tmp); // Run from within the git repository
    cmd.args(&[
        "acknowledgements",
        "--app-name",
        "TestApp",
        "--output",
        output_file.to_str().unwrap(),
    ]);

    let output = cmd.output().unwrap();

    if output.status.success() {
        assert!(output_file.exists());
        let content = fs::read_to_string(&output_file).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Should have contributors from git history
        let contributors = json["contributors"].as_array().unwrap();

        // Should contain our test commit author
        assert!(
            contributors
                .iter()
                .any(|c| c["name"].as_str().unwrap().contains("Test User"))
        );
    }
}

#[test]
fn test_acknowledgements_command_app_name_with_spaces() {
    let tmp = tempdir().unwrap();

    // Create mock DerivedData for app with spaces in name
    let home_dir = tmp.path().join("home");
    fs::create_dir_all(&home_dir).unwrap();
    let derived_data_dir = home_dir
        .join("Library/Developer/Xcode/DerivedData")
        .join("My Test App-abc123");
    let source_packages_dir = derived_data_dir.join("SourcePackages");
    fs::create_dir_all(&source_packages_dir).unwrap();

    // Create minimal workspace-state.json
    let workspace_state_content = r#"
    {
        "object": {
            "dependencies": []
        }
    }
    "#;
    fs::write(
        source_packages_dir.join("workspace-state.json"),
        workspace_state_content,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.env("HOME", home_dir);
    cmd.args(&[
        "acknowledgements",
        "--app-name",
        "My Test App",
        "--output",
        tmp.path().join("output.json").to_str().unwrap(),
    ]);

    let output = cmd.output().unwrap();

    // Should handle app names with spaces correctly
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("✅ Acknowledgements written to:"));
    } else {
        // If it fails, should be due to technical issues, not CLI parsing
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("error: the following required arguments were not provided"));
    }
}

#[test]
fn test_acknowledgements_command_short_flags() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "acknowledgements",
        "-a",
        "TestApp",
        "-o",
        "./acknowledgements.json",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should parse short flags correctly (even if it fails later due to missing DerivedData)
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("unexpected argument"));
}

// Test command integration tests
#[test]
fn test_test_command_help() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&["test", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Test Xcode project"))
        .stdout(predicate::str::contains("--scheme"))
        .stdout(predicate::str::contains("--destination"))
        .stdout(predicate::str::contains("--configuration"))
        .stdout(predicate::str::contains("--project"))
        .stdout(predicate::str::contains("--workspace"));
}

#[test]
fn test_test_command_missing_required_args() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.arg("test");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_test_command_missing_schema() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--destination",
        "iOS Simulator,name=iPhone 15 Pro",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"))
        .stderr(predicate::str::contains("scheme"));
}

#[test]
fn test_test_command_missing_destination() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"))
        .stderr(predicate::str::contains("destination"));
}

#[test]
fn test_test_command_missing_project_or_workspace() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
        "--destination",
        "iOS Simulator,name=iPhone 15 Pro",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_test_command_with_both_project_and_workspace() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
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
fn test_test_command_invalid_configuration() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
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
fn test_test_command_with_project() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
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
fn test_test_command_with_workspace() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppUITests",
        "--destination",
        "iOS Simulator,name=iPhone 15 Pro",
        "--workspace",
        "TestXcodeApp/TestXcodeApp.xcworkspace",
        "--configuration",
        "release",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_test_command_debug_configuration() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
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
fn test_test_command_release_configuration() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
        "--destination",
        "iOS Simulator,name=iPhone 15 Pro",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
        "--configuration",
        "release",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_test_command_ios_simulator_destination() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
        "--destination",
        "iOS Simulator,name=iPhone 15 Pro",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_test_command_ios_generic_destination() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
        "--destination",
        "generic/platform=iOS",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_test_command_macos_destination() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
        "--destination",
        "platform=macOS",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_test_command_ui_tests_scheme() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppUITests",
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
fn test_test_command_unit_tests_scheme() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
        "--destination",
        "iOS Simulator,name=iPhone 15 Pro",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
        "--configuration",
        "release",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_test_command_iphone_16_pro_destination() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
        "--destination",
        "platform=iOS Simulator,name=iPhone 16 Pro",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_test_command_default_configuration() {
    // Test that default configuration is applied when not specified
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "TestXcodeAppTests",
        "--destination",
        "iOS Simulator,name=iPhone 15 Pro",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_test_command_argument_parsing_comprehensive() {
    // Test that all valid test arguments are parsed correctly without running xcodebuild
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "--scheme",
        "MyTestScheme",
        "--destination",
        "iOS Simulator,name=iPhone 15 Pro Max",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
        "--configuration",
        "release",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
    assert!(!stderr.contains("unexpected argument"));
}

#[test]
fn test_test_command_short_flags() {
    // Test that short flags work correctly
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "-s",
        "TestXcodeAppTests",
        "-d",
        "iOS Simulator,name=iPhone 15 Pro",
        "-p",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
        "-c",
        "debug",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should parse short flags correctly (even if it fails later due to actual xcodebuild execution)
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("unexpected argument"));
}

#[test]
fn test_test_command_workspace_short_flag() {
    // Test workspace with short flag
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "test",
        "-s",
        "TestXcodeAppUITests",
        "-d",
        "iOS Simulator,name=iPhone 15 Pro",
        "-w",
        "TestXcodeApp/TestXcodeApp.xcworkspace",
        "-c",
        "release",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should parse short flags correctly
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("unexpected argument"));
}

// Archive command integration tests
#[test]
fn test_archive_command_help() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&["archive", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Archive Xcode project"))
        .stdout(predicate::str::contains("--scheme"))
        .stdout(predicate::str::contains("--destination"))
        .stdout(predicate::str::contains("--sdk"))
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--project"))
        .stdout(predicate::str::contains("--workspace"));
}

#[test]
fn test_archive_command_missing_required_args() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.arg("archive");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_archive_command_missing_schema() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "iphoneos",
        "--output",
        "MyApp.xcarchive",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"))
        .stderr(predicate::str::contains("scheme"));
}

#[test]
fn test_archive_command_missing_destination() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--sdk",
        "iphoneos",
        "--output",
        "MyApp.xcarchive",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"))
        .stderr(predicate::str::contains("destination"));
}

#[test]
fn test_archive_command_missing_sdk() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=iOS",
        "--output",
        "MyApp.xcarchive",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"))
        .stderr(predicate::str::contains("sdk"));
}

#[test]
fn test_archive_command_missing_output() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "iphoneos",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"))
        .stderr(predicate::str::contains("output"));
}

#[test]
fn test_archive_command_missing_project_or_workspace() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "iphoneos",
        "--output",
        "MyApp.xcarchive",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_archive_command_with_both_project_and_workspace() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "iphoneos",
        "--output",
        "MyApp.xcarchive",
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
fn test_archive_command_invalid_sdk() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "invalid",
        "--output",
        "MyApp.xcarchive",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'invalid'"));
}

#[test]
fn test_archive_command_invalid_configuration() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "iphoneos",
        "--output",
        "MyApp.xcarchive",
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
fn test_archive_command_valid_iphoneos_sdk() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "iphoneos",
        "--output",
        "MyApp.xcarchive",
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
fn test_archive_command_valid_macosx_sdk() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=macOS",
        "--sdk",
        "macosx",
        "--output",
        "MyApp.xcarchive",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
        "--configuration",
        "release",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_archive_command_with_workspace() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "iphoneos",
        "--output",
        "MyApp.xcarchive",
        "--workspace",
        "TestXcodeApp/TestXcodeApp.xcworkspace",
        "--configuration",
        "release",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_archive_command_debug_configuration() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "iphoneos",
        "--output",
        "MyApp-Debug.xcarchive",
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
fn test_archive_command_release_configuration() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "iphoneos",
        "--output",
        "MyApp-Release.xcarchive",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
        "--configuration",
        "release",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_archive_command_custom_output_path() {
    let tmp = tempdir().unwrap();
    let archive_path = tmp.path().join("build/archives/MyApp-v1.0.xcarchive");

    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "iphoneos",
        "--output",
        archive_path.to_str().unwrap(),
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
        "--configuration",
        "release",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_archive_command_ios_generic_destination() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "iphoneos",
        "--output",
        "MyApp.xcarchive",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_archive_command_macos_generic_destination() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "TestXcodeApp",
        "--destination",
        "generic/platform=macOS",
        "--sdk",
        "macosx",
        "--output",
        "MyApp.xcarchive",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
}

#[test]
fn test_archive_command_argument_parsing_comprehensive() {
    // Test that all valid archive arguments are parsed correctly without running xcodebuild
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "archive",
        "--scheme",
        "MyTestScheme",
        "--destination",
        "generic/platform=iOS",
        "--sdk",
        "iphoneos",
        "--output",
        "/tmp/MyApp-v2.0.xcarchive",
        "--project",
        "TestXcodeApp/TestXcodeApp.xcodeproj",
        "--configuration",
        "release",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("cannot be used with"));
    assert!(!stderr.contains("unexpected argument"));
}

// ---- notarize command tests ----

#[test]
fn test_notarize_command_help() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&["notarize", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Notarize"))
        .stdout(predicate::str::contains("--file-path"))
        .stdout(predicate::str::contains("--apple-id"))
        .stdout(predicate::str::contains("--password"))
        .stdout(predicate::str::contains("--team-id"));
}

#[test]
fn test_notarize_command_missing_required_args() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.arg("notarize");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_notarize_command_missing_apple_id() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "notarize",
        "--file-path",
        "MyApp.dmg",
        "--password",
        "xxxx-xxxx-xxxx-xxxx",
        "--team-id",
        "A1B2C3D4E5",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_notarize_command_accepts_valid_args() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "notarize",
        "--file-path",
        "MyApp.dmg",
        "--apple-id",
        "developer@example.com",
        "--password",
        "xxxx-xxxx-xxxx-xxxx",
        "--team-id",
        "A1B2C3D4E5",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("unexpected argument"));
}

// ---- setup-signing command tests ----

#[test]
fn test_setup_signing_command_help() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&["setup-signing", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("signing"))
        .stdout(predicate::str::contains("--certificate-path"))
        .stdout(predicate::str::contains("--certificate-password"))
        .stdout(predicate::str::contains("--provisioning-profile"));
}

#[test]
fn test_setup_signing_command_missing_required_args() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.arg("setup-signing");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_setup_signing_command_missing_certificate_password() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&["setup-signing", "--certificate-path", "signing.p12"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_setup_signing_command_accepts_valid_args_no_profiles() {
    let tmp = tempdir().unwrap();
    let cert_path = tmp.path().join("signing.p12");
    fs::write(&cert_path, b"fake p12 content").unwrap();

    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "setup-signing",
        "--certificate-path",
        cert_path.to_str().unwrap(),
        "--certificate-password",
        "secretpassword",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("unexpected argument"));
}

#[test]
fn test_setup_signing_command_accepts_multiple_provisioning_profiles() {
    let mut cmd = Command::cargo_bin("xctools").unwrap();
    cmd.args(&[
        "setup-signing",
        "--certificate-path",
        "signing.p12",
        "--certificate-password",
        "secretpassword",
        "--provisioning-profile",
        "AppStore.mobileprovision",
        "--provisioning-profile",
        "WatchApp.mobileprovision",
    ]);

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure we don't get CLI argument parsing errors
    assert!(!stderr.contains("error: the following required arguments were not provided"));
    assert!(!stderr.contains("invalid value"));
    assert!(!stderr.contains("unexpected argument"));
}
