use anyhow::{Context, Result, anyhow};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    ffi::OsStr,
    path::PathBuf,
    process::{Command, Stdio},
    str::SplitWhitespace,
};

/// Generates acknowledgements file for Swift Package Manager dependencies and git contributors.
///
/// This function scans the Xcode project's DerivedData to find Swift Package Manager dependencies,
/// extracts package information (name, license, author, URL), analyzes git commit history to
/// identify project contributors, and outputs a structured JSON acknowledgements file.
///
/// # Arguments
///
/// * `app_name` - The name of the Xcode app/project to generate acknowledgements for. This is used
///   to locate the correct DerivedData folder containing package information.
/// * `output` - The output path for the acknowledgements file. Can be either:
///   - A specific file path (e.g., "./Credits.json")
///   - A directory path (will create "acknowledgements.json" in that directory)
///
/// # Returns
///
/// Returns `Ok(String)` with a success message indicating where the acknowledgements file was written,
/// or `Err` if the operation fails (e.g., DerivedData not found, file write permissions, etc.).
///
/// # Examples
///
/// ## Using as a library:
/// ```rust
/// use xctools_acknowledgements::acknowledgements;
///
/// // Generate acknowledgements for "MyApp" project
/// let result = acknowledgements(&"MyApp".to_string(), &"./acknowledgements.json".to_string());
/// match result {
///     Ok(message) => println!("{}", message),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
///
/// ## Using the xctools CLI:
/// ```bash
/// # Generate acknowledgements to a specific file
/// xctools acknowledgements --app-name MyApp --output ./acknowledgements.json
///
/// # Generate acknowledgements to a directory (creates acknowledgements.json)
/// xctools acknowledgements --app-name MyApp --output ./output-directory/
///
/// # Generate acknowledgements for a specific app with custom name
/// xctools acknowledgements --app-name "My iOS App" --output ./Credits.json
/// ```
///
/// # Output Format
///
/// The generated JSON file contains:
/// - `packages`: Array of Swift Package Manager dependencies with name, license, author, and URL
/// - `contributors`: Array of git contributors with name and contribution count (emails removed)
///
/// # Requirements
///
/// - The app must have been built at least once to generate DerivedData
/// - Git repository must exist for contributor analysis
/// - Write permissions for the output location
pub fn acknowledgements(app_name: &String, output: &String) -> Result<String> {
    let packages = get_packages_acknowledgements(&app_name)?;
    let contributors = get_contributors_list();
    let acknowledgements = Acknowledgements::new(&packages, &contributors);
    let final_output_path = make_final_output_path(output);
    write_acknowledgements(&acknowledgements, &final_output_path)?;
    let stdout = format!(
        "✅ Acknowledgements written to: {}",
        final_output_path.display()
    );

    Ok(stdout)
}

#[derive(Debug, Serialize)]
struct Acknowledgements {
    packages: Vec<PackageAcknowledgement>,
    contributors: Vec<Contributor>,
}

impl Acknowledgements {
    fn new(
        packages: &Vec<PackageAcknowledgement>,
        contributors: &Vec<Contributor>,
    ) -> Acknowledgements {
        Acknowledgements {
            packages: packages.clone(),
            contributors: contributors.clone(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
struct Contributor {
    name: String,
    email: Option<String>,
    contributions: i64,
}

impl Contributor {
    fn new(name: &String, email: Option<&String>, contributions: &i64) -> Contributor {
        Contributor {
            name: name.clone(),
            email: email.cloned(),
            contributions: *contributions,
        }
    }

    fn without_email(&self) -> Contributor {
        Contributor::new(&self.name, None, &self.contributions)
    }

    fn first_name(&self) -> Option<&str> {
        self.name_parts().nth(0)
    }

    fn has_only_a_single_name(&self) -> bool {
        self.collected_name_parts().len() == 1
    }

    fn name_parts(&self) -> SplitWhitespace<'_> {
        self.name.split_whitespace()
    }

    fn collected_name_parts(&self) -> Vec<&str> {
        self.name_parts().collect()
    }
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

#[derive(Debug, Serialize, Clone)]
struct PackageAcknowledgement {
    name: String,
    license: Option<String>,
    author: String,
    url: String,
}

impl PackageAcknowledgement {
    fn new(
        name: &String,
        license: Option<&String>,
        author: &String,
        url: &String,
    ) -> PackageAcknowledgement {
        PackageAcknowledgement {
            name: name.clone(),
            license: license.cloned(),
            author: author.clone(),
            url: url.clone(),
        }
    }
}

fn make_final_output_path(output: &String) -> PathBuf {
    let output_path = PathBuf::from(output);
    let final_output_path = if output_path.is_dir() {
        output_path.join("acknowledgements.json")
    } else {
        output_path
    };

    final_output_path
}

fn write_acknowledgements(
    acknowledgements: &Acknowledgements,
    output_path: &PathBuf,
) -> Result<()> {
    let json_content = serde_json::to_string_pretty(&acknowledgements)
        .context("Failed to serialize acknowledgements to JSON")?;
    std::fs::write(&output_path, &json_content).context(format!(
        "Failed to write acknowledgements to file: {}",
        output_path.display()
    ))?;

    Ok(())
}

fn get_contributors_list() -> Vec<Contributor> {
    let output = match run_zsh_command(&"git --no-pager log \"--pretty=format:%an <%ae>\"") {
        None => return Vec::new(),
        Some(output) => output,
    };
    let contributor_names_mapped_by_emails = output
        .lines()
        .filter_map(|line| {
            let email = extract_email_out_of_contributors_line(line)?;
            let name = extract_name_out_of_contributors_line(line)?;

            Some((email, patch_contributor_name(&name)))
        })
        .fold(
            HashMap::<String, Vec<String>>::new(),
            |mut acc, (email, name)| {
                acc.entry(email).or_insert_with(Vec::new).push(name);
                acc
            },
        );
    let aggregated_contributors = contributor_names_mapped_by_emails.iter().fold(
        Vec::<Contributor>::new(),
        |mut acc, (email, names)| {
            let longest_name = names.iter().fold(String::from(""), |longest_name, name| {
                if longest_name.len() >= name.len() {
                    return longest_name;
                }

                name.clone()
            });
            let contributor = Contributor::new(&longest_name, Some(&email), &(names.len() as i64));
            acc.push(contributor);

            acc
        },
    );
    let mut contributors = merge_contributors_with_similar_names(&aggregated_contributors);
    contributors.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    contributors.iter().map(|c| c.without_email()).collect()
}

fn merge_contributors_with_similar_names(contributors: &Vec<Contributor>) -> Vec<Contributor> {
    contributors.iter().fold(
        Vec::<Contributor>::new(),
        |mut merged_contributors, contributor| {
            let contributor_first_name = match contributor.first_name() {
                None => return merged_contributors,
                Some(contributor_first_name) => contributor_first_name,
            };
            if contributor_first_name.is_empty() {
                return merged_contributors;
            }

            let first_names: HashSet<_> = merged_contributors
                .iter()
                .filter_map(|c| c.first_name())
                .collect();
            if !first_names.contains(contributor_first_name) {
                merged_contributors.push(contributor.clone());

                return merged_contributors;
            }

            let merged_contributor_to_update =
                merged_contributors.iter().enumerate().find(|(_i, c)| {
                    let first_name_is_the_same =
                        contributor_first_name == c.first_name().unwrap_or("");
                    let name_is_the_same = contributor.name == c.name;
                    let one_of_authors_has_just_a_single_name =
                        (contributor.has_only_a_single_name() || c.has_only_a_single_name())
                            && contributor.collected_name_parts().len()
                                != c.collected_name_parts().len();

                    first_name_is_the_same
                        && (name_is_the_same || one_of_authors_has_just_a_single_name)
                });
            if let Some(merged_contributor_to_update) = merged_contributor_to_update {
                let (i, c) = merged_contributor_to_update;
                let longest_author_name = if contributor.name.len() > c.name.len() {
                    &contributor.name
                } else {
                    &c.name
                };
                let merged_contributor = Contributor::new(
                    longest_author_name,
                    contributor.email.as_ref(),
                    &(contributor.contributions + c.contributions),
                );
                merged_contributors[i] = merged_contributor;
            }

            merged_contributors
        },
    )
}

fn patch_contributor_name(name: &String) -> String {
    // TODO: Make this extendable somehow!
    if name == "kamaal111" || name == "Kamaal" {
        String::from("Kamaal Farah")
    } else {
        name.clone()
    }
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
            let license = package_licenses.get(name);
            let url_parts: Vec<_> = url.split("/").collect();
            let author = url_parts[url_parts.len() - 2].to_string();
            let entry = PackageAcknowledgement::new(name, license, &author, url);
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
                .context(
                    "Could not find any DerivedData for project, make sure to build at least once",
                )
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
        .context("Failed to load home directory")?
        .join("Library/Developer/Xcode/DerivedData");

    Ok(result)
}

fn run_zsh_command<S>(command: &S) -> Option<String>
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
        run_zsh_command(&"defaults read com.apple.dt.Xcode IDECustomDerivedDataLocation")?;
    let trimmed_stdout_string = stdout_string.trim();
    if trimmed_stdout_string.is_empty() {
        return None;
    }

    Some(PathBuf::from(trimmed_stdout_string))
}

#[cfg(test)]
mod tests {}
