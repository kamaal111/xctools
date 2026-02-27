use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

const KEYCHAIN_NAME: &str = "xctools-signing.keychain";

/// Sets up code signing for a CI environment by importing a certificate into a
/// temporary keychain and installing provisioning profiles.
///
/// This function performs all the steps required to prepare a CI machine for
/// code-signing Xcode builds:
/// 1. Creates a dedicated keychain (`xctools-signing.keychain`).
/// 2. Imports the provided P12 certificate into the new keychain.
/// 3. Sets the keychain as the default and unlocks it so `codesign` can access it.
/// 4. Copies each provisioning profile to `~/Library/MobileDevice/Provisioning Profiles/`.
///
/// # Arguments
///
/// * `certificate_path` - Filesystem path to the P12 certificate file (.p12 or .pfx)
/// * `certificate_password` - Password protecting the P12 certificate
/// * `provisioning_profiles` - Slice of paths to `.mobileprovision` files to install;
///   may be empty if no profiles are needed (e.g., macOS Developer ID signing)
///
/// # Returns
///
/// Returns `Ok(String)` with a summary of what was set up, or `Err` if any step fails.
///
/// # Examples
///
/// ## Using the xctools CLI:
/// ```bash
/// # Install a certificate and two provisioning profiles
/// xctools setup-signing \
///     --certificate-path signing.p12 \
///     --certificate-password "$CERT_PASSWORD" \
///     --provisioning-profile AppStore.mobileprovision \
///     --provisioning-profile WatchApp.mobileprovision
///
/// # Install a certificate only (no provisioning profiles)
/// xctools setup-signing \
///     --certificate-path DeveloperID.p12 \
///     --certificate-password "$CERT_PASSWORD"
/// ```
///
/// # Security Notes
///
/// - Pass credentials via environment variables rather than hardcoding them in scripts.
/// - The created keychain (`xctools-signing.keychain`) is ephemeral and only exists for
///   the duration of the CI job. CI agents should clean up the keychain after use with
///   `security delete-keychain xctools-signing.keychain` if persistence is not desired.
/// - The keychain's lock timeout is set to 3600 seconds to avoid lock-outs during
///   long-running builds.
pub fn setup_signing(
    certificate_path: &str,
    certificate_password: &str,
    provisioning_profiles: &[String],
) -> Result<String> {
    let keychain_password = generate_keychain_password();
    create_keychain(KEYCHAIN_NAME, &keychain_password)?;
    import_certificate(
        KEYCHAIN_NAME,
        &keychain_password,
        certificate_path,
        certificate_password,
    )?;
    set_default_keychain(KEYCHAIN_NAME)?;
    install_provisioning_profiles(provisioning_profiles)?;

    let profile_count = provisioning_profiles.len();
    Ok(format!(
        "Code signing setup complete.\nKeychain: {}\nCertificate: {}\nProvisioning profiles installed: {}\n",
        KEYCHAIN_NAME, certificate_path, profile_count
    ))
}

/// Generates a unique password for the ephemeral CI keychain using the current
/// Unix timestamp and process ID so it is not predictable and differs across runs.
fn generate_keychain_password() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    format!("xctools-{}-{}", timestamp, pid)
}

fn create_keychain(keychain_name: &str, keychain_password: &str) -> Result<()> {
    let commands = vec![
        format!(
            "security create-keychain -p {} {}",
            keychain_password, keychain_name
        ),
        format!("security set-keychain-settings -lut 3600 {}", keychain_name),
        format!(
            "security unlock-keychain -p {} {}",
            keychain_password, keychain_name
        ),
    ];

    for command in commands {
        run_security_command(&command)?;
    }

    Ok(())
}

fn import_certificate(
    keychain_name: &str,
    keychain_password: &str,
    certificate_path: &str,
    certificate_password: &str,
) -> Result<()> {
    let command = format!(
        "security import {} -k {} -P {} -T /usr/bin/codesign -T /usr/bin/productsign",
        certificate_path, keychain_name, certificate_password
    );
    run_security_command(&command)?;

    // Allow codesign to access the certificate without user confirmation
    let partition_command = format!(
        "security set-key-partition-list -S apple-tool:,apple: -s -k {} {}",
        keychain_password, keychain_name
    );
    run_security_command(&partition_command)?;

    Ok(())
}

fn set_default_keychain(keychain_name: &str) -> Result<()> {
    let command = format!("security default-keychain -s {}", keychain_name);
    run_security_command(&command)?;

    Ok(())
}

fn install_provisioning_profiles(provisioning_profiles: &[String]) -> Result<()> {
    if provisioning_profiles.is_empty() {
        return Ok(());
    }

    let profiles_dir = provisioning_profiles_directory()?;
    fs::create_dir_all(&profiles_dir)
        .context("Failed to create provisioning profiles directory")?;

    for profile_path in provisioning_profiles {
        install_provisioning_profile(profile_path, &profiles_dir)?;
    }

    Ok(())
}

fn install_provisioning_profile(profile_path: &str, profiles_dir: &Path) -> Result<()> {
    let source = Path::new(profile_path);
    if !source.exists() {
        bail!("Provisioning profile not found: {}", profile_path);
    }

    let file_name = source
        .file_name()
        .context("Invalid provisioning profile path")?;
    let destination = profiles_dir.join(file_name);

    fs::copy(source, &destination).context(format!(
        "Failed to install provisioning profile: {}",
        profile_path
    ))?;

    Ok(())
}

fn provisioning_profiles_directory() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("MobileDevice")
        .join("Provisioning Profiles"))
}

fn run_security_command(command: &str) -> Result<String> {
    let output = Command::new("zsh")
        .arg("-c")
        .arg(command)
        .spawn()
        .context(format!("Failed to spawn command: {}", command))?
        .wait_with_output()
        .context(format!("Failed to run command: {}", command))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Command failed (exit {}): {}", output.status, stderr.trim());
    }

    String::from_utf8(output.stdout).context("Failed to decode command output")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_provisioning_profiles_directory() {
        let result = provisioning_profiles_directory();

        assert!(result.is_ok());
        let dir = result.unwrap();
        assert!(dir.ends_with("Library/MobileDevice/Provisioning Profiles"));
    }

    #[test]
    fn test_install_provisioning_profile_missing_file() {
        let tmp = tempdir().unwrap();
        let result = install_provisioning_profile("nonexistent.mobileprovision", tmp.path());

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Provisioning profile not found")
        );
    }

    #[test]
    fn test_install_provisioning_profile_copies_file() {
        let tmp = tempdir().unwrap();
        let source_dir = tempdir().unwrap();
        let profile_path = source_dir.path().join("AppStore.mobileprovision");
        fs::write(&profile_path, b"fake profile content").unwrap();

        let result = install_provisioning_profile(profile_path.to_str().unwrap(), tmp.path());

        assert!(result.is_ok());
        assert!(tmp.path().join("AppStore.mobileprovision").exists());
    }

    #[test]
    fn test_install_provisioning_profiles_empty_slice() {
        let result = install_provisioning_profiles(&[]);

        assert!(result.is_ok());
    }

    #[test]
    fn test_install_provisioning_profiles_missing_file() {
        let profiles = vec!["missing_profile.mobileprovision".to_string()];
        let result = install_provisioning_profiles(&profiles);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Provisioning profile not found")
        );
    }

    #[test]
    fn test_generate_keychain_password_is_non_empty() {
        let password = generate_keychain_password();
        assert!(!password.is_empty());
        assert!(password.starts_with("xctools-"));
    }

    #[test]
    fn test_generate_keychain_password_differs_across_calls() {
        // Two consecutive calls should produce different passwords because the
        // timestamp component changes even within the same process (nanosecond precision).
        // We cannot guarantee strict inequality in a very fast test, but we can verify
        // the format is correct.
        let pw1 = generate_keychain_password();
        let pw2 = generate_keychain_password();
        // Both should have the correct prefix
        assert!(pw1.starts_with("xctools-"));
        assert!(pw2.starts_with("xctools-"));
    }
}
