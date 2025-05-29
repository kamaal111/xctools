use anyhow::{Context, Result, anyhow};
use glob::glob;
use std::{
    collections::HashMap,
    ffi::OsString,
    path::PathBuf,
    process::{Command, Stdio},
};

pub fn acknowledgements(app_name: &String, output: &String) -> Result<String> {
    let packages_directory = get_packages_directory(app_name)?;
    let packages_licenses = get_packages_licenses(&packages_directory);
    println!("{:?}", packages_licenses);
    Ok(String::from(""))
}

fn get_packages_licenses(packages_directory: &PathBuf) -> Result<HashMap<OsString, String>> {
    assert!(packages_directory.is_dir());

    let packages_directory_contents = std::fs::read_dir(packages_directory)
        .map_err(|e| {
            anyhow!(format!(
                "Failed to read packages directory contents; error='{}'",
                e
            ))
        })?
        .filter_map(|p| p.ok())
        .filter(|e| e.path().is_dir());
    let mut licenses: HashMap<OsString, String> = HashMap::new();
    for content in packages_directory_contents {
        let package_licenses: Vec<_> = std::fs::read_dir(content.path())
            .map_err(|e| {
                anyhow!(format!(
                    "Failed to read packages directory contents; error='{}'",
                    e
                ))
            })?
            .filter_map(|d| d.ok())
            .map(|d| d.path())
            .filter(|p| {
                if !p.is_file() {
                    return false;
                }

                p.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name_str| name_str.to_lowercase().contains("license"))
                    .unwrap_or(false)
            })
            .collect();
        let license_path = match package_licenses.first() {
            None => continue,
            Some(entry) => entry,
        };
        let license = std::fs::read_to_string(license_path)
            .map_err(|e| anyhow!(format!("Failed to read license; error='{}'", e)))?;
        licenses.insert(content.file_name(), license);
    }

    Ok(licenses)
}

fn get_packages_directory(app_name: &String) -> Result<PathBuf> {
    let result = find_derived_data_for_app(app_name)?
        .join("SourcePackages/checkouts")
        .clone();

    Ok(result)
}

fn find_derived_data_for_app(app_name: &String) -> Result<PathBuf> {
    let xcode_derived_data_base_display = get_xcode_derived_data_base()?;
    let glob_pattern = format!(
        "{}-*",
        xcode_derived_data_base_display.join(app_name).display()
    );

    glob(&glob_pattern)
        .map_err(|e| {
            anyhow!(format!(
                "Failed to search through derived data; error={}",
                e
            ))
        })
        .and_then(|matches| {
            let mut paths: Vec<_> = matches
                .filter_map(|p| p.ok())
                .filter(|p| p.is_dir())
                .collect();
            // Sort by last modified, the first in the list being latest updated
            paths.sort_by(|a, b| {
                let a_modified = std::fs::metadata(a)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let b_modified = std::fs::metadata(b)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                b_modified.cmp(&a_modified)
            });

            paths
                .first()
                .with_context(|| "Could not find any DerivedData for project, make sure to build at least once")
                .cloned()
        })
}

fn get_xcode_derived_data_base() -> Result<PathBuf> {
    if let Some(configured_derived_data_base) = get_user_configured_derived_data_base() {
        return Ok(configured_derived_data_base);
    }

    let default_derived_data_base = get_default_derived_data_base()?;

    Ok(default_derived_data_base)
}

fn get_default_derived_data_base() -> Result<PathBuf> {
    let result = std::env::home_dir()
        .with_context(|| "Failed to load home directory")?
        .join("Library/Developer/Xcode/DerivedData");

    Ok(result)
}

fn get_user_configured_derived_data_base() -> Option<PathBuf> {
    let child = match Command::new("zsh")
        .arg("-c")
        .arg("defaults read com.apple.dt.Xcode IDECustomDerivedDataLocation")
        .stderr(Stdio::null())
        .spawn()
    {
        Err(_error) => return None,
        Ok(child) => child,
    };
    let output = match child.wait_with_output() {
        Err(_error) => return None,
        Ok(output) => output,
    };
    let stdout_string = String::from_utf8(output.stdout).unwrap_or(String::from(""));
    let trimmed_stdout_string = stdout_string.trim();
    if trimmed_stdout_string.is_empty() {
        return None;
    }

    Some(PathBuf::from(trimmed_stdout_string))
}

#[cfg(test)]
mod tests {}
