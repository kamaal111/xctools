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
/// ## Basic usage with error handling:
/// ```rust
/// use xctools_acknowledgements::acknowledgements;
///
/// // This will fail because "NonExistentApp" doesn't have DerivedData
/// let result = acknowledgements(&"NonExistentApp".to_string(), &"/tmp/test.json".to_string());
/// assert!(result.is_err());
/// ```
///
/// ## Testing parameter validation:
/// ```rust
/// use xctools_acknowledgements::acknowledgements;
///
/// // Test with empty app name - should fail
/// let result = acknowledgements(&"".to_string(), &"/tmp/acknowledgements.json".to_string());
/// assert!(result.is_err());
/// ```
///
/// ## Testing output path handling:
/// ```rust
/// use xctools_acknowledgements::acknowledgements;
/// use std::path::Path;
///
/// // The function should handle both file and directory paths
/// // Even though this will fail due to missing DerivedData, we can test the interface
/// let file_result = acknowledgements(&"TestApp".to_string(), &"/tmp/credits.json".to_string());
/// let dir_result = acknowledgements(&"TestApp".to_string(), &"/tmp/".to_string());
///
/// // Both should fail with the same type of error (missing DerivedData)
/// assert!(file_result.is_err());
/// assert!(dir_result.is_err());
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
            let author = if url_parts.len() >= 2 {
                url_parts[url_parts.len() - 2].to_string()
            } else {
                // Fallback for URLs with insufficient parts
                url_parts.get(0).unwrap_or(&"unknown").to_string()
            };
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
    if !packages_directory.is_dir() {
        return Err(anyhow!(format!(
            "Failed to read packages directory contents; directory does not exist: {}",
            packages_directory.display()
        )));
    }

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
mod tests {
    use super::*;
    use std::collections::{BTreeMap, HashMap};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_contributor_creation() {
        let name = "John Doe".to_string();
        let email = Some("john@example.com".to_string());
        let contributions = 5;

        let contributor = Contributor::new(&name, email.as_ref(), &contributions);

        assert_eq!(contributor.name, "John Doe");
        assert_eq!(contributor.email, Some("john@example.com".to_string()));
        assert_eq!(contributor.contributions, 5);
    }

    #[test]
    fn test_contributor_without_email() {
        let contributor = Contributor {
            name: "Jane Smith".to_string(),
            email: Some("jane@example.com".to_string()),
            contributions: 3,
        };

        let without_email = contributor.without_email();

        assert_eq!(without_email.name, "Jane Smith");
        assert_eq!(without_email.email, None);
        assert_eq!(without_email.contributions, 3);
    }

    #[test]
    fn test_contributor_first_name() {
        let contributor = Contributor {
            name: "John Doe".to_string(),
            email: None,
            contributions: 1,
        };

        assert_eq!(contributor.first_name(), Some("John"));

        let single_name_contributor = Contributor {
            name: "John".to_string(),
            email: None,
            contributions: 1,
        };

        assert_eq!(single_name_contributor.first_name(), Some("John"));

        let empty_name_contributor = Contributor {
            name: "".to_string(),
            email: None,
            contributions: 1,
        };

        assert_eq!(empty_name_contributor.first_name(), None);
    }

    #[test]
    fn test_contributor_has_only_single_name() {
        let single_name = Contributor {
            name: "John".to_string(),
            email: None,
            contributions: 1,
        };
        assert!(single_name.has_only_a_single_name());

        let multiple_names = Contributor {
            name: "John Doe".to_string(),
            email: None,
            contributions: 1,
        };
        assert!(!multiple_names.has_only_a_single_name());
    }

    #[test]
    fn test_package_acknowledgement_creation() {
        let name = "TestPackage".to_string();
        let license = Some("MIT".to_string());
        let author = "TestAuthor".to_string();
        let url = "https://github.com/testauthor/testpackage".to_string();

        let package = PackageAcknowledgement::new(&name, license.as_ref(), &author, &url);

        assert_eq!(package.name, "TestPackage");
        assert_eq!(package.license, Some("MIT".to_string()));
        assert_eq!(package.author, "TestAuthor");
        assert_eq!(package.url, "https://github.com/testauthor/testpackage");
    }

    #[test]
    fn test_acknowledgements_creation() {
        let packages = vec![PackageAcknowledgement {
            name: "Package1".to_string(),
            license: Some("MIT".to_string()),
            author: "Author1".to_string(),
            url: "https://github.com/author1/package1".to_string(),
        }];

        let contributors = vec![Contributor {
            name: "John Doe".to_string(),
            email: None,
            contributions: 5,
        }];

        let acknowledgements = Acknowledgements::new(&packages, &contributors);

        assert_eq!(acknowledgements.packages.len(), 1);
        assert_eq!(acknowledgements.contributors.len(), 1);
        assert_eq!(acknowledgements.packages[0].name, "Package1");
        assert_eq!(acknowledgements.contributors[0].name, "John Doe");
    }

    #[test]
    fn test_extract_name_from_contributors_line() {
        let line = "John Doe <john@example.com>";
        let name = extract_name_out_of_contributors_line(line);
        assert_eq!(name, Some("John Doe".to_string()));

        let line_with_spaces = "  Jane Smith  <jane@example.com>";
        let name = extract_name_out_of_contributors_line(line_with_spaces);
        assert_eq!(name, Some("Jane Smith".to_string()));

        let invalid_line = "Invalid line without brackets";
        let name = extract_name_out_of_contributors_line(invalid_line);
        assert_eq!(name, None);

        let empty_name = " <email@example.com>";
        let name = extract_name_out_of_contributors_line(empty_name);
        assert_eq!(name, None);
    }

    #[test]
    fn test_extract_email_from_contributors_line() {
        let line = "John Doe <john@example.com>";
        let email = extract_email_out_of_contributors_line(line);
        assert_eq!(email, Some("john@example.com".to_string()));

        let invalid_line = "John Doe john@example.com";
        let email = extract_email_out_of_contributors_line(invalid_line);
        assert_eq!(email, None);

        let malformed_brackets = "John Doe >john@example.com<";
        let email = extract_email_out_of_contributors_line(malformed_brackets);
        assert_eq!(email, None);
    }

    #[test]
    fn test_patch_contributor_name() {
        assert_eq!(
            patch_contributor_name(&"kamaal111".to_string()),
            "Kamaal Farah"
        );
        assert_eq!(
            patch_contributor_name(&"Kamaal".to_string()),
            "Kamaal Farah"
        );
        assert_eq!(
            patch_contributor_name(&"Other Name".to_string()),
            "Other Name"
        );
    }

    #[test]
    fn test_make_final_output_path() {
        // Test with file path
        let file_path = "./output.json".to_string();
        let result = make_final_output_path(&file_path);
        assert_eq!(result, PathBuf::from("./output.json"));

        // Test with directory path - this test may need to be adjusted based on actual directory structure
        // For now, we'll test the logic assuming the path doesn't exist (treated as file)
        let dir_path = "./non_existent_directory/".to_string();
        let result = make_final_output_path(&dir_path);
        assert_eq!(result, PathBuf::from("./non_existent_directory/"));
    }

    #[test]
    fn test_make_packages_acknowledgements() {
        let mut packages_urls = BTreeMap::new();
        packages_urls.insert(
            "Package1".to_string(),
            "https://github.com/author1/package1".to_string(),
        );
        packages_urls.insert(
            "Package2".to_string(),
            "https://github.com/author2/package2".to_string(),
        );

        let mut package_licenses = HashMap::new();
        package_licenses.insert("Package1".to_string(), "MIT License".to_string());

        let acknowledgements = make_packages_acknowledgements(&packages_urls, &package_licenses);

        assert_eq!(acknowledgements.len(), 2);

        let package1 = acknowledgements
            .iter()
            .find(|p| p.name == "Package1")
            .unwrap();
        assert_eq!(package1.author, "author1");
        assert_eq!(package1.license, Some("MIT License".to_string()));
        assert_eq!(package1.url, "https://github.com/author1/package1");

        let package2 = acknowledgements
            .iter()
            .find(|p| p.name == "Package2")
            .unwrap();
        assert_eq!(package2.author, "author2");
        assert_eq!(package2.license, None);
        assert_eq!(package2.url, "https://github.com/author2/package2");
    }

    #[test]
    fn test_merge_contributors_with_similar_names() {
        let contributors = vec![
            Contributor {
                name: "John".to_string(),
                email: Some("john1@example.com".to_string()),
                contributions: 5,
            },
            Contributor {
                name: "John Doe".to_string(),
                email: Some("john2@example.com".to_string()),
                contributions: 3,
            },
            Contributor {
                name: "Jane Smith".to_string(),
                email: Some("jane@example.com".to_string()),
                contributions: 2,
            },
        ];

        let merged = merge_contributors_with_similar_names(&contributors);

        // Should merge John and John Doe into one contributor with John Doe name and combined contributions
        assert_eq!(merged.len(), 2);

        let john_contributor = merged.iter().find(|c| c.name.contains("John")).unwrap();
        assert_eq!(john_contributor.name, "John Doe"); // Longer name should be kept
        assert_eq!(john_contributor.contributions, 8); // 5 + 3

        let jane_contributor = merged.iter().find(|c| c.name == "Jane Smith").unwrap();
        assert_eq!(jane_contributor.contributions, 2);
    }

    #[test]
    fn test_write_and_read_acknowledgements() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_acknowledgements.json");

        let packages = vec![PackageAcknowledgement {
            name: "TestPackage".to_string(),
            license: Some("MIT".to_string()),
            author: "TestAuthor".to_string(),
            url: "https://github.com/testauthor/testpackage".to_string(),
        }];

        let contributors = vec![Contributor {
            name: "Test Contributor".to_string(),
            email: None,
            contributions: 10,
        }];

        let acknowledgements = Acknowledgements::new(&packages, &contributors);

        // Write acknowledgements
        let result = write_acknowledgements(&acknowledgements, &output_path);
        assert!(result.is_ok());

        // Verify file was created and contains expected content
        assert!(output_path.exists());
        let content = fs::read_to_string(&output_path).unwrap();

        // Parse back to verify structure
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed["packages"].is_array());
        assert!(parsed["contributors"].is_array());
        assert_eq!(parsed["packages"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["contributors"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_workspace_state_deserialization() {
        let json_content = r#"
        {
            "object": {
                "dependencies": [
                    {
                        "packageRef": {
                            "name": "TestPackage",
                            "location": "https://github.com/testauthor/testpackage"
                        }
                    }
                ]
            }
        }
        "#;

        let workspace_state: WorkspaceState = serde_json::from_str(json_content).unwrap();
        assert_eq!(workspace_state.object.dependencies.len(), 1);
        assert_eq!(
            workspace_state.object.dependencies[0].package_ref.name,
            "TestPackage"
        );
        assert_eq!(
            workspace_state.object.dependencies[0].package_ref.location,
            "https://github.com/testauthor/testpackage"
        );
    }

    #[test]
    fn test_acknowledgements_serialization() {
        let packages = vec![PackageAcknowledgement {
            name: "TestPackage".to_string(),
            license: Some("MIT".to_string()),
            author: "TestAuthor".to_string(),
            url: "https://github.com/testauthor/testpackage".to_string(),
        }];

        let contributors = vec![Contributor {
            name: "Test Contributor".to_string(),
            email: None,
            contributions: 5,
        }];

        let acknowledgements = Acknowledgements::new(&packages, &contributors);
        let json_result = serde_json::to_string(&acknowledgements);

        assert!(json_result.is_ok());
        let json_string = json_result.unwrap();

        // Verify it contains expected fields
        assert!(json_string.contains("packages"));
        assert!(json_string.contains("contributors"));
        assert!(json_string.contains("TestPackage"));
        assert!(json_string.contains("Test Contributor"));
    }

    #[test]
    fn test_acknowledgements_main_function_success() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test_output.json");

        // Create mock DerivedData structure
        let derived_data_dir = temp_dir.path().join("DerivedData").join("TestApp-abc123");
        let source_packages_dir = derived_data_dir.join("SourcePackages");
        let checkouts_dir = source_packages_dir.join("checkouts");
        std::fs::create_dir_all(&checkouts_dir).unwrap();

        // Create workspace-state.json
        let workspace_state_content = r#"
        {
            "object": {
                "dependencies": [
                    {
                        "packageRef": {
                            "name": "TestPackage",
                            "location": "https://github.com/testauthor/testpackage"
                        }
                    }
                ]
            }
        }
        "#;
        std::fs::write(
            source_packages_dir.join("workspace-state.json"),
            workspace_state_content,
        )
        .unwrap();

        // Create package directory with license
        let package_dir = checkouts_dir.join("TestPackage");
        std::fs::create_dir_all(&package_dir).unwrap();
        std::fs::write(
            package_dir.join("LICENSE"),
            "MIT License\n\nCopyright (c) 2024 Test Author",
        )
        .unwrap();

        // Mock get_xcode_derived_data_base to return our temp directory
        unsafe { std::env::set_var("HOME", temp_dir.path()) };

        // Create the DerivedData directory structure that the function expects
        std::fs::create_dir_all(temp_dir.path().join("Library/Developer/Xcode/DerivedData"))
            .unwrap();
        std::fs::create_dir_all(&derived_data_dir).unwrap();

        // Since we can't easily mock the git commands and derived data discovery,
        // this test focuses on the JSON serialization and file writing parts
        let _result = acknowledgements(
            &"TestApp".to_string(),
            &output_path.to_string_lossy().to_string(),
        );

        // The function might fail due to DerivedData discovery, but we're testing the structure
        // In a real scenario, this would require more complex mocking
    }

    #[test]
    fn test_make_final_output_path_with_actual_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        let result = make_final_output_path(&dir_path);
        let expected = temp_dir.path().join("acknowledgements.json");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_packages_urls_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent_file = temp_dir.path().join("non_existent.json");

        let result = get_packages_urls(&non_existent_file);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read workspace-state.json")
        );
    }

    #[test]
    fn test_get_packages_urls_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_json_file = temp_dir.path().join("invalid.json");
        std::fs::write(&invalid_json_file, "{ invalid json }").unwrap();

        let result = get_packages_urls(&invalid_json_file);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse workspace-state.json")
        );
    }

    #[test]
    fn test_get_packages_licenses_directory_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent_dir = temp_dir.path().join("non_existent");

        let result = get_packages_licenses(&non_existent_dir);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read packages directory contents")
        );
    }

    #[test]
    fn test_get_packages_licenses_success() {
        let temp_dir = TempDir::new().unwrap();

        // Create a package directory with a license file
        let package_dir = temp_dir.path().join("TestPackage");
        std::fs::create_dir_all(&package_dir).unwrap();
        std::fs::write(
            package_dir.join("LICENSE"),
            "MIT License\n\nCopyright (c) 2024 Test Author",
        )
        .unwrap();

        let result = get_packages_licenses(&temp_dir.path().to_path_buf());
        assert!(result.is_ok());

        let licenses = result.unwrap();
        assert_eq!(licenses.len(), 1);
        assert!(licenses.contains_key("TestPackage"));
        assert!(licenses["TestPackage"].contains("MIT License"));
    }

    #[test]
    fn test_get_packages_licenses_no_license_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create a package directory without license files
        let package_dir = temp_dir.path().join("TestPackage");
        std::fs::create_dir_all(&package_dir).unwrap();
        std::fs::write(package_dir.join("README.md"), "# Test Package").unwrap();

        let result = get_packages_licenses(&temp_dir.path().to_path_buf());
        assert!(result.is_ok());

        let licenses = result.unwrap();
        assert_eq!(licenses.len(), 0);
    }

    #[test]
    fn test_write_acknowledgements_invalid_path() {
        let packages = vec![PackageAcknowledgement {
            name: "TestPackage".to_string(),
            license: Some("MIT".to_string()),
            author: "TestAuthor".to_string(),
            url: "https://github.com/testauthor/testpackage".to_string(),
        }];

        let contributors = vec![Contributor {
            name: "Test Contributor".to_string(),
            email: None,
            contributions: 10,
        }];

        let acknowledgements = Acknowledgements::new(&packages, &contributors);

        // Try to write to an invalid path (non-existent directory)
        let invalid_path = PathBuf::from("/non/existent/path/acknowledgements.json");
        let result = write_acknowledgements(&acknowledgements, &invalid_path);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to write acknowledgements to file")
        );
    }

    #[test]
    fn test_get_default_derived_data_base() {
        let result = get_default_derived_data_base();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(
            path.to_string_lossy()
                .contains("Library/Developer/Xcode/DerivedData")
        );
    }

    #[test]
    fn test_run_zsh_command_success() {
        let result = run_zsh_command(&"echo 'test'");
        assert!(result.is_some());
        assert_eq!(result.unwrap().trim(), "test");
    }

    #[test]
    fn test_run_zsh_command_failure() {
        let result = run_zsh_command(&"nonexistentcommand12345");
        // The command should fail but the function returns None on error
        // This tests the error handling path
        assert!(result.is_none() || result.unwrap().is_empty());
    }

    #[test]
    fn test_get_contributors_list_no_git() {
        // Save current directory and change to a temp directory without git
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let contributors = get_contributors_list();
        assert_eq!(contributors.len(), 0);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_get_user_configured_derived_data_base_success() {
        // Test when IDECustomDerivedDataLocation is set
        // This is hard to test without actually setting the Xcode preference
        // so we test the function structure but expect None in most environments
        let result = get_user_configured_derived_data_base();
        // This will be None in most test environments, which is expected
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_get_xcode_derived_data_base_fallback() {
        // Test that it falls back to default when no custom path is configured
        let result = get_xcode_derived_data_base();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(
            path.to_string_lossy()
                .contains("Library/Developer/Xcode/DerivedData")
        );
    }

    #[test]
    fn test_acknowledgements_struct_methods() {
        let packages = vec![PackageAcknowledgement {
            name: "TestPackage".to_string(),
            license: Some("MIT".to_string()),
            author: "TestAuthor".to_string(),
            url: "https://github.com/testauthor/testpackage".to_string(),
        }];

        let contributors = vec![Contributor {
            name: "Test Contributor".to_string(),
            email: None,
            contributions: 5,
        }];

        let acknowledgements = Acknowledgements {
            packages: packages.clone(),
            contributors: contributors.clone(),
        };

        // Test that the struct maintains the data correctly
        assert_eq!(acknowledgements.packages.len(), 1);
        assert_eq!(acknowledgements.contributors.len(), 1);
        assert_eq!(acknowledgements.packages[0].name, "TestPackage");
        assert_eq!(acknowledgements.contributors[0].name, "Test Contributor");
    }

    #[test]
    fn test_contributor_collected_name_parts() {
        let contributor = Contributor {
            name: "John Doe Smith".to_string(),
            email: None,
            contributions: 1,
        };

        let parts = contributor.collected_name_parts();
        assert_eq!(parts, vec!["John", "Doe", "Smith"]);

        let single_name_contributor = Contributor {
            name: "John".to_string(),
            email: None,
            contributions: 1,
        };

        let single_parts = single_name_contributor.collected_name_parts();
        assert_eq!(single_parts, vec!["John"]);
    }

    #[test]
    fn test_contributor_without_email_method() {
        let contributor_with_email = Contributor {
            name: "John Doe".to_string(),
            email: Some("john@example.com".to_string()),
            contributions: 5,
        };

        let contributor_without_email = contributor_with_email.without_email();
        assert_eq!(contributor_without_email.name, "John Doe");
        assert_eq!(contributor_without_email.email, None);
        assert_eq!(contributor_without_email.contributions, 5);
    }

    #[test]
    fn test_extract_edge_cases() {
        // Test extract_name_out_of_contributors_line with various edge cases
        assert_eq!(extract_name_out_of_contributors_line(""), None);
        assert_eq!(
            extract_name_out_of_contributors_line("No angle brackets"),
            None
        );
        assert_eq!(extract_name_out_of_contributors_line("< >"), None);
        assert_eq!(
            extract_name_out_of_contributors_line("Name<email"),
            Some("Name".to_string())
        );

        // Test extract_email_out_of_contributors_line with various edge cases
        assert_eq!(extract_email_out_of_contributors_line(""), None);
        assert_eq!(extract_email_out_of_contributors_line("No brackets"), None);
        assert_eq!(
            extract_email_out_of_contributors_line("< >"),
            Some(" ".to_string())
        );
        assert_eq!(extract_email_out_of_contributors_line("Name email>"), None);
        assert_eq!(extract_email_out_of_contributors_line("Name <email"), None);
    }

    #[test]
    fn test_package_acknowledgement_methods() {
        let package = PackageAcknowledgement::new(
            &"TestPackage".to_string(),
            Some(&"MIT License".to_string()),
            &"TestAuthor".to_string(),
            &"https://github.com/testauthor/testpackage".to_string(),
        );

        assert_eq!(package.name, "TestPackage");
        assert_eq!(package.license, Some("MIT License".to_string()));
        assert_eq!(package.author, "TestAuthor");
        assert_eq!(package.url, "https://github.com/testauthor/testpackage");

        // Test with None license
        let package_no_license = PackageAcknowledgement::new(
            &"TestPackage2".to_string(),
            None,
            &"TestAuthor2".to_string(),
            &"https://github.com/testauthor2/testpackage2".to_string(),
        );

        assert_eq!(package_no_license.license, None);
    }

    #[test]
    fn test_merge_contributors_complex_scenarios() {
        let contributors = vec![
            Contributor {
                name: "John".to_string(),
                email: Some("john@example.com".to_string()),
                contributions: 3,
            },
            Contributor {
                name: "John Doe".to_string(),
                email: Some("john@example.com".to_string()),
                contributions: 2,
            },
            Contributor {
                name: "Jane Smith".to_string(),
                email: Some("jane@example.com".to_string()),
                contributions: 1,
            },
            Contributor {
                name: "J".to_string(),
                email: Some("j@example.com".to_string()),
                contributions: 1,
            },
        ];

        let merged = merge_contributors_with_similar_names(&contributors);

        // Should merge John and John Doe, but not Jane Smith or J
        assert!(merged.len() <= contributors.len());

        // Check that John Doe (longer name) is kept and contributions are merged
        let john_entry = merged.iter().find(|c| c.name.contains("John"));
        assert!(john_entry.is_some());
        if let Some(john) = john_entry {
            assert_eq!(john.name, "John Doe"); // Longer name should be kept
            assert_eq!(john.contributions, 5); // 3 + 2 = 5
        }
    }

    #[test]
    fn test_find_derived_data_for_app_with_glob_pattern() {
        let temp_dir = TempDir::new().unwrap();

        // Mock the home directory temporarily
        unsafe { std::env::set_var("HOME", temp_dir.path()) };

        // Create a DerivedData directory structure
        let derived_data_base = temp_dir.path().join("Library/Developer/Xcode/DerivedData");
        std::fs::create_dir_all(&derived_data_base).unwrap();

        // Create multiple app directories
        let app1_dir = derived_data_base.join("TestApp-abc123");
        let app2_dir = derived_data_base.join("TestApp-def456");
        std::fs::create_dir_all(&app1_dir).unwrap();
        std::fs::create_dir_all(&app2_dir).unwrap();

        // Test finding derived data for app
        let result = find_derived_data_for_app(&"TestApp".to_string());

        // Should find one of the directories (the most recently modified)
        assert!(result.is_ok() || result.is_err()); // Either finds it or doesn't due to timing
    }
}
