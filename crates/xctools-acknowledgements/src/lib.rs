use anyhow::{Context, Result, anyhow};
use glob::glob;
use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
};

pub fn acknowledgements(app_name: &String, output: &String) -> Result<String> {
    let result = find_derived_data_for_app(app_name);
    println!("{:?}", result);
    Ok(String::from(""))
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
            let mut paths: Vec<_> = matches.filter_map(|p| p.ok()).collect();
            // Sort by last modified, the first in the list being latest updated
            paths.sort_by(|a, b| {
                let a_modified = fs::metadata(a)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let b_modified = fs::metadata(b)
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
    let result = env::home_dir()
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
