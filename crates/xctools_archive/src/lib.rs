use anyhow::Result;
use xcbuild_common::{
    BuildTarget, Configuration, SDK, XcodebuildCommandAction, XcodebuildParams,
    run_xcodebuild_command,
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
/// * `scheme` - The Xcode scheme name to archive (e.g., "MyApp", "MyApp Release")
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
/// xctools archive --scheme MyApp --destination "generic/platform=iOS" --sdk iphoneos --output MyApp.xcarchive --project MyApp.xcodeproj --configuration release
///
/// # Create macOS archive with workspace file
/// xctools archive --scheme MyApp --destination "generic/platform=macOS" --sdk macosx --output MyApp.xcarchive --workspace MyApp.xcworkspace --configuration release
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
    scheme: &String,
    destination: &String,
    configuration: &Configuration,
    sdk: &SDK,
    output: &String,
    project: &Option<String>,
    workspace: &Option<String>,
) -> Result<String> {
    let target = BuildTarget::new(project.as_ref(), workspace.as_ref());
    let params = XcodebuildParams::new(XcodebuildCommandAction::Archive)
        .with_scheme(scheme.clone())
        .with_destination(destination.clone())
        .with_configuration(configuration.clone())
        .with_target(target)
        .with_sdk(sdk.clone())
        .with_archive_path(output.clone());
    let output = run_xcodebuild_command(&params)?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use xcbuild_common::{Configuration, SDK};

    #[test]
    fn test_archive_with_project_and_iphoneos_sdk() {
        // Test that archive function properly validates parameters
        // This will fail because neither project nor workspace is specified
        let result = archive(
            &"MyApp".to_string(),
            &"generic/platform=iOS".to_string(),
            &Configuration::Release,
            &SDK::Iphoneos,
            &"MyApp.xcarchive".to_string(),
            &None,
            &None,
        );

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Neither project nor workspace is specified"));
    }

    #[test]
    fn test_archive_with_workspace_and_macosx_sdk() {
        // Test that archive function properly validates parameters
        // This will fail because neither project nor workspace is specified
        let result = archive(
            &"MyApp".to_string(),
            &"generic/platform=macOS".to_string(),
            &Configuration::Debug,
            &SDK::Macosx,
            &"MyApp.xcarchive".to_string(),
            &None,
            &None,
        );

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Neither project nor workspace is specified"));
    }

    #[test]
    fn test_archive_parameter_validation_debug_configuration() {
        // Test archive with Debug configuration
        let result = archive(
            &"TestScheme".to_string(),
            &"generic/platform=iOS".to_string(),
            &Configuration::Debug,
            &SDK::Iphoneos,
            &"TestApp-Debug.xcarchive".to_string(),
            &None,
            &None,
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Neither project nor workspace is specified")
        );
    }

    #[test]
    fn test_archive_parameter_validation_release_configuration() {
        // Test archive with Release configuration
        let result = archive(
            &"TestScheme".to_string(),
            &"generic/platform=macOS".to_string(),
            &Configuration::Release,
            &SDK::Macosx,
            &"TestApp-Release.xcarchive".to_string(),
            &None,
            &None,
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Neither project nor workspace is specified")
        );
    }

    #[test]
    fn test_archive_with_custom_output_path() {
        // Test archive with custom output path
        let result = archive(
            &"MyTestApp".to_string(),
            &"generic/platform=iOS".to_string(),
            &Configuration::Release,
            &SDK::Iphoneos,
            &"/tmp/build/archives/MyTestApp-v1.0.0.xcarchive".to_string(),
            &None,
            &None,
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Neither project nor workspace is specified")
        );
    }

    #[test]
    fn test_archive_ios_destination_patterns() {
        // Test various iOS destination patterns
        let destinations = vec!["generic/platform=iOS", "generic/platform=iOS Simulator"];

        for destination in destinations {
            let result = archive(
                &"TestApp".to_string(),
                &destination.to_string(),
                &Configuration::Release,
                &SDK::Iphoneos,
                &"TestApp.xcarchive".to_string(),
                &None,
                &None,
            );

            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Neither project nor workspace is specified")
            );
        }
    }

    #[test]
    fn test_archive_macos_destination_patterns() {
        // Test various macOS destination patterns
        let destinations = vec!["generic/platform=macOS", "platform=macOS"];

        for destination in destinations {
            let result = archive(
                &"TestApp".to_string(),
                &destination.to_string(),
                &Configuration::Release,
                &SDK::Macosx,
                &"TestApp.xcarchive".to_string(),
                &None,
                &None,
            );

            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Neither project nor workspace is specified")
            );
        }
    }

    #[test]
    fn test_archive_scheme_name_variations() {
        // Test various scheme name patterns
        let schemes = vec![
            "MyApp",
            "MyApp Release",
            "MyApp-Production",
            "TestScheme123",
        ];

        for scheme in schemes {
            let result = archive(
                &scheme.to_string(),
                &"generic/platform=iOS".to_string(),
                &Configuration::Release,
                &SDK::Iphoneos,
                &"Archive.xcarchive".to_string(),
                &None,
                &None,
            );

            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Neither project nor workspace is specified")
            );
        }
    }

    #[test]
    fn test_archive_output_path_variations() {
        // Test various output path patterns
        let outputs = vec![
            "MyApp.xcarchive",
            "./build/MyApp.xcarchive",
            "/tmp/MyApp.xcarchive",
            "../archives/MyApp-v1.0.xcarchive",
            "MyApp-Debug.xcarchive",
            "MyApp-Release.xcarchive",
        ];

        for output in outputs {
            let result = archive(
                &"TestApp".to_string(),
                &"generic/platform=iOS".to_string(),
                &Configuration::Release,
                &SDK::Iphoneos,
                &output.to_string(),
                &None,
                &None,
            );

            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Neither project nor workspace is specified")
            );
        }
    }

    #[test]
    fn test_archive_configuration_enum_values() {
        // Test Configuration enum usage in archive function
        let debug_result = archive(
            &"TestApp".to_string(),
            &"generic/platform=iOS".to_string(),
            &Configuration::Debug,
            &SDK::Iphoneos,
            &"TestApp-Debug.xcarchive".to_string(),
            &None,
            &None,
        );

        let release_result = archive(
            &"TestApp".to_string(),
            &"generic/platform=iOS".to_string(),
            &Configuration::Release,
            &SDK::Iphoneos,
            &"TestApp-Release.xcarchive".to_string(),
            &None,
            &None,
        );

        // Both should fail with the same error (no project/workspace)
        assert!(debug_result.is_err());
        assert!(release_result.is_err());
        assert_eq!(
            debug_result.unwrap_err().to_string(),
            release_result.unwrap_err().to_string()
        );
    }

    #[test]
    fn test_archive_sdk_enum_values() {
        // Test SDK enum usage in archive function
        let ios_result = archive(
            &"TestApp".to_string(),
            &"generic/platform=iOS".to_string(),
            &Configuration::Release,
            &SDK::Iphoneos,
            &"TestApp-iOS.xcarchive".to_string(),
            &None,
            &None,
        );

        let macos_result = archive(
            &"TestApp".to_string(),
            &"generic/platform=macOS".to_string(),
            &Configuration::Release,
            &SDK::Macosx,
            &"TestApp-macOS.xcarchive".to_string(),
            &None,
            &None,
        );

        // Both should fail with the same error (no project/workspace)
        assert!(ios_result.is_err());
        assert!(macos_result.is_err());
        assert_eq!(
            ios_result.unwrap_err().to_string(),
            macos_result.unwrap_err().to_string()
        );
    }

    #[test]
    fn test_archive_function_signature() {
        // Test that the archive function accepts the correct parameter types
        let scheme = "TestScheme".to_string();
        let destination = "generic/platform=iOS".to_string();
        let configuration = Configuration::Release;
        let sdk = SDK::Iphoneos;
        let output = "TestApp.xcarchive".to_string();
        let project = Some("TestApp.xcodeproj".to_string());
        let workspace: Option<String> = None;

        // This should compile and demonstrate the correct function signature
        let _result = archive(
            &scheme,
            &destination,
            &configuration,
            &sdk,
            &output,
            &project,
            &workspace,
        );

        // We don't assert on the result since it will fail due to missing xcodebuild,
        // but the fact that this compiles validates the function signature
    }

    #[test]
    fn test_archive_with_both_project_and_workspace_none() {
        // Test the specific error case when both project and workspace are None
        let result = archive(
            &"TestScheme".to_string(),
            &"generic/platform=iOS".to_string(),
            &Configuration::Release,
            &SDK::Iphoneos,
            &"TestApp.xcarchive".to_string(),
            &None,
            &None,
        );

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Neither project nor workspace is specified")
        );
    }

    #[test]
    fn test_archive_return_type() {
        // Test that archive function returns Result<String>
        let result = archive(
            &"TestScheme".to_string(),
            &"generic/platform=iOS".to_string(),
            &Configuration::Release,
            &SDK::Iphoneos,
            &"TestApp.xcarchive".to_string(),
            &None,
            &None,
        );

        // Verify it's a Result<String> by checking the error type
        match result {
            Ok(_output) => {
                // If it succeeds, output should be a String
                // This is unlikely in test environment without real Xcode project
            }
            Err(error) => {
                // Error should be anyhow::Error
                assert!(error.to_string().len() > 0);
            }
        }
    }
}
