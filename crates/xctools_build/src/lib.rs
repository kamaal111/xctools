use std::process::Command;

use anyhow::Context;
use xcbuild_common::Configuration;

/// Builds an Xcode project or workspace using the `xcodebuild` command-line tool.
///
/// This function constructs and executes an `xcodebuild` command with the specified parameters
/// to build an iOS, macOS, watchOS, or tvOS application. It supports building from either
/// Xcode project files (.xcodeproj) or workspace files (.xcworkspace).
///
/// # Arguments
///
/// * `schema` - The Xcode scheme name to build (e.g., "MyApp", "MyAppTests")
/// * `destination` - The build destination specifying the target device or simulator:
///   - iOS Simulator: "iOS Simulator,name=iPhone 15 Pro"
///   - Generic iOS: "generic/platform=iOS"
///   - macOS: "platform=macOS"
/// * `configuration` - The build configuration to use (Debug or Release)
/// * `project` - Optional path to the Xcode project file (.xcodeproj). Either this or
///   `workspace` must be provided, but not both.
/// * `workspace` - Optional path to the Xcode workspace file (.xcworkspace). Either this or
///   `project` must be provided, but not both.
///
/// # Returns
///
/// Returns `Ok(String)` containing the stdout from the xcodebuild command on success,
/// or `Err` if the build fails or if neither project nor workspace is specified.
///
/// # Examples
///
/// ## Testing parameter validation - neither project nor workspace:
/// ```rust
/// use xctools_build::build;
/// use xcbuild_common::Configuration;
///
/// // This should fail because neither project nor workspace is specified
/// let result = build(
///     &"MyApp".to_string(),
///     &"iOS Simulator,name=iPhone 15 Pro".to_string(),
///     &Configuration::Debug,
///     &None,
///     &None,
/// );
/// assert!(result.is_err());
/// let error_msg = result.unwrap_err().to_string();
/// assert!(error_msg.contains("Failed to determine project or workspace"));
/// ```
///
/// ## Testing Configuration enum usage:
/// ```rust
/// use xcbuild_common::Configuration;
///
/// // Test Configuration enum values
/// assert_eq!(Configuration::Debug.command_string(), "Debug");
/// assert_eq!(Configuration::Release.command_string(), "Release");
/// assert_eq!(Configuration::Debug.to_string(), "debug");
/// assert_eq!(Configuration::Release.to_string(), "release");
/// ```
///
/// ## Testing with project parameter (will attempt to build):
/// ```rust,no_run
/// use xctools_build::build;
/// use xcbuild_common::Configuration;
///
/// // This example shows the function signature but doesn't run
/// // because it would try to execute xcodebuild with a non-existent project
/// let result = build(
///     &"MyApp".to_string(),
///     &"iOS Simulator,name=iPhone 15 Pro".to_string(),
///     &Configuration::Debug,
///     &Some("MyApp.xcodeproj".to_string()),
///     &None,
/// );
/// // In a real scenario with a valid project, this would either succeed or
/// // fail based on the actual build outcome
/// ```
///
/// ## Testing with workspace parameter (will attempt to build):
/// ```rust,no_run
/// use xctools_build::build;
/// use xcbuild_common::Configuration;
///
/// // This example shows the function signature but doesn't run
/// // because it would try to execute xcodebuild with a non-existent workspace
/// let result = build(
///     &"MyApp".to_string(),
///     &"generic/platform=iOS".to_string(),
///     &Configuration::Release,
///     &None,
///     &Some("MyApp.xcworkspace".to_string()),
/// );
/// // In a real scenario with a valid workspace, this would either succeed or
/// // fail based on the actual build outcome
/// ```
///
/// ## Using the xctools CLI:
/// ```bash
/// # Build with project file and Debug configuration (default)
/// xctools build --schema MyApp --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj
///
/// # Build with workspace file and Release configuration
/// xctools build --schema MyApp --destination "generic/platform=iOS" --workspace MyApp.xcworkspace --configuration release
///
/// # Build for macOS
/// xctools build --schema MyApp --destination "platform=macOS" --project MyApp.xcodeproj
/// ```
///
/// # Generated Command
///
/// The function generates an xcodebuild command in the format:
/// ```bash
/// xcodebuild -project MyApp.xcodeproj -scheme MyApp -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Debug build
/// ```
///
/// # Requirements
///
/// - Xcode must be installed and `xcodebuild` must be available in PATH
/// - The specified project/workspace file must exist
/// - The specified scheme must exist in the project/workspace
/// - The destination must be valid for the target platform
pub fn build(
    schema: &String,
    destination: &String,
    configuration: &Configuration,
    project: &Option<String>,
    workspace: &Option<String>,
) -> anyhow::Result<String> {
    let target = BuildTarget::new(project.as_ref(), workspace.as_ref());
    let project_or_workspace = match target.project_or_workspace_string() {
        Ok(project_or_workspace) => project_or_workspace,
        Err(_) => anyhow::bail!("Failed to determine project or workspace"),
    };
    let command = match build_command(schema, destination, configuration, &target) {
        Ok(command) => command,
        Err(_) => anyhow::bail!("Failed to build command"),
    };
    let output = match Command::new("zsh")
        .arg("-c")
        .arg(command)
        .spawn()
        .context(format!("Failed to build {}", project_or_workspace))?
        .wait_with_output()
        .context(format!("Failed to build {}", project_or_workspace))
    {
        Err(error) => return Err(error),
        Ok(output) => output,
    };

    String::from_utf8(output.stdout).context("Failed to decode output")
}

struct BuildTarget {
    project: Option<String>,
    workspace: Option<String>,
}

impl BuildTarget {
    fn new(project: Option<&String>, workspace: Option<&String>) -> Self {
        Self {
            project: project.cloned(),
            workspace: workspace.cloned(),
        }
    }

    fn project_or_workspace_string(&self) -> anyhow::Result<String> {
        if let Some(project) = &self.project {
            return Ok(project.clone());
        }

        if let Some(workspace) = &self.workspace {
            return Ok(workspace.clone());
        }

        anyhow::bail!("Neither project nor workspace is specified")
    }

    fn project_or_workspace_argument(&self) -> anyhow::Result<String> {
        if let Some(project) = &self.project {
            return Ok(format!("-project {}", project));
        }

        if let Some(workspace) = &self.workspace {
            return Ok(format!("-workspace {}", workspace));
        }

        anyhow::bail!("Neither project nor workspace is specified")
    }
}

fn build_command(
    schema: &String,
    destination: &String,
    configuration: &Configuration,
    target: &BuildTarget,
) -> anyhow::Result<String> {
    let project_or_workspace_argument = target.project_or_workspace_argument()?;
    let configuration_string = configuration.command_string();

    let command = format!(
        "xcodebuild {} -scheme {} -destination '{}' -configuration {} build",
        project_or_workspace_argument, schema, destination, configuration_string
    );

    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_target_with_project() {
        let target = BuildTarget::new(Some(&"TestProject.xcodeproj".to_string()), None);

        assert_eq!(
            target.project_or_workspace_string().unwrap(),
            "TestProject.xcodeproj"
        );
        assert_eq!(
            target.project_or_workspace_argument().unwrap(),
            "-project TestProject.xcodeproj"
        );
    }

    #[test]
    fn test_build_target_with_workspace() {
        let target = BuildTarget::new(None, Some(&"TestWorkspace.xcworkspace".to_string()));

        assert_eq!(
            target.project_or_workspace_string().unwrap(),
            "TestWorkspace.xcworkspace"
        );
        assert_eq!(
            target.project_or_workspace_argument().unwrap(),
            "-workspace TestWorkspace.xcworkspace"
        );
    }

    #[test]
    fn test_build_target_with_neither() {
        let target = BuildTarget::new(None, None);

        assert!(target.project_or_workspace_string().is_err());
        assert!(target.project_or_workspace_argument().is_err());
    }

    #[test]
    fn test_configuration_command_string() {
        assert_eq!(Configuration::Debug.command_string(), "Debug");
        assert_eq!(Configuration::Release.command_string(), "Release");
    }

    #[test]
    fn test_configuration_display() {
        assert_eq!(Configuration::Debug.to_string(), "debug");
        assert_eq!(Configuration::Release.to_string(), "release");
    }

    #[test]
    fn test_build_command() {
        let target = BuildTarget::new(Some(&"TestProject.xcodeproj".to_string()), None);
        let command = build_command(
            &"TestScheme".to_string(),
            &"iOS Simulator,name=iPhone 15 Pro".to_string(),
            &Configuration::Debug,
            &target,
        )
        .unwrap();

        assert_eq!(
            command,
            "xcodebuild -project TestProject.xcodeproj -scheme TestScheme -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Debug build"
        );
    }
}
