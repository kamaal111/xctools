use anyhow::{anyhow, bail, Result};
use glob::glob;
use semver::Version;
use std::path::{Path, PathBuf};

/// Updates version numbers and build numbers in Xcode project files.
///
/// This function searches for the first `project.pbxproj` file in the current directory or its
/// subdirectories, then updates the `CURRENT_PROJECT_VERSION` (build number) and/or
/// `MARKETING_VERSION` (version number) fields within that file. It preserves the original
/// file formatting and indentation.
///
/// # Arguments
///
/// * `build_number` - Optional build number to set. If provided, updates `CURRENT_PROJECT_VERSION`
///   in the project.pbxproj file (e.g., 42, 100, 1337)
/// * `version_number` - Optional semantic version to set. If provided, updates `MARKETING_VERSION`
///   in the project.pbxproj file (e.g., "1.0.0", "2.1.3", "0.5.0-beta")
///
/// # Returns
///
/// Returns `Ok(String)` with a success message indicating what was updated and where,
/// or `Err` if the operation fails (e.g., no project.pbxproj found, file read/write errors).
///
/// # Examples
///
/// ## Testing with no project.pbxproj file (will fail):
/// ```rust
/// use xctools_bump_version::bump_version;
/// use semver::Version;
///
/// // This will fail because no project.pbxproj exists in a clean test environment
/// let result = bump_version(&Some(42), &None);
/// assert!(result.is_err());
/// assert!(result.unwrap_err().to_string().contains("No project.pbxproj found"));
/// ```
///
/// ## Testing parameter validation with Version parsing:
/// ```rust
/// use xctools_bump_version::bump_version;
/// use semver::Version;
///
/// // Test that Version parsing works correctly for different formats
/// let version_1_0_0 = Version::parse("1.0.0").unwrap();
/// assert_eq!(version_1_0_0.major, 1);
/// assert_eq!(version_1_0_0.minor, 0);
/// assert_eq!(version_1_0_0.patch, 0);
///
/// let version_beta = Version::parse("2.1.0-beta.1").unwrap();
/// assert_eq!(version_beta.major, 2);
/// assert_eq!(version_beta.minor, 1);
/// assert_eq!(version_beta.patch, 0);
/// assert!(!version_beta.pre.is_empty());
/// ```
///
/// ## Testing edge cases with parameters:
/// ```rust
/// use xctools_bump_version::bump_version;
/// use semver::Version;
///
/// // Test with both parameters as None (should still fail due to no project file)
/// let result = bump_version(&None, &None);
/// assert!(result.is_err());
///
/// // Test with negative build number
/// let result = bump_version(&Some(-1), &None);
/// assert!(result.is_err());
/// ```
///
/// ## Demonstrating function signature (no_run):
/// ```rust,no_run
/// use xctools_bump_version::bump_version;
/// use semver::Version;
///
/// // Update build number only
/// let result = bump_version(&Some(42), &None);
/// match result {
///     Ok(message) => println!("{}", message),
///     Err(e) => eprintln!("Error: {}", e),
/// }
///
/// // Update version number only
/// let version = Version::parse("2.1.0").unwrap();
/// let result = bump_version(&None, &Some(version));
///
/// // Update both build and version numbers
/// let version = Version::parse("1.5.0").unwrap();
/// let result = bump_version(&Some(100), &Some(version));
/// ```
///
/// ## Using the xctools CLI:
/// ```bash
/// # Update build number only
/// xctools bump-version --build-number 42
///
/// # Update version number only
/// xctools bump-version --version-number 2.1.0
///
/// # Update both build and version numbers
/// xctools bump-version --build-number 42 --version-number 2.1.0
/// ```
///
/// # File Changes
///
/// The function modifies lines in project.pbxproj that match these patterns:
/// - `CURRENT_PROJECT_VERSION = <old_value>;` → `CURRENT_PROJECT_VERSION = <new_build_number>;`
/// - `MARKETING_VERSION = <old_value>;` → `MARKETING_VERSION = <new_version>;`
///
/// # Requirements
///
/// - At least one `project.pbxproj` file must exist in the current directory or subdirectories
/// - Write permissions for the project.pbxproj file
/// - At least one of `build_number` or `version_number` must be provided
pub fn bump_version(
    build_number: &Option<i32>,
    version_number: &Option<Version>,
) -> Result<String> {
    bump_version_in_path(build_number, version_number, None)
}

fn bump_version_in_path(
    build_number: &Option<i32>,
    version_number: &Option<Version>,
    search_path: Option<&Path>,
) -> Result<String> {
    let pbxproj_filepath = match find_first_pbxproj_filepath(search_path) {
        None => bail!("No project.pbxproj found"),
        Some(pbxproj_filepath) => pbxproj_filepath,
    };
    let content = match read_pbxproj_file(&pbxproj_filepath) {
        Err(error) => return Err(error),
        Ok(content) => content,
    };
    let updated_content = content
        .lines()
        .map(|line| replace_pbxproj_line(line, build_number, version_number))
        .collect::<Vec<String>>()
        .join("\n");
    let write_result = std::fs::write(&pbxproj_filepath, updated_content).map_err(|e| {
        anyhow!(
            "Failed to write updated content to {}: {}",
            pbxproj_filepath.display(),
            e
        )
    });
    if let Err(error) = write_result {
        return Err(error);
    }

    Ok(format!(
        "Successfully updated project.pbxproj at: {}\nBuild number set to: {}\nVersion number set to: {}\n",
        pbxproj_filepath.display(),
        build_number
            .map(|number| number.to_string())
            .unwrap_or(String::from("UNSET")),
        version_number
            .clone()
            .map(|version| version.to_string())
            .unwrap_or(String::from("UNSET"))
    ))
}

fn replace_pbxproj_line(
    line: &str,
    build_number: &Option<i32>,
    version_number: &Option<Version>,
) -> String {
    let formatted_line = replace_key_value_line(
        line,
        "CURRENT_PROJECT_VERSION",
        &build_number.map(|number| number.to_string()),
    );

    replace_key_value_line(
        &formatted_line,
        "MARKETING_VERSION",
        &version_number.as_ref().map(|version| version.to_string()),
    )
}

fn replace_key_value_line(line: &str, key: &str, value: &Option<String>) -> String {
    if !line.contains(key) || value.is_none() {
        return line.to_string();
    }

    let trimmed = line.trim();
    let indent = &line[..line.len() - trimmed.len()];

    format!("{}{} = {};", indent, key, value.as_ref().unwrap())
}

fn find_first_pbxproj_filepath(search_path: Option<&Path>) -> Option<PathBuf> {
    let pattern = match search_path {
        Some(path) => format!("{}/**/project.pbxproj", path.display()),
        None => "**/project.pbxproj".to_string(),
    };

    glob(&pattern)
        .map(|paths| {
            let mut files: Vec<_> = paths
                .filter_map(|entry| entry.ok())
                .filter(|path| path.exists() && path.is_file())
                .collect();
            // Sort for consistent retrieval
            files.sort();

            files.first().cloned()
        })
        .unwrap_or(None)
}

fn read_pbxproj_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|e| {
        anyhow!(
            "Failed to read project.pbxproj file at {}: {}",
            path.display(),
            e
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_no_pbxproj() {
        let tmp = tempdir().unwrap();
        let err = bump_version_in_path(&Some(1), &None, Some(tmp.path())).unwrap_err();

        assert!(err.to_string().contains("No project.pbxproj found"));
    }

    #[test]
    fn test_bump_both() {
        let tmp = tempdir().unwrap();
        let filepath = tmp.path().join("project.pbxproj");
        std::fs::write(
            &filepath,
            "CURRENT_PROJECT_VERSION = 1;\n    MARKETING_VERSION = 1.0.0;\nOtherKey = 42;",
        )
        .unwrap();
        let output = bump_version_in_path(
            &Some(5),
            &Some(Version::parse("2.3.4").unwrap()),
            Some(tmp.path()),
        )
        .unwrap();

        assert!(output.contains("Build number set to: 5"));
        assert!(output.contains("Version number set to: 2.3.4"));

        let content = std::fs::read_to_string(&filepath).unwrap();

        assert!(content.contains("CURRENT_PROJECT_VERSION = 5;"));
        assert!(content.contains("MARKETING_VERSION = 2.3.4;"));
    }

    #[test]
    fn test_bump_build_only() {
        let tmp = tempdir().unwrap();
        let filepath = tmp.path().join("project.pbxproj");
        std::fs::write(
            &filepath,
            "CURRENT_PROJECT_VERSION = 9;\nMARKETING_VERSION = 3.0.0;",
        )
        .unwrap();
        let _ = bump_version_in_path(&Some(10), &None, Some(tmp.path())).unwrap();
        let content = std::fs::read_to_string(&filepath).unwrap();

        assert!(content.contains("CURRENT_PROJECT_VERSION = 10;"));
        assert!(content.contains("MARKETING_VERSION = 3.0.0;"));
    }

    #[test]
    fn test_bump_version_only() {
        let tmp = tempdir().unwrap();
        let filepath = tmp.path().join("project.pbxproj");
        std::fs::write(
            &filepath,
            "CURRENT_PROJECT_VERSION = 7;\n    MARKETING_VERSION = 0.1.0;",
        )
        .unwrap();
        let _ = bump_version_in_path(
            &None,
            &Some(Version::parse("0.2.0").unwrap()),
            Some(tmp.path()),
        )
        .unwrap();
        let content = std::fs::read_to_string(&filepath).unwrap();

        assert!(content.contains("CURRENT_PROJECT_VERSION = 7;"));
        assert!(content.contains("MARKETING_VERSION = 0.2.0;"));
    }

    #[test]
    fn test_replace_key_value_line() {
        // Test the helper function directly
        let line = "    CURRENT_PROJECT_VERSION = 1;";
        let result =
            replace_key_value_line(line, "CURRENT_PROJECT_VERSION", &Some("42".to_string()));

        assert_eq!(result, "    CURRENT_PROJECT_VERSION = 42;");

        // Test with no value (should return original line)
        let result2 = replace_key_value_line(line, "CURRENT_PROJECT_VERSION", &None);

        assert_eq!(result2, line);

        // Test with non-matching key
        let result3 = replace_key_value_line(line, "OTHER_KEY", &Some("42".to_string()));

        assert_eq!(result3, line);
    }

    #[test]
    fn test_replace_pbxproj_line() {
        // Test replacing both values
        let line = "    CURRENT_PROJECT_VERSION = 1;";
        let result = replace_pbxproj_line(line, &Some(42), &Some(Version::parse("2.0.0").unwrap()));

        assert_eq!(result, "    CURRENT_PROJECT_VERSION = 42;");

        let line2 = "        MARKETING_VERSION = 1.0.0;";
        let result2 =
            replace_pbxproj_line(line2, &Some(42), &Some(Version::parse("2.0.0").unwrap()));

        assert_eq!(result2, "        MARKETING_VERSION = 2.0.0;");

        // Test line that doesn't match either key
        let line3 = "    OTHER_KEY = value;";
        let result3 =
            replace_pbxproj_line(line3, &Some(42), &Some(Version::parse("2.0.0").unwrap()));

        assert_eq!(result3, line3);
    }
}
