use std::process::Command;

use anyhow::{Context, Result};
use clap::ValueEnum;

pub fn run_xcodebuild_command(
    action: &XcodebuildCommandAction,
    schema: &String,
    destination: &String,
    configuration: &Configuration,
    target: &BuildTarget,
    sdk: Option<&SDK>,
    archive_path: Option<&String>,
) -> Result<String> {
    let project_or_workspace = target.project_or_workspace_string()?;
    let command = make_xcodebuild_command(
        action,
        schema,
        destination,
        configuration,
        &target,
        sdk,
        archive_path,
    )?;
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
    sdk: Option<&SDK>,
    archive_path: Option<&String>,
) -> anyhow::Result<String> {
    let project_or_workspace_argument = target.project_or_workspace_argument()?;
    let configuration_string = configuration.command_string();
    let mut command = format!(
        "xcodebuild {} {} -scheme {} -destination '{}' -configuration {}",
        action.command_string(),
        project_or_workspace_argument,
        schema,
        destination,
        configuration_string
    );
    if let Some(archive_path) = archive_path {
        command += &format!(" -archivePath {}", archive_path);
    }
    if let Some(sdk) = sdk {
        command += &format!(" -sdk {}", sdk.command_string());
    }

    Ok(command)
}

#[derive(Debug, PartialEq)]
pub enum XcodebuildCommandAction {
    Build,
    Test,
    Archive,
}

impl XcodebuildCommandAction {
    pub fn command_string(&self) -> String {
        match self {
            XcodebuildCommandAction::Build => String::from("build"),
            XcodebuildCommandAction::Test => String::from("test"),
            XcodebuildCommandAction::Archive => String::from("archive"),
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
    Iphoneos,
    Macosx,
}

impl SDK {
    pub fn command_string(&self) -> String {
        match self {
            SDK::Iphoneos => String::from("iphoneos"),
            SDK::Macosx => String::from("macosx"),
        }
    }
}

impl std::fmt::Display for SDK {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SDK::Iphoneos => write!(f, "iphoneos"),
            SDK::Macosx => write!(f, "macosx"),
        }
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum UploadTarget {
    Ios,
    Macos,
}

impl UploadTarget {
    pub fn command_string(&self) -> String {
        match self {
            UploadTarget::Ios => String::from("ios"),
            UploadTarget::Macos => String::from("macos"),
        }
    }
}

impl std::fmt::Display for UploadTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploadTarget::Ios => write!(f, "ios"),
            UploadTarget::Macos => write!(f, "macos"),
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
            None,
            None,
        )
        .unwrap();

        assert_eq!(
            command,
            "xcodebuild build -project TestProject.xcodeproj -scheme TestScheme -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Debug"
        );
    }

    #[test]
    fn test_xcodebuild_command_action_strings() {
        assert_eq!(XcodebuildCommandAction::Build.command_string(), "build");
        assert_eq!(XcodebuildCommandAction::Test.command_string(), "test");
        assert_eq!(XcodebuildCommandAction::Archive.command_string(), "archive");
    }

    #[test]
    fn test_sdk_command_string() {
        assert_eq!(SDK::Iphoneos.command_string(), "iphoneos");
        assert_eq!(SDK::Macosx.command_string(), "macosx");
    }

    #[test]
    fn test_sdk_display() {
        assert_eq!(SDK::Iphoneos.to_string(), "iphoneos");
        assert_eq!(SDK::Macosx.to_string(), "macosx");
    }

    #[test]
    fn test_configuration_default() {
        let default_config: Configuration = Default::default();
        assert_eq!(default_config.command_string(), "Debug");
        assert_eq!(default_config.to_string(), "debug");
    }

    #[test]
    fn test_test_command() {
        let target = BuildTarget::new(Some(&"TestProject.xcodeproj".to_string()), None);
        let command = make_xcodebuild_command(
            &XcodebuildCommandAction::Test,
            &"TestScheme".to_string(),
            &"iOS Simulator,name=iPhone 15 Pro".to_string(),
            &Configuration::Release,
            &target,
            None,
            None,
        )
        .unwrap();

        assert_eq!(
            command,
            "xcodebuild test -project TestProject.xcodeproj -scheme TestScheme -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Release"
        );
    }

    #[test]
    fn test_archive_command() {
        let target = BuildTarget::new(Some(&"TestProject.xcodeproj".to_string()), None);
        let command = make_xcodebuild_command(
            &XcodebuildCommandAction::Archive,
            &"TestScheme".to_string(),
            &"iOS Simulator,name=iPhone 15 Pro".to_string(),
            &Configuration::Release,
            &target,
            None,
            Some(&"/path/to/archive.xcarchive".to_string()),
        )
        .unwrap();

        assert_eq!(
            command,
            "xcodebuild archive -project TestProject.xcodeproj -scheme TestScheme -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Release -archivePath /path/to/archive.xcarchive"
        );
    }

    #[test]
    fn test_command_with_sdk() {
        let target = BuildTarget::new(Some(&"TestProject.xcodeproj".to_string()), None);
        let command = make_xcodebuild_command(
            &XcodebuildCommandAction::Build,
            &"TestScheme".to_string(),
            &"iOS Simulator,name=iPhone 15 Pro".to_string(),
            &Configuration::Debug,
            &target,
            Some(&SDK::Iphoneos),
            None,
        )
        .unwrap();

        assert_eq!(
            command,
            "xcodebuild build -project TestProject.xcodeproj -scheme TestScheme -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Debug -sdk iphoneos"
        );
    }

    #[test]
    fn test_command_with_workspace() {
        let target = BuildTarget::new(None, Some(&"TestWorkspace.xcworkspace".to_string()));
        let command = make_xcodebuild_command(
            &XcodebuildCommandAction::Build,
            &"TestScheme".to_string(),
            &"iOS Simulator,name=iPhone 15 Pro".to_string(),
            &Configuration::Debug,
            &target,
            None,
            None,
        )
        .unwrap();

        assert_eq!(
            command,
            "xcodebuild build -workspace TestWorkspace.xcworkspace -scheme TestScheme -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Debug"
        );
    }

    #[test]
    fn test_command_with_all_options() {
        let target = BuildTarget::new(None, Some(&"TestWorkspace.xcworkspace".to_string()));
        let command = make_xcodebuild_command(
            &XcodebuildCommandAction::Archive,
            &"TestScheme".to_string(),
            &"Generic/iOS".to_string(),
            &Configuration::Release,
            &target,
            Some(&SDK::Macosx),
            Some(&"/tmp/MyApp.xcarchive".to_string()),
        )
        .unwrap();

        assert_eq!(
            command,
            "xcodebuild archive -workspace TestWorkspace.xcworkspace -scheme TestScheme -destination 'Generic/iOS' -configuration Release -archivePath /tmp/MyApp.xcarchive -sdk macosx"
        );
    }

    #[test]
    fn test_command_error_with_invalid_target() {
        let target = BuildTarget::new(None, None);
        let result = make_xcodebuild_command(
            &XcodebuildCommandAction::Build,
            &"TestScheme".to_string(),
            &"iOS Simulator,name=iPhone 15 Pro".to_string(),
            &Configuration::Debug,
            &target,
            None,
            None,
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Neither project nor workspace is specified")
        );
    }

    #[test]
    fn test_build_target_with_both_project_and_workspace() {
        // Edge case: if both are provided, project takes precedence
        let target = BuildTarget::new(
            Some(&"TestProject.xcodeproj".to_string()),
            Some(&"TestWorkspace.xcworkspace".to_string()),
        );

        assert_eq!(
            target.project_or_workspace_string().unwrap(),
            "TestProject.xcodeproj"
        );
        assert_eq!(
            target.project_or_workspace_argument().unwrap(),
            "-project TestProject.xcodeproj"
        );
    }
}
