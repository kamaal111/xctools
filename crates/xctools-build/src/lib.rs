use std::process::Command;

use anyhow::Context;
use clap::ValueEnum;

#[derive(ValueEnum, Clone, Debug)]
pub enum Configuration {
    Debug,
    Release,
}

impl Configuration {
    pub fn command_string(&self) -> String {
        match self {
            Configuration::Debug => String::from("Debug"),
            Configuration::Release => String::from("Release"),
        }
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::Debug
    }
}

impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Configuration::Debug => write!(f, "debug"),
            Configuration::Release => write!(f, "release"),
        }
    }
}

pub struct BuildTarget {
    project: Option<String>,
    workspace: Option<String>,
}

pub fn build(
    schema: &String,
    destination: &String,
    configuration: &Configuration,
    project: &Option<String>,
    workspace: &Option<String>,
) -> anyhow::Result<String> {
    let target = BuildTarget::new(project.clone(), workspace.clone());
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
        .with_context(|| format!("Failed to build {}", project_or_workspace))?
        .wait_with_output()
        .with_context(|| format!("Failed to build {}", project_or_workspace))
    {
        Err(error) => return Err(error),
        Ok(output) => output,
    };

    String::from_utf8(output.stdout).with_context(|| "Failed to decode output")
}

impl BuildTarget {
    fn new(project: Option<String>, workspace: Option<String>) -> Self {
        Self { project, workspace }
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
        let target = BuildTarget::new(Some("TestProject.xcodeproj".to_string()), None);

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
        let target = BuildTarget::new(None, Some("TestWorkspace.xcworkspace".to_string()));

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
        let target = BuildTarget::new(Some("TestProject.xcodeproj".to_string()), None);
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
