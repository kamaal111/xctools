use std::process::Command;

use anyhow::{Context, Result};
use clap::ValueEnum;

/// Data Transfer Object for xcodebuild command parameters
#[derive(Debug)]
pub struct XcodebuildParams {
    pub action: XcodebuildCommandAction,
    pub scheme: Option<String>,
    pub destination: Option<String>,
    pub configuration: Option<Configuration>,
    pub target: Option<BuildTarget>,
    pub sdk: Option<SDK>,
    pub archive_path: Option<String>,
    pub export_path: Option<String>,
    pub export_options: Option<String>,
}

impl XcodebuildParams {
    pub fn new(action: XcodebuildCommandAction) -> Self {
        Self {
            action,
            scheme: None,
            destination: None,
            configuration: None,
            target: None,
            sdk: None,
            archive_path: None,
            export_path: None,
            export_options: None,
        }
    }

    fn make_xcodebuild_command(&self) -> anyhow::Result<String> {
        let mut command = format!("xcodebuild {}", self.action.command_string());
        if let Some(target) = &self.target {
            command += &format!(" {}", target.project_or_workspace_argument()?)
        }
        if let Some(scheme) = &self.scheme {
            command += &format!(" -scheme {}", scheme);
        }
        if let Some(destination) = &self.destination {
            command += &format!(" -destination '{}'", destination);
        }
        if let Some(configuration) = &self.configuration {
            command += &format!(" -configuration {}", configuration.command_string());
        }
        if let Some(archive_path) = &self.archive_path {
            command += &format!(" -archivePath {}", archive_path);
        }
        if let Some(sdk) = &self.sdk {
            command += &format!(" -sdk {}", sdk.command_string());
        }
        if let Some(export_path) = &self.export_path {
            command += &format!(" -exportPath {}", export_path);
        }
        if let Some(export_options) = &self.export_options {
            command += &format!(" -exportOptionsPlist {}", export_options);
        }

        Ok(command)
    }

    pub fn with_export_options(mut self, export_options: String) -> Self {
        self.export_options = Some(export_options);
        self
    }

    pub fn with_export_path(mut self, export_path: String) -> Self {
        self.export_path = Some(export_path);
        self
    }

    pub fn with_scheme(mut self, scheme: String) -> Self {
        self.scheme = Some(scheme);
        self
    }

    pub fn with_destination(mut self, destination: String) -> Self {
        self.destination = Some(destination);
        self
    }

    pub fn with_configuration(mut self, configuration: Configuration) -> Self {
        self.configuration = Some(configuration);
        self
    }

    pub fn with_target(mut self, target: BuildTarget) -> Self {
        self.target = Some(target);
        self
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
    let command = params.make_xcodebuild_command()?;
    let output = Command::new("zsh")
        .arg("-c")
        .arg(command)
        .spawn()
        .context(format!("Failed to run {}", params.action.command_string()))?
        .wait_with_output()
        .context(format!("Failed to run {}", params.action.command_string()))?;

    String::from_utf8(output.stdout).context("Failed to decode output")
}

#[derive(Debug, PartialEq)]
pub enum XcodebuildCommandAction {
    Build,
    Test,
    Archive,
    ExportArchive,
}

impl XcodebuildCommandAction {
    pub fn command_string(&self) -> String {
        match self {
            XcodebuildCommandAction::Build => String::from("build"),
            XcodebuildCommandAction::Test => String::from("test"),
            XcodebuildCommandAction::Archive => String::from("archive"),
            XcodebuildCommandAction::ExportArchive => String::from("-exportArchive"),
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
        let params = &XcodebuildParams::new(XcodebuildCommandAction::Build)
            .with_scheme("TestScheme".to_string())
            .with_destination("iOS Simulator,name=iPhone 15 Pro".to_string())
            .with_configuration(Configuration::Debug)
            .with_target(target);
        let command = params.make_xcodebuild_command().unwrap();

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
        let params = XcodebuildParams::new(XcodebuildCommandAction::Test)
            .with_scheme("TestScheme".to_string())
            .with_destination("iOS Simulator,name=iPhone 15 Pro".to_string())
            .with_configuration(Configuration::Release)
            .with_target(target);
        let command = params.make_xcodebuild_command().unwrap();

        assert_eq!(
            command,
            "xcodebuild test -project TestProject.xcodeproj -scheme TestScheme -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Release"
        );
    }

    #[test]
    fn test_archive_command() {
        let target = BuildTarget::new(Some(&"TestProject.xcodeproj".to_string()), None);
        let params = XcodebuildParams::new(XcodebuildCommandAction::Archive)
            .with_scheme("TestScheme".to_string())
            .with_destination("iOS Simulator,name=iPhone 15 Pro".to_string())
            .with_configuration(Configuration::Release)
            .with_target(target)
            .with_archive_path("/path/to/archive.xcarchive".to_string());
        let command = params.make_xcodebuild_command().unwrap();

        assert_eq!(
            command,
            "xcodebuild archive -project TestProject.xcodeproj -scheme TestScheme -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Release -archivePath /path/to/archive.xcarchive"
        );
    }

    #[test]
    fn test_command_with_sdk() {
        let target = BuildTarget::new(Some(&"TestProject.xcodeproj".to_string()), None);
        let params = XcodebuildParams::new(XcodebuildCommandAction::Build)
            .with_scheme("TestScheme".to_string())
            .with_destination("iOS Simulator,name=iPhone 15 Pro".to_string())
            .with_configuration(Configuration::Debug)
            .with_target(target)
            .with_sdk(SDK::Iphoneos);
        let command = params.make_xcodebuild_command().unwrap();

        assert_eq!(
            command,
            "xcodebuild build -project TestProject.xcodeproj -scheme TestScheme -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Debug -sdk iphoneos"
        );
    }

    #[test]
    fn test_command_with_workspace() {
        let target = BuildTarget::new(None, Some(&"TestWorkspace.xcworkspace".to_string()));
        let params = XcodebuildParams::new(XcodebuildCommandAction::Build)
            .with_scheme("TestScheme".to_string())
            .with_destination("iOS Simulator,name=iPhone 15 Pro".to_string())
            .with_configuration(Configuration::Debug)
            .with_target(target);
        let command = params.make_xcodebuild_command().unwrap();

        assert_eq!(
            command,
            "xcodebuild build -workspace TestWorkspace.xcworkspace -scheme TestScheme -destination 'iOS Simulator,name=iPhone 15 Pro' -configuration Debug"
        );
    }

    #[test]
    fn test_command_with_all_options() {
        let target = BuildTarget::new(None, Some(&"TestWorkspace.xcworkspace".to_string()));
        let params = XcodebuildParams::new(XcodebuildCommandAction::Archive)
            .with_scheme("TestScheme".to_string())
            .with_destination("Generic/iOS".to_string())
            .with_configuration(Configuration::Release)
            .with_target(target)
            .with_sdk(SDK::Macosx)
            .with_archive_path("/tmp/MyApp.xcarchive".to_string());
        let command = params.make_xcodebuild_command().unwrap();

        assert_eq!(
            command,
            "xcodebuild archive -workspace TestWorkspace.xcworkspace -scheme TestScheme -destination 'Generic/iOS' -configuration Release -archivePath /tmp/MyApp.xcarchive -sdk macosx"
        );
    }

    #[test]
    fn test_command_error_with_invalid_target() {
        let target = BuildTarget::new(None, None);
        let params = XcodebuildParams::new(XcodebuildCommandAction::Build)
            .with_scheme("TestScheme".to_string())
            .with_destination("iOS Simulator,name=iPhone 15 Pro".to_string())
            .with_configuration(Configuration::Debug)
            .with_target(target);
        let result = params.make_xcodebuild_command();

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

    #[test]
    fn test_export_archive_command_with_export_path() {
        let params = XcodebuildParams::new(XcodebuildCommandAction::ExportArchive)
            .with_archive_path("/path/to/archive.xcarchive".to_string())
            .with_export_path("/path/to/export".to_string());
        let command = params.make_xcodebuild_command().unwrap();

        assert_eq!(
            command,
            "xcodebuild -exportArchive -archivePath /path/to/archive.xcarchive -exportPath /path/to/export"
        );
    }

    #[test]
    fn test_export_archive_command_with_export_options() {
        let params = XcodebuildParams::new(XcodebuildCommandAction::ExportArchive)
            .with_archive_path("/path/to/archive.xcarchive".to_string())
            .with_export_options("/path/to/ExportOptions.plist".to_string());
        let command = params.make_xcodebuild_command().unwrap();

        assert_eq!(
            command,
            "xcodebuild -exportArchive -archivePath /path/to/archive.xcarchive -exportOptionsPlist /path/to/ExportOptions.plist"
        );
    }

    #[test]
    fn test_export_archive_command_with_all_export_options() {
        let params = XcodebuildParams::new(XcodebuildCommandAction::ExportArchive)
            .with_archive_path("/path/to/archive.xcarchive".to_string())
            .with_export_path("/path/to/export".to_string())
            .with_export_options("/path/to/ExportOptions.plist".to_string());
        let command = params.make_xcodebuild_command().unwrap();

        assert_eq!(
            command,
            "xcodebuild -exportArchive -archivePath /path/to/archive.xcarchive -exportPath /path/to/export -exportOptionsPlist /path/to/ExportOptions.plist"
        );
    }

    #[test]
    fn test_export_archive_action_string() {
        assert_eq!(
            XcodebuildCommandAction::ExportArchive.command_string(),
            "-exportArchive"
        );
    }
}
