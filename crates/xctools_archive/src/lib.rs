use anyhow::Result;
use xcbuild_common::{
    BuildTarget, Configuration, SDK, XcodebuildCommandAction, run_xcodebuild_command,
};

/// Creates an archive for an Xcode project or workspace using the `xcodebuild` command-line tool.
///
/// This function constructs and executes an `xcodebuild archive` command with the specified parameters
/// to create an .xcarchive bundle containing the built application and its debug symbols (dSYMs).
/// Archives are typically used for distribution, App Store submission, or enterprise deployment
/// of iOS, macOS, watchOS, or tvOS applications.
///
/// # Arguments
///
/// * `schema` - The Xcode scheme name to archive (e.g., "MyApp", "MyApp Release")
/// * `destination` - The archive destination specifying the target platform:
///   - Generic iOS: "generic/platform=iOS"
///   - Generic macOS: "generic/platform=macOS"
/// * `configuration` - The build configuration to use (Debug or Release)
/// * `sdk` - The SDK to use for building the archive:
///   - iOS: SDK::Iphoneos
///   - macOS: SDK::Macosx
/// * `output` - The path where the .xcarchive bundle should be created (e.g., "MyApp.xcarchive")
/// * `project` - Optional path to the Xcode project file (.xcodeproj). Either this or
///   `workspace` must be provided, but not both.
/// * `workspace` - Optional path to the Xcode workspace file (.xcworkspace). Either this or
///   `project` must be provided, but not both.
///
/// # Returns
///
/// Returns `Ok(String)` containing the stdout from the xcodebuild archive command on success,
/// or `Err` if the archive process fails or if neither project nor workspace is specified.
///
/// # Examples
///
/// ## Archiving parameter validation - neither project nor workspace:
/// ```rust
/// use xctools_archive::archive;
/// use xcbuild_common::{Configuration, SDK};
///
/// // This should fail because neither project nor workspace is specified
/// let result = archive(
///     &"MyApp".to_string(),
///     &"generic/platform=iOS".to_string(),
///     &Configuration::Release,
///     &SDK::Iphoneos,
///     &"MyApp.xcarchive".to_string(),
///     &None,
///     &None,
/// );
/// assert!(result.is_err());
/// let error_msg = result.unwrap_err().to_string();
/// assert!(error_msg.contains("Neither project nor workspace is specified"));
/// ```
///
/// ## Testing Configuration and SDK enum usage:
/// ```rust
/// use xcbuild_common::{Configuration, SDK};
///
/// // Test Configuration enum values
/// assert_eq!(Configuration::Debug.command_string(), "Debug");
/// assert_eq!(Configuration::Release.command_string(), "Release");
/// assert_eq!(Configuration::Debug.to_string(), "debug");
/// assert_eq!(Configuration::Release.to_string(), "release");
///
/// // Test SDK enum values
/// assert_eq!(SDK::Iphoneos.command_string(), "iphoneos");
/// assert_eq!(SDK::Macosx.command_string(), "macosx");
/// ```
///
/// ## Archiving with project parameter (will attempt to create archive):
/// ```rust,no_run
/// use xctools_archive::archive;
/// use xcbuild_common::{Configuration, SDK};
///
/// // This example shows the function signature but doesn't run
/// // because it would try to execute xcodebuild archive with a non-existent project
/// let result = archive(
///     &"MyApp".to_string(),
///     &"generic/platform=iOS".to_string(),
///     &Configuration::Release,
///     &SDK::Iphoneos,
///     &"build/MyApp.xcarchive".to_string(),
///     &Some("MyApp.xcodeproj".to_string()),
///     &None,
/// );
/// // In a real scenario with a valid project, this would either succeed or
/// // fail based on the actual build results
/// ```
///
/// ## Archiving with workspace parameter (will attempt to create archive):
/// ```rust,no_run
/// use xctools_archive::archive;
/// use xcbuild_common::{Configuration, SDK};
///
/// // This example shows the function signature but doesn't run
/// // because it would try to execute xcodebuild archive with a non-existent workspace
/// let result = archive(
///     &"MyApp".to_string(),
///     &"generic/platform=macOS".to_string(),
///     &Configuration::Release,
///     &SDK::Macosx,
///     &"archives/MyApp.xcarchive".to_string(),
///     &None,
///     &Some("MyApp.xcworkspace".to_string()),
/// );
/// // In a real scenario with a valid workspace, this would either succeed or
/// // fail based on the actual build results
/// ```
///
/// ## Using the xctools CLI:
/// ```bash
/// # Create iOS archive with project file and Release configuration
/// xctools archive --schema MyApp --destination "generic/platform=iOS" --sdk iphoneos --output MyApp.xcarchive --project MyApp.xcodeproj --configuration release
///
/// # Create macOS archive with workspace file
/// xctools archive --schema MyApp --destination "generic/platform=macOS" --sdk macosx --output MyApp.xcarchive --workspace MyApp.xcworkspace --configuration release
/// ```
///
/// # Generated Command
///
/// The function generates an xcodebuild command in the format:
/// ```bash
/// xcodebuild archive -project MyApp.xcodeproj -scheme MyApp -destination 'generic/platform=iOS' -configuration Release -sdk iphoneos -archivePath MyApp.xcarchive
/// ```
///
/// # Archive Contents
///
/// The generated .xcarchive bundle contains:
/// - **Applications/**: The built application bundle (.app)
/// - **dSYMs/**: Debug symbol files for crash symbolication
/// - **Info.plist**: Archive metadata including creation date, Xcode version, and scheme information
/// - **Products/**: Additional products and frameworks
///
/// # Archive Types and Use Cases
///
/// Archives are used for various distribution scenarios:
/// - **App Store Distribution**: Submit to Apple's App Store Connect
/// - **Enterprise Distribution**: Internal company app distribution  
/// - **Ad Hoc Distribution**: Limited device testing and distribution
/// - **Development Distribution**: Testing with development certificates
/// - **Crash Symbolication**: Debug symbols for analyzing crash reports
/// - **Backup and Versioning**: Preserve specific builds for future reference
///
/// # Best Practices
///
/// - Always use Release configuration for distribution archives
/// - Use generic destinations (e.g., "generic/platform=iOS") for archives
/// - Ensure proper code signing certificates are installed
/// - Store archives in a consistent directory structure
/// - Include version information in archive names
/// - Keep archives for crash symbolication and debugging
/// - Use SDK::Iphoneos for iOS applications
/// - Use SDK::Macosx for macOS applications
///
/// # Requirements
///
/// - Xcode must be installed and `xcodebuild` must be available in PATH
/// - The specified project/workspace file must exist
/// - The specified scheme must exist in the project/workspace
/// - Valid code signing certificates must be configured
/// - The destination must be valid for the target platform and SDK
/// - Sufficient disk space for the archive output
/// - Write permissions for the output directory
/// - The SDK must match the target platform
pub fn archive(
    schema: &String,
    destination: &String,
    configuration: &Configuration,
    sdk: &SDK,
    output: &String,
    project: &Option<String>,
    workspace: &Option<String>,
) -> Result<String> {
    let target = BuildTarget::new(project.as_ref(), workspace.as_ref());
    let output = run_xcodebuild_command(
        &XcodebuildCommandAction::Archive,
        schema,
        destination,
        configuration,
        &target,
        Some(sdk),
        Some(output),
    )?;

    Ok(output)
}

#[cfg(test)]
mod tests {}
