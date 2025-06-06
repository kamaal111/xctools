use anyhow::Result;
use xcbuild_common::{BuildTarget, Configuration, XcodebuildCommandAction, run_xcodebuild_command};

/// Runs tests for an Xcode project or workspace using the `xcodebuild` command-line tool.
///
/// This function constructs and executes an `xcodebuild test` command with the specified parameters
/// to run unit tests, UI tests, or integration tests for iOS, macOS, watchOS, or tvOS applications.
/// It supports testing from either Xcode project files (.xcodeproj) or workspace files (.xcworkspace).
///
/// # Arguments
///
/// * `schema` - The Xcode scheme name to test (e.g., "MyApp", "MyAppTests", "MyAppUITests")
/// * `destination` - The test destination specifying the target device or simulator:
///   - iOS Simulator: "iOS Simulator,name=iPhone 15 Pro"
///   - Generic iOS: "generic/platform=iOS"
///   - macOS: "platform=macOS"
/// * `configuration` - The build configuration to use (Debug or Release)
/// * `project` - Optional path to the Xcode project file (.xcodeproj). Either this or
///   `workspace` must be provided, but not both.
/// * `workspace` - Optional path to the Xcode workspace file (.xcworkspace). Either this or
///   `project` must be provided, but not both.
///
/// # Returns
///
/// Returns `Ok(String)` containing the stdout from the xcodebuild test command on success,
/// or `Err` if the tests fail or if neither project nor workspace is specified.
///
/// # Examples
///
/// ## Testing parameter validation - neither project nor workspace:
/// ```rust
/// use xctools_test::test;
/// use xcbuild_common::Configuration;
///
/// // This should fail because neither project nor workspace is specified
/// let result = test(
///     &"MyAppTests".to_string(),
///     &"iOS Simulator,name=iPhone 15 Pro".to_string(),
///     &Configuration::Debug,
///     &None,
///     &None,
/// );
/// assert!(result.is_err());
/// let error_msg = result.unwrap_err().to_string();
/// assert!(error_msg.contains("Neither project nor workspace is specified"));
/// ```
///
/// ## Testing Configuration enum usage:
/// ```rust
/// use xcbuild_common::Configuration;
///
/// // Test Configuration enum values
/// assert_eq!(Configuration::Debug.command_string(), "Debug");
/// assert_eq!(Configuration::Release.command_string(), "Release");
/// assert_eq!(Configuration::Debug.to_string(), "debug");
/// assert_eq!(Configuration::Release.to_string(), "release");
/// ```
///
/// ## Testing with project parameter (will attempt to run tests):
/// ```rust,no_run
/// use xctools_test::test;
/// use xcbuild_common::Configuration;
///
/// // This example shows the function signature but doesn't run
/// // because it would try to execute xcodebuild test with a non-existent project
/// let result = test(
///     &"MyAppTests".to_string(),
///     &"iOS Simulator,name=iPhone 15 Pro".to_string(),
///     &Configuration::Debug,
///     &Some("MyApp.xcodeproj".to_string()),
///     &None,
/// );
/// // In a real scenario with a valid project, this would either succeed or
/// // fail based on the actual test results
/// ```
///
/// ## Testing with workspace parameter (will attempt to run tests):
/// ```rust,no_run
/// use xctools_test::test;
/// use xcbuild_common::Configuration;
///
/// // This example shows the function signature but doesn't run
/// // because it would try to execute xcodebuild test with a non-existent workspace
/// let result = test(
///     &"MyAppUITests".to_string(),
///     &"generic/platform=iOS".to_string(),
///     &Configuration::Release,
///     &None,
///     &Some("MyApp.xcworkspace".to_string()),
/// );
/// // In a real scenario with a valid workspace, this would either succeed or
/// // fail based on the actual test results
/// ```
///
/// ## Using the xctools CLI:
/// ```bash
/// # Run unit tests with project file and Debug configuration (default)
/// xctools test --schema MyAppTests --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj
///
/// # Run UI tests with workspace file and Release configuration
/// xctools test --schema MyAppUITests --destination "generic/platform=iOS" --workspace MyApp.xcworkspace --configuration release
///
/// # Run tests for macOS
/// xctools test --schema MyAppTests --destination "platform=macOS" --project MyApp.xcodeproj
///
/// # Run all test schemes
/// xctools test --schema MyApp --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj
/// ```
///
/// # Generated Command
///
/// The function generates an xcodebuild command in the format:
/// ```bash
/// xcodebuild test -project MyApp.xcodeproj -scheme MyAppTests -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Debug
/// ```
///
/// # Test Types
///
/// This function can run various types of tests:
/// - **Unit Tests**: Fast, isolated tests that test individual components
/// - **Integration Tests**: Tests that verify interactions between components
/// - **UI Tests**: Automated tests that interact with the user interface
/// - **Performance Tests**: Tests that measure and validate performance metrics
///
/// # Requirements
///
/// - Xcode must be installed and `xcodebuild` must be available in PATH
/// - The specified project/workspace file must exist
/// - The specified scheme must exist in the project/workspace and contain test targets
/// - The destination must be valid for the target platform
/// - Test targets must be properly configured in the Xcode project
/// - For simulator testing, the specified simulator must be available
pub fn test(
    schema: &String,
    destination: &String,
    configuration: &Configuration,
    project: &Option<String>,
    workspace: &Option<String>,
) -> Result<String> {
    let target = BuildTarget::new(project.as_ref(), workspace.as_ref());
    let output = run_xcodebuild_command(
        &XcodebuildCommandAction::Test,
        schema,
        destination,
        configuration,
        &target,
    )?;

    Ok(output)
}

#[cfg(test)]
mod tests {}
