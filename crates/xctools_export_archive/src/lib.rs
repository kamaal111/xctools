use anyhow::Result;
use xcbuild_common::{XcodebuildCommandAction, XcodebuildParams, run_xcodebuild_command};

/// Exports an Xcode archive using the `xcodebuild -exportArchive` command-line tool.
///
/// This function constructs and executes an `xcodebuild -exportArchive` command with the specified
/// parameters to export an existing .xcarchive bundle into a distributable format. The export
/// process creates an .ipa file (for iOS) or .app bundle (for macOS) that can be distributed
/// via the App Store, TestFlight, enterprise distribution, or ad-hoc distribution.
///
/// # Arguments
///
/// * `archive_path` - Path to the existing .xcarchive bundle to export (e.g., "MyApp.xcarchive")
/// * `export_options` - Path to the ExportOptions.plist file that specifies export method,
///   signing options, and other export settings (e.g., "ExportOptions.plist")
/// * `export_path` - Directory path where the exported files should be placed (e.g., "build/export")
///
/// # Returns
///
/// Returns `Ok(String)` containing the stdout from the xcodebuild -exportArchive command on success,
/// or `Err` if the export process fails.
///
/// # Examples
///
/// ## Basic export archive usage:
/// ```rust,no_run
/// use xctools_export_archive::export_archive;
///
/// // This example shows the function signature but doesn't run
/// // because it would try to execute xcodebuild -exportArchive with non-existent files
/// let result = export_archive(
///     &"MyApp.xcarchive".to_string(),
///     &"ExportOptions.plist".to_string(),
///     &"build/export".to_string(),
/// );
/// // In a real scenario with valid archive and export options, this would either
/// // succeed and create an .ipa/.app file or fail based on the export configuration
/// ```
///
/// ## Using the xctools CLI:
/// ```bash
/// # Export an archive for App Store distribution
/// xctools export-archive --archive-path MyApp.xcarchive --export-options ExportOptions.plist --export-path build/export
///
/// # The ExportOptions.plist file typically contains:
/// # <?xml version="1.0" encoding="UTF-8"?>
/// # <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
/// # <plist version="1.0">
/// # <dict>
/// #     <key>method</key>
/// #     <string>app-store</string>
/// #     <key>teamID</key>
/// #     <string>YOUR_TEAM_ID</string>
/// # </dict>
/// # </plist>
/// ```
///
/// # Export Methods
///
/// The export method is specified in the ExportOptions.plist file and can be one of:
/// - **app-store**: For App Store distribution
/// - **testflight**: For TestFlight beta distribution
/// - **ad-hoc**: For ad-hoc distribution to registered devices
/// - **enterprise**: For enterprise in-house distribution
/// - **development**: For development/debugging purposes
///
/// # Notes
///
/// - The archive must be created first using `xcodebuild archive` or the `xctools archive` command
/// - The ExportOptions.plist file must contain valid export configuration
/// - Code signing certificates and provisioning profiles must be properly configured
/// - The export path directory will be created if it doesn't exist
///
pub fn export_archive(
    archive_path: &String,
    export_options: &String,
    export_path: &String,
) -> Result<String> {
    let params = XcodebuildParams::new(XcodebuildCommandAction::ExportArchive)
        .with_archive_path(archive_path.clone())
        .with_export_options(export_options.clone())
        .with_export_path(export_path.clone());
    let output = run_xcodebuild_command(&params)?;

    Ok(output)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn assert_export_does_not_fail(result: Result<String>) {
        let output = result.ok().unwrap();
        assert!(output.is_empty() || output.len() > 0);
    }

    #[test]
    fn test_export_archive_basic_functionality() {
        // Test basic export archive function call
        let result = export_archive(
            &"MyApp.xcarchive".to_string(),
            &"ExportOptions.plist".to_string(),
            &"build/export".to_string(),
        );

        assert_export_does_not_fail(result);
    }

    #[test]
    fn test_export_archive_with_various_archive_paths() {
        // Test various archive path patterns
        let archive_paths = vec![
            "MyApp.xcarchive",
            "./build/MyApp.xcarchive",
            "/tmp/archives/MyApp.xcarchive",
            "../MyApp-Release.xcarchive",
            "MyApp-v1.0.0.xcarchive",
            "/Users/developer/Archives/MyApp.xcarchive",
        ];

        for archive_path in archive_paths {
            let result = export_archive(
                &archive_path.to_string(),
                &"ExportOptions.plist".to_string(),
                &"export".to_string(),
            );

            assert_export_does_not_fail(result);
        }
    }

    #[test]
    fn test_export_archive_with_various_export_options_paths() {
        // Test various export options plist path patterns
        let export_options_paths = vec![
            "ExportOptions.plist",
            "./ExportOptions.plist",
            "/tmp/ExportOptions.plist",
            "../config/ExportOptions.plist",
            "AppStoreExportOptions.plist",
            "AdHocExportOptions.plist",
            "EnterpriseExportOptions.plist",
        ];

        for export_options_path in export_options_paths {
            let result = export_archive(
                &"TestApp.xcarchive".to_string(),
                &export_options_path.to_string(),
                &"export".to_string(),
            );

            assert_export_does_not_fail(result);
        }
    }

    #[test]
    fn test_export_archive_with_various_export_paths() {
        // Test various export path patterns
        let export_paths = vec![
            "export",
            "./build/export",
            "/tmp/export",
            "../exports/MyApp",
            "build/iOS",
            "build/macOS",
            "/Users/developer/Exports/MyApp-v1.0",
        ];

        for export_path in export_paths {
            let result = export_archive(
                &"TestApp.xcarchive".to_string(),
                &"ExportOptions.plist".to_string(),
                &export_path.to_string(),
            );

            assert_export_does_not_fail(result);
        }
    }

    #[test]
    fn test_export_archive_app_store_scenario() {
        // Test typical App Store export scenario
        let result = export_archive(
            &"MyApp.xcarchive".to_string(),
            &"AppStoreExportOptions.plist".to_string(),
            &"build/appstore".to_string(),
        );

        assert_export_does_not_fail(result);
    }

    #[test]
    fn test_export_archive_ad_hoc_scenario() {
        // Test typical Ad Hoc export scenario
        let result = export_archive(
            &"MyApp-Release.xcarchive".to_string(),
            &"AdHocExportOptions.plist".to_string(),
            &"build/adhoc".to_string(),
        );

        assert_export_does_not_fail(result);
    }

    #[test]
    fn test_export_archive_enterprise_scenario() {
        // Test typical Enterprise export scenario
        let result = export_archive(
            &"MyEnterpriseApp.xcarchive".to_string(),
            &"EnterpriseExportOptions.plist".to_string(),
            &"build/enterprise".to_string(),
        );

        assert_export_does_not_fail(result);
    }

    #[test]
    fn test_export_archive_development_scenario() {
        // Test typical Development export scenario
        let result = export_archive(
            &"MyApp-Debug.xcarchive".to_string(),
            &"DevelopmentExportOptions.plist".to_string(),
            &"build/development".to_string(),
        );

        assert_export_does_not_fail(result);
    }

    #[test]
    fn test_export_archive_function_signature() {
        // Test that export_archive function has the correct signature
        let archive_path = "TestApp.xcarchive".to_string();
        let export_options = "ExportOptions.plist".to_string();
        let export_path = "export".to_string();

        // This should compile and demonstrate the correct function signature
        let result = export_archive(&archive_path, &export_options, &export_path);

        assert_export_does_not_fail(result);
    }

    #[test]
    fn test_export_archive_return_type() {
        // Test that export_archive function returns Result<String>
        let result = export_archive(
            &"TestApp.xcarchive".to_string(),
            &"ExportOptions.plist".to_string(),
            &"export".to_string(),
        );

        assert_export_does_not_fail(result);
    }

    #[test]
    fn test_export_archive_with_empty_strings() {
        // Test behavior with empty string parameters
        let result = export_archive(&"".to_string(), &"".to_string(), &"".to_string());

        assert_export_does_not_fail(result);
    }

    #[test]
    fn test_export_archive_with_special_characters() {
        // Test archive paths with special characters and spaces
        let result = export_archive(
            &"My App With Spaces.xcarchive".to_string(),
            &"Export_Options_AppStore.plist".to_string(),
            &"build/My App Export".to_string(),
        );

        assert_export_does_not_fail(result);
    }

    #[test]
    fn test_export_archive_parameter_types() {
        // Test that the function accepts String references
        let archive = "test.xcarchive".to_string();
        let options = "test.plist".to_string();
        let path = "test_export".to_string();

        // This demonstrates the function accepts &String parameters
        let result = export_archive(&archive, &options, &path);

        assert_export_does_not_fail(result);

        // Also test with string literals converted to String
        let result2 = export_archive(
            &"literal.xcarchive".to_string(),
            &"literal.plist".to_string(),
            &"literal_export".to_string(),
        );

        assert_export_does_not_fail(result2);
    }

    #[test]
    fn test_export_archive_ios_naming_patterns() {
        // Test typical iOS app naming patterns
        let ios_patterns = vec![
            ("MyiOSApp.xcarchive", "iOSExportOptions.plist", "build/ios"),
            (
                "MyApp-iOS.xcarchive",
                "AppStore.plist",
                "exports/ios/appstore",
            ),
            (
                "MyApp-iOS-Release.xcarchive",
                "AdHoc-iOS.plist",
                "exports/ios/adhoc",
            ),
        ];

        for (archive, options, export_path) in ios_patterns {
            let result = export_archive(
                &archive.to_string(),
                &options.to_string(),
                &export_path.to_string(),
            );

            assert_export_does_not_fail(result);
        }
    }

    #[test]
    fn test_export_archive_macos_naming_patterns() {
        // Test typical macOS app naming patterns
        let macos_patterns = vec![
            (
                "MyMacApp.xcarchive",
                "macOSExportOptions.plist",
                "build/macos",
            ),
            (
                "MyApp-macOS.xcarchive",
                "AppStore-macOS.plist",
                "exports/macos/appstore",
            ),
            (
                "MyApp-macOS-Release.xcarchive",
                "DeveloperID.plist",
                "exports/macos/developerid",
            ),
        ];

        for (archive, options, export_path) in macos_patterns {
            let result = export_archive(
                &archive.to_string(),
                &options.to_string(),
                &export_path.to_string(),
            );

            assert_export_does_not_fail(result);
        }
    }
}
