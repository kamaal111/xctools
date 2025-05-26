use anyhow::{Result, anyhow, bail};
use glob::glob;
use std::fs;
use std::path::Path;

pub fn bump_version(
    build_number: &Option<i32>,
    version_number: &Option<semver::Version>,
) -> Result<String> {
    let pbxproj_filepath = match find_first_pbxproj_filepath() {
        None => bail!("No project.pbxproj found"),
        Some(pbxproj_filepath) => pbxproj_filepath,
    };
    let content = match read_pbxproj_file(&pbxproj_filepath) {
        Err(error) => return Err(error),
        Ok(content) => content,
    };
    Ok(format!(
        "Found project.pbxproj at: {}\nBuild number to set: {:?}\nVersion number to set: {:?}\n",
        pbxproj_filepath.display(),
        build_number,
        version_number
    ))
}

fn find_first_pbxproj_filepath() -> Option<std::path::PathBuf> {
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
