use std::process::Command;

use anyhow::{Context, Result};
use clap::ValueEnum;

/// Data Transfer Object for xcodebuild command parameters
#[derive(Debug)]
pub struct XcodebuildParams {
    pub action: XcodebuildCommandAction,
    pub schema: String,
    pub destination: String,
    pub configuration: Configuration,
    pub target: BuildTarget,
    pub sdk: Option<SDK>,
    pub archive_path: Option<String>,
}

impl XcodebuildParams {
    pub fn new(
        action: XcodebuildCommandAction,
        schema: String,
        destination: String,
        configuration: Configuration,
        target: BuildTarget,
    ) -> Self {
        Self {
            action,
            schema,
            destination,
            configuration,
            target,
            sdk: None,
            archive_path: None,
        }
    }

    pub fn with_sdk(mut self, sdk: SDK) -> Self {
        self.sdk = Some(sdk);
        self
    }

    pub fn with_archive_path(mut self, archive_path: String) -> Self {
        self.archive_path = Some(archive_path);
        self
    }
}

pub fn run_xcodebuild_command(params: &XcodebuildParams) -> Result<String> {
    let project_or_workspace = params.target.project_or_workspace_string()?;
    let command = make_xcodebuild_command(params)?;
    let output = Command::new("zsh")
        .arg("-c")
        .arg(command)
        .spawn()
        .context(format!("Failed to run {}", project_or_workspace))?
        .wait_with_output()
        .context(format!("Failed to run {}", project_or_workspace))?;

    String::from_utf8(output.stdout).context("Failed to decode output")
}

fn make_xcodebuild_command(params: &XcodebuildParams) -> anyhow::Result<String> {
    let project_or_workspace_argument = params.target.project_or_workspace_argument()?;
    let configuration_string = params.configuration.command_string();
    let mut command = format!(
        "xcodebuild {} {} -scheme {} -destination '{}' -configuration {}",
        params.action.command_string(),
        project_or_workspace_argument,
        params.schema,
        params.destination,
        configuration_string
    );
    if let Some(archive_path) = &params.archive_path {
        command += &format!(" -archivePath {}", archive_path);
    }
    if let Some(sdk) = &params.sdk {
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

#[derive(Debug)]
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
        let command = make_xcodebuild_command(&XcodebuildParams::new(
            XcodebuildCommandAction::Build,
            "TestScheme".to_string(),
            "iOS Simulator,name=iPhone 15 Pro".to_string(),
            Configuration::Debug,
            target,
        ))
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
        let command = make_xcodebuild_command(&XcodebuildParams::new(
            XcodebuildCommandAction::Test,
            "TestScheme".to_string(),
            "iOS Simulator,name=iPhone 15 Pro".to_string(),
            Configuration::Release,
            target,
        ))
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
            &XcodebuildParams::new(
                XcodebuildCommandAction::Archive,
                "TestScheme".to_string(),
                "iOS Simulator,name=iPhone 15 Pro".to_string(),
                Configuration::Release,
                target,
            )
            .with_archive_path("/path/to/archive.xcarchive".to_string()),
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
            &XcodebuildParams::new(
                XcodebuildCommandAction::Build,
                "TestScheme".to_string(),
                "iOS Simulator,name=iPhone 15 Pro".to_string(),
                Configuration::Debug,
                target,
            )
            .with_sdk(SDK::Iphoneos),
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
        let command = make_xcodebuild_command(&XcodebuildParams::new(
            XcodebuildCommandAction::Build,
            "TestScheme".to_string(),
            "iOS Simulator,name=iPhone 15 Pro".to_string(),
            Configuration::Debug,
            target,
        ))
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
            &XcodebuildParams::new(
                XcodebuildCommandAction::Archive,
                "TestScheme".to_string(),
                "Generic/iOS".to_string(),
                Configuration::Release,
                target,
            )
            .with_sdk(SDK::Macosx)
            .with_archive_path("/tmp/MyApp.xcarchive".to_string()),
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
        let result = make_xcodebuild_command(&XcodebuildParams::new(
            XcodebuildCommandAction::Build,
            "TestScheme".to_string(),
            "iOS Simulator,name=iPhone 15 Pro".to_string(),
            Configuration::Debug,
            target,
        ));

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
