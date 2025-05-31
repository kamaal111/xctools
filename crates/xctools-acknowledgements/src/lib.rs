use anyhow::{Context, Result, anyhow};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsStr,
    path::PathBuf,
    process::{Command, Stdio},
};

pub fn acknowledgements(app_name: &String, output: &String) -> Result<String> {
    let packages_acknowledgements = get_packages_acknowledgements(&app_name);
    let contributors_list = get_contributors_list();

    Ok(String::from(""))
}

#[derive(Debug, Deserialize)]
struct WorkspaceState {
    object: WorkspaceObject,
}

#[derive(Debug, Deserialize)]
struct WorkspaceObject {
    dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize)]
struct Dependency {
    #[serde(rename = "packageRef")]
    package_ref: PackageRef,
}

#[derive(Debug, Deserialize)]
struct PackageRef {
    name: String,
    location: String,
}

#[derive(Debug, Serialize)]
struct PackageAcknowledgement {
    name: String,
    license: Option<String>,
    author: String,
    url: String,
}

impl PackageAcknowledgement {
    fn new(
        name: String,
        license: Option<String>,
        author: String,
        url: String,
    ) -> PackageAcknowledgement {
        PackageAcknowledgement {
            name,
            license,
            author,
            url,
        }
    }
}

fn get_contributors_list() -> Option<String> {
    let output = match run_zsh_command("git --no-pager log \"--pretty=format:%an <%ae>\"") {
        None => return None,
        Some(output) => output,
    };
    let contributor_names_mapped_by_emails = output
        .lines()
        .filter_map(|line| {
            let email = extract_email_out_of_contributors_line(line)?;
            let name = extract_name_out_of_contributors_line(line)?;
            Some((email, name))
        })
        .fold(
            HashMap::<String, Vec<String>>::new(),
            |mut acc, (email, name)| {
                acc.entry(email).or_insert_with(Vec::new).push(name);
                acc
            },
        );
    println!("🐸🐸🐸 {:?}", contributor_names_mapped_by_emails);

    Some(String::from(""))
}

/// Extract name from format "Name <email@domain.com>"
fn extract_name_out_of_contributors_line(line: &str) -> Option<String> {
    let end = match line.find('<') {
        None => return None,
        Some(end) => end,
    };

    let name = line[..end].trim().to_string();
    if name.is_empty() {
        return None;
    }

    Some(name)
}

/// Extract email from format "Name <email@domain.com>"
fn extract_email_out_of_contributors_line(line: &str) -> Option<String> {
    let start = match line.find('<') {
        None => return None,
        Some(start) => start,
    };
    let end = match line.find('>') {
        None => return None,
        Some(end) => end,
    };
    if end <= start {
        return None;
    }

    let email = line[start + 1..end].to_string();

    Some(email)
}

fn get_packages_acknowledgements(app_name: &String) -> Result<Vec<PackageAcknowledgement>> {
    let packages_directory = find_derived_data_for_app(app_name)?.join("SourcePackages");
    let packages_licenses = get_packages_licenses(&packages_directory.join("checkouts"))?;
    let packages_urls = get_packages_urls(&packages_directory.join("workspace-state.json"))?;
    let packages_acknowledgements =
        make_packages_acknowledgements(&packages_urls, &packages_licenses);

    Ok(packages_acknowledgements)
}

fn make_packages_acknowledgements(
    packages_urls: &BTreeMap<String, String>,
    package_licenses: &HashMap<String, String>,
) -> Vec<PackageAcknowledgement> {
    packages_urls
        .iter()
        .fold(Vec::new(), |mut acc, (name, url)| {
            let name = name.clone();
            let url = url.clone();
            let license = package_licenses.get(&name).cloned();
            let url_parts: Vec<_> = url.split("/").collect();
            let author = url_parts[url_parts.len() - 2].to_string();
            let entry = PackageAcknowledgement::new(name, license, author, url);
            acc.push(entry);

            acc
        })
}

fn get_packages_urls(workspace_state_url: &PathBuf) -> Result<BTreeMap<String, String>> {
    let workspace_state_json_content =
        std::fs::read_to_string(workspace_state_url).map_err(|e| {
            anyhow!(format!(
                "Failed to read workspace-state.json; error='{}'",
                e
            ))
        })?;
    let workspace_state: WorkspaceState = serde_json::from_str(&workspace_state_json_content)
        .map_err(|e| {
            anyhow!(format!(
                "Failed to parse workspace-state.json; error='{}'",
                e
            ))
        })?;

    let packages =
        workspace_state
            .object
            .dependencies
            .iter()
            .fold(BTreeMap::new(), |mut acc, dep| {
                acc.insert(
                    dep.package_ref.name.clone(),
                    dep.package_ref.location.clone(),
                );
                acc
            });

    Ok(packages)
}

fn get_packages_licenses(packages_directory: &PathBuf) -> Result<HashMap<String, String>> {
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
    let mut licenses: HashMap<String, String> = HashMap::new();
    for content in packages_directory_contents {
        let filename = match content.file_name().to_str() {
            None => continue,
            Some(filename) => filename.to_owned(),
        };
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
        licenses.insert(filename, license);
    }

    Ok(licenses)
}

fn find_derived_data_for_app(app_name: &String) -> Result<PathBuf> {
    let xcode_derived_data_base_display = get_xcode_derived_data_base()?;
    let glob_pattern = format!(
        "{}-*",
        xcode_derived_data_base_display.join(app_name).display()
    );

    glob(&glob_pattern)
        .map_err(|e| anyhow!(format!("Failed to search through derived data; error={}", e)))
        .and_then(|matches| {
            let mut paths: Vec<_> = matches.filter_map(|p| p.ok()).filter(|p| p.is_dir()).collect();
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

fn run_zsh_command<S>(command: S) -> Option<String>
where
    S: AsRef<OsStr>,
{
    let child = match Command::new("zsh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
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
    let stdout_string = match String::from_utf8(output.stdout) {
        Err(_error) => return None,
        Ok(stdout_string) => stdout_string,
    };

    Some(stdout_string)
}

fn get_user_configured_derived_data_base() -> Option<PathBuf> {
    let stdout_string =
        run_zsh_command("defaults read com.apple.dt.Xcode IDECustomDerivedDataLocation")?;
    let trimmed_stdout_string = stdout_string.trim();
    if trimmed_stdout_string.is_empty() {
        return None;
    }

    Some(PathBuf::from(trimmed_stdout_string))
}

#[cfg(test)]
mod tests {}
