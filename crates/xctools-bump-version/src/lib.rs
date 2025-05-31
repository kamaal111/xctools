use anyhow::{anyhow, bail, Result};
use glob::glob;
use semver::Version;
use std::path::{Path, PathBuf};

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
