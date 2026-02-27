use std::process::Command;

use anyhow::{Context, Result};

/// Notarizes a macOS application using Apple's notarization service.
///
/// This function submits a macOS application, disk image (.dmg), or package (.pkg) to
/// Apple's notarization service using `xcrun notarytool submit`, waits for the notarization
/// to complete, and then staples the resulting notarization ticket to the file using
/// `xcrun stapler staple`.
///
/// Notarization is required for distributing macOS apps outside the Mac App Store on
/// macOS 10.15 (Catalina) and later. Without notarization, Gatekeeper will prevent
/// users from opening the application.
///
/// # Arguments
///
/// * `file_path` - Path to the file to notarize (.app bundle inside a .zip, .dmg, or .pkg)
/// * `apple_id` - Apple ID email address associated with the developer account
/// * `password` - App-specific password generated at appleid.apple.com for the Apple ID
/// * `team_id` - The 10-character Apple Developer Team ID (e.g., "A1B2C3D4E5")
///
/// # Returns
///
/// Returns `Ok(String)` with a success message on successful notarization and stapling,
/// or `Err` if submission, notarization, or stapling fails.
///
/// # Examples
///
/// ## Using the xctools CLI:
/// ```bash
/// xctools notarize --file-path MyApp.dmg --apple-id developer@example.com \
///     --password app-specific-password --team-id A1B2C3D4E5
/// ```
///
/// # Notes
///
/// - An app-specific password is required; generate one at <https://appleid.apple.com>.
/// - The `--wait` flag is passed to `notarytool`, so the command blocks until
///   notarization finishes (or fails). Typical notarization takes 1–5 minutes.
/// - After successful notarization, the ticket is stapled to the file so that
///   Gatekeeper can verify it offline.
/// - Requires Xcode 13 or later (`xcrun notarytool` was introduced in Xcode 13).
pub fn notarize(file_path: &str, apple_id: &str, password: &str, team_id: &str) -> Result<String> {
    let submit_output = run_notarytool_submit(file_path, apple_id, password, team_id)?;
    let staple_output = run_stapler_staple(file_path)?;

    Ok(format!("{}{}", submit_output, staple_output))
}

fn run_notarytool_submit(
    file_path: &str,
    apple_id: &str,
    password: &str,
    team_id: &str,
) -> Result<String> {
    let command = make_notarytool_submit_command(file_path, apple_id, password, team_id);
    let output = Command::new("zsh")
        .arg("-c")
        .arg(&command)
        .spawn()
        .context("Failed to run xcrun notarytool submit")?
        .wait_with_output()
        .context("Failed to run xcrun notarytool submit")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "xcrun notarytool submit failed (exit {}): {}",
            output.status,
            stderr.trim()
        );
    }

    String::from_utf8(output.stdout).context("Failed to decode notarytool output")
}

fn run_stapler_staple(file_path: &str) -> Result<String> {
    let command = make_stapler_staple_command(file_path);
    let output = Command::new("zsh")
        .arg("-c")
        .arg(&command)
        .spawn()
        .context("Failed to run xcrun stapler staple")?
        .wait_with_output()
        .context("Failed to run xcrun stapler staple")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "xcrun stapler staple failed (exit {}): {}",
            output.status,
            stderr.trim()
        );
    }

    String::from_utf8(output.stdout).context("Failed to decode stapler output")
}

fn make_notarytool_submit_command(
    file_path: &str,
    apple_id: &str,
    password: &str,
    team_id: &str,
) -> String {
    format!(
        "xcrun notarytool submit {} --apple-id {} --password {} --team-id {} --wait",
        file_path, apple_id, password, team_id
    )
}

fn make_stapler_staple_command(file_path: &str) -> String {
    format!("xcrun stapler staple {}", file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_notarytool_submit_command() {
        let result = make_notarytool_submit_command(
            "MyApp.dmg",
            "developer@example.com",
            "app-specific-password",
            "A1B2C3D4E5",
        );

        assert_eq!(
            result,
            "xcrun notarytool submit MyApp.dmg --apple-id developer@example.com \
             --password app-specific-password --team-id A1B2C3D4E5 --wait"
        );
    }

    #[test]
    fn test_make_notarytool_submit_command_with_pkg() {
        let result = make_notarytool_submit_command(
            "/path/to/MyApp.pkg",
            "mac.developer@example.com",
            "secure-password",
            "Z9Y8X7W6V5",
        );

        assert_eq!(
            result,
            "xcrun notarytool submit /path/to/MyApp.pkg --apple-id mac.developer@example.com \
             --password secure-password --team-id Z9Y8X7W6V5 --wait"
        );
    }

    #[test]
    fn test_make_notarytool_submit_command_with_zip() {
        let result = make_notarytool_submit_command(
            "./build/MyApp.zip",
            "team@company.com",
            "xxxx-xxxx-xxxx-xxxx",
            "TEAMID1234",
        );

        assert_eq!(
            result,
            "xcrun notarytool submit ./build/MyApp.zip --apple-id team@company.com \
             --password xxxx-xxxx-xxxx-xxxx --team-id TEAMID1234 --wait"
        );
    }

    #[test]
    fn test_make_stapler_staple_command() {
        let result = make_stapler_staple_command("MyApp.dmg");

        assert_eq!(result, "xcrun stapler staple MyApp.dmg");
    }

    #[test]
    fn test_make_stapler_staple_command_with_path() {
        let result = make_stapler_staple_command("/path/to/MyApp.pkg");

        assert_eq!(result, "xcrun stapler staple /path/to/MyApp.pkg");
    }
}
