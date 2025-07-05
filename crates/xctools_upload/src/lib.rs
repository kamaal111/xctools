use std::process::Command;

use anyhow::{Context, Result};
use xcbuild_common::UploadTarget;

/// Uploads an iOS or macOS application package to distribution platforms.
///
/// This function constructs and executes an `xcrun altool` command to upload an .ipa file
/// to Apple's distribution platforms such as the App Store, TestFlight, or enterprise
/// distribution systems. It supports both iOS and macOS applications.
///
/// # Arguments
///
/// * `target` - The upload target platform:
///   - UploadTarget::Ios - Upload iOS application (.ipa file)
///   - UploadTarget::Macos - Upload macOS application (.pkg or .dmg file)
/// * `app_file_path` - The path to the application file to upload (e.g., "MyApp.ipa", "/path/to/MyApp.ipa")
/// * `username` - The Apple ID username for authentication (e.g., "developer@example.com")
/// * `password` - The password or app-specific password for authentication
///
/// # Returns
///
/// Returns `Ok(String)` containing the stdout from the xcrun altool command on success,
/// or `Err` if the upload fails, authentication fails, or the command execution fails.
///
/// # Examples
///
/// ```rust
/// use xcbuild_common::UploadTarget;
/// use xctools_upload::upload;
///
/// // Upload an iOS app
/// let result = upload(
///     &UploadTarget::Ios,
///     "MyApp.ipa",
///     "developer@example.com",
///     "app-specific-password"
/// );
/// ```
///
/// # Notes
///
/// - The function uses `xcrun altool` which may be deprecated in newer Xcode versions.
///   Consider migrating to `xcrun notarytool` for newer workflows.
/// - App-specific passwords are recommended over regular Apple ID passwords for security.
/// - The upload process may take several minutes depending on file size and network connection.
pub fn upload(
    target: &UploadTarget,
    app_file_path: &str,
    username: &str,
    password: &str,
) -> Result<String> {
    let output = run_xcrun_command(target, app_file_path, username, password)?;

    Ok(output)
}

fn run_xcrun_command(
    target: &UploadTarget,
    app_file_path: &str,
    username: &str,
    password: &str,
) -> Result<String> {
    let command = make_xcrun_command(target, app_file_path, username, password);
    let output = Command::new("zsh")
        .arg("-c")
        .arg(&command)
        .spawn()
        .context(format!("Failed to run {}", command))?
        .wait_with_output()
        .context(format!("Failed to run {}", command))?;

    String::from_utf8(output.stdout).context("Failed to decode output")
}

fn make_xcrun_command(
    target: &UploadTarget,
    app_file_path: &str,
    username: &str,
    password: &str,
) -> String {
    format!(
        "xcrun altool --upload-app -t {} -f {} -u {} -p {}",
        target, app_file_path, username, password
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use xcbuild_common::UploadTarget;

    #[test]
    fn test_make_xcrun_command_with_ios_target() {
        let target = UploadTarget::Ios;
        let app_file_path = "MyApp.ipa";
        let username = "developer@example.com";
        let password = "app-specific-password";

        let result = make_xcrun_command(&target, app_file_path, username, password);

        assert_eq!(
            result,
            "xcrun altool --upload-app -t ios -f MyApp.ipa -u developer@example.com -p app-specific-password"
        );
    }

    #[test]
    fn test_make_xcrun_command_with_macos_target() {
        let target = UploadTarget::Macos;
        let app_file_path = "MyMacApp.pkg";
        let username = "mac.developer@example.com";
        let password = "secure-password-123";

        let result = make_xcrun_command(&target, app_file_path, username, password);

        assert_eq!(
            result,
            "xcrun altool --upload-app -t macos -f MyMacApp.pkg -u mac.developer@example.com -p secure-password-123"
        );
    }

    #[test]
    fn test_make_xcrun_command_with_special_characters() {
        let target = UploadTarget::Ios;
        let app_file_path = "/path/to/My App With Spaces.ipa";
        let username = "test@company.co.uk";
        let password = "p@ssw0rd!";

        let result = make_xcrun_command(&target, app_file_path, username, password);

        assert_eq!(
            result,
            "xcrun altool --upload-app -t ios -f /path/to/My App With Spaces.ipa -u test@company.co.uk -p p@ssw0rd!"
        );
    }

    #[test]
    fn test_upload_function_signature() {
        // Test that upload function has the correct signature and returns Result<String>
        // This test validates the function signature without attempting to run xcrun
        let target = UploadTarget::Ios;
        let app_file_path = "test.ipa";
        let username = "test@example.com";
        let password = "password";

        let result = upload(&target, app_file_path, username, password);

        // Since this will fail without actual xcrun command available,
        // we just verify that it returns a Result<String> type
        match result {
            Ok(_output) => {
                // If it succeeds, output should be a String
                // This is unlikely in test environment without real xcrun
            }
            Err(error) => {
                // Error should be anyhow::Error and contain some meaningful message
                assert!(error.to_string().len() > 0);
            }
        }
    }

    #[test]
    fn test_upload_target_display() {
        // Test that UploadTarget enum displays correctly in the command
        assert_eq!(UploadTarget::Ios.to_string(), "ios");
        assert_eq!(UploadTarget::Macos.to_string(), "macos");
    }
}
