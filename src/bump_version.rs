use anyhow::{Result, anyhow, bail};
use glob::glob;
use semver::Version;
use std::fs;
use std::path::{Path, PathBuf};

pub fn bump_version(
    build_number: &Option<i32>,
    version_number: &Option<Version>,
) -> Result<String> {
    let pbxproj_filepath = match find_first_pbxproj_filepath() {
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
    let write_result = fs::write(&pbxproj_filepath, updated_content).map_err(|e| {
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
        "Successfully updated project.pbxproj at: {}\nBuild number set to: {:?}\nVersion number set to: {:?}\n",
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

fn find_first_pbxproj_filepath() -> Option<PathBuf> {
    glob("**/project.pbxproj")
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
    fs::read_to_string(path).map_err(|e| {
        anyhow!(
            "Failed to read project.pbxproj file at {}: {}",
            path.display(),
            e
        )
    })
}
