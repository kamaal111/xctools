use std::process::Command;

use anyhow::{Context, Result};
use clap::ValueEnum;

pub fn run_xcodebuild_command(
    action: &XcodebuildCommandAction,
    schema: &String,
    destination: &String,
    configuration: &Configuration,
    target: &BuildTarget,
) -> Result<String> {
    let project_or_workspace = target.project_or_workspace_string()?;
    let command = make_xcodebuild_command(action, schema, destination, configuration, &target)?;
    let output = Command::new("zsh")
        .arg("-c")
        .arg(command)
        .spawn()
        .context(format!("Failed to run {}", project_or_workspace))?
        .wait_with_output()
        .context(format!("Failed to run {}", project_or_workspace))?;

    String::from_utf8(output.stdout).context("Failed to decode output")
}

fn make_xcodebuild_command(
    action: &XcodebuildCommandAction,
    schema: &String,
    destination: &String,
    configuration: &Configuration,
    target: &BuildTarget,
) -> anyhow::Result<String> {
    let project_or_workspace_argument = target.project_or_workspace_argument()?;
    let configuration_string = configuration.command_string();

    let command = format!(
        "xcodebuild {} {} -scheme {} -destination '{}' -configuration {}",
        action.command_string(),
        project_or_workspace_argument,
        schema,
        destination,
        configuration_string
    );

    Ok(command)
}

pub enum XcodebuildCommandAction {
    Build,
    Test,
}

impl XcodebuildCommandAction {
    pub fn command_string(&self) -> String {
        match self {
            XcodebuildCommandAction::Build => String::from("build"),
            XcodebuildCommandAction::Test => String::from("test"),
        }
    }
}

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

#[derive(ValueEnum, Clone, Debug)]
pub enum SDK {
    IPhoneOS,
    MacOSX,
}

impl SDK {
    pub fn command_string(&self) -> String {
        match self {
            SDK::IPhoneOS => String::from("iphoneos"),
            SDK::MacOSX => String::from("macosx"),
        }
    }
}

impl std::fmt::Display for SDK {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SDK::IPhoneOS => write!(f, "iphoneos"),
            SDK::MacOSX => write!(f, "macosx"),
        }
    }
}

pub struct BuildTarget {
    project: Option<String>,
    workspace: Option<String>,
}

impl BuildTarget {
    pub fn new(project: Option<&String>, workspace: Option<&String>) -> Self {
        Self {
            project: project.cloned(),
            workspace: workspace.cloned(),
        }
    }

    pub fn project_or_workspace_string(&self) -> Result<String> {
        if let Some(project) = &self.project {
            return Ok(project.clone());
        }

        if let Some(workspace) = &self.workspace {
            return Ok(workspace.clone());
        }

        anyhow::bail!("Neither project nor workspace is specified")
    }

    pub fn project_or_workspace_argument(&self) -> Result<String> {
        if let Some(project) = &self.project {
            return Ok(format!("-project {}", project));
        }

        if let Some(workspace) = &self.workspace {
            return Ok(format!("-workspace {}", workspace));
        }

        anyhow::bail!("Neither project nor workspace is specified")
    }
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
        let command = make_xcodebuild_command(
            &XcodebuildCommandAction::Build,
            &"TestScheme".to_string(),
            &"iOS Simulator,name=iPhone 15 Pro".to_string(),
            &Configuration::Debug,
            &target,
        )
        .unwrap();

        assert_eq!(
            command,
            "xcodebuild build -project TestProject.xcodeproj -scheme TestScheme -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Debug"
        );
    }
}
