use xcbuild_common::{
    BuildTarget, Configuration, XcodebuildCommandAction, XcodebuildParams, run_xcodebuild_command,
};

/// Builds an Xcode project or workspace using the `xcodebuild` command-line tool.
///
/// This function constructs and executes an `xcodebuild` command with the specified parameters
/// to build an iOS, macOS, watchOS, or tvOS application. It supports building from either
/// Xcode project files (.xcodeproj) or workspace files (.xcworkspace).
///
/// # Arguments
///
/// * `scheme` - The Xcode scheme name to build (e.g., "MyApp", "MyAppTests")
/// * `destination` - The build destination specifying the target device or simulator:
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
/// Returns `Ok(String)` containing the stdout from the xcodebuild command on success,
/// or `Err` if the build fails or if neither project nor workspace is specified.
///
/// # Examples
///
/// ## Testing parameter validation - neither project nor workspace:
/// ```rust
/// use xctools_build::build;
/// use xcbuild_common::Configuration;
///
/// // This should fail because neither project nor workspace is specified
/// let result = build(
///     &"MyApp".to_string(),
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
/// ## Testing with project parameter (will attempt to build):
/// ```rust,no_run
/// use xctools_build::build;
/// use xcbuild_common::Configuration;
///
/// // This example shows the function signature but doesn't run
/// // because it would try to execute xcodebuild with a non-existent project
/// let result = build(
///     &"MyApp".to_string(),
///     &"iOS Simulator,name=iPhone 15 Pro".to_string(),
///     &Configuration::Debug,
///     &Some("MyApp.xcodeproj".to_string()),
///     &None,
/// );
/// // In a real scenario with a valid project, this would either succeed or
/// // fail based on the actual build outcome
/// ```
///
/// ## Testing with workspace parameter (will attempt to build):
/// ```rust,no_run
/// use xctools_build::build;
/// use xcbuild_common::Configuration;
///
/// // This example shows the function signature but doesn't run
/// // because it would try to execute xcodebuild with a non-existent workspace
/// let result = build(
///     &"MyApp".to_string(),
///     &"generic/platform=iOS".to_string(),
///     &Configuration::Release,
///     &None,
///     &Some("MyApp.xcworkspace".to_string()),
/// );
/// // In a real scenario with a valid workspace, this would either succeed or
/// // fail based on the actual build outcome
/// ```
///
/// ## Using the xctools CLI:
/// ```bash
/// # Build with project file and Debug configuration (default)
/// xctools build --scheme MyApp --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj
///
/// # Build with workspace file and Release configuration
/// xctools build --scheme MyApp --destination "generic/platform=iOS" --workspace MyApp.xcworkspace --configuration release
///
/// # Build for macOS
/// xctools build --scheme MyApp --destination "platform=macOS" --project MyApp.xcodeproj
/// ```
///
/// # Generated Command
///
/// The function generates an xcodebuild command in the format:
/// ```bash
/// xcodebuild build -project MyApp.xcodeproj -scheme MyApp -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Debug
/// ```
///
/// # Requirements
///
/// - Xcode must be installed and `xcodebuild` must be available in PATH
/// - The specified project/workspace file must exist
/// - The specified scheme must exist in the project/workspace
/// - The destination must be valid for the target platform
pub fn build(
    scheme: &String,
    destination: &String,
    configuration: &Configuration,
    project: &Option<String>,
    workspace: &Option<String>,
) -> anyhow::Result<String> {
    let target = BuildTarget::new(project.as_ref(), workspace.as_ref());
    let params = XcodebuildParams::new(XcodebuildCommandAction::Build)
        .with_scheme(scheme.clone())
        .with_destination(destination.clone())
        .with_configuration(configuration.clone())
        .with_target(target);
    let output = run_xcodebuild_command(&params)?;

    Ok(output)
}

#[cfg(test)]
mod tests {}
