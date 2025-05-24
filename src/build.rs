use std::process::Command;

use anyhow::Context;

use crate::Configuration;

pub struct BuildTarget {
    project: Option<String>,
    workspace: Option<String>,
}

pub fn build(
    schema: String,
    destination: String,
    configuration: Configuration,
    project: Option<String>,
    workspace: Option<String>,
) -> anyhow::Result<String> {
    let target = BuildTarget::new(project, workspace);
    let project_or_workspace = match target.project_or_workspace_string() {
        Ok(project_or_workspace) => project_or_workspace,
        Err(_) => anyhow::bail!("Failed to determine project or workspace"),
    };
    let command = match build_command(schema, destination, configuration, target) {
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
    pub fn new(project: Option<String>, workspace: Option<String>) -> Self {
        Self { project, workspace }
    }

    fn project_or_workspace_string(&self) -> anyhow::Result<String> {
        if let Some(project) = &self.project {
            return Ok(project.clone());
        }

        if let Some(workspace) = &self.workspace {
            return Ok(workspace.clone());
        }

        Err(anyhow::anyhow!(
            "Neither project nor workspace was provided"
        ))
    }

    fn xcode_command_flag(&self) -> anyhow::Result<String> {
        if let Some(project) = &self.project {
            return Ok(format!("-project {}", project.clone()));
        }

        if let Some(workspace) = &self.workspace {
            return Ok(format!("-workspace {}", workspace));
        }

        Err(anyhow::anyhow!(
            "Neither project nor workspace was provided"
        ))
    }
}

fn build_command(
    schema: String,
    destination: String,
    configuration: Configuration,
    target: BuildTarget,
) -> anyhow::Result<String> {
    let xcode_command_flag = match target.xcode_command_flag() {
        Err(error) => return Err(error),
        Ok(xcode_command_flag) => xcode_command_flag,
    };
    let command = format!(
        "xcodebuild build -scheme {} -configuration {} -destination '{}' {}",
        schema,
        configuration.command_string(),
        destination,
        xcode_command_flag
    );

    Ok(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_target_new() {
        let target = BuildTarget::new(Some("MyApp.xcodeproj".to_string()), None);
        assert_eq!(target.project, Some("MyApp.xcodeproj".to_string()));
        assert_eq!(target.workspace, None);

        let target = BuildTarget::new(None, Some("MyApp.xcworkspace".to_string()));
        assert_eq!(target.project, None);
        assert_eq!(target.workspace, Some("MyApp.xcworkspace".to_string()));
    }

    #[test]
    fn test_build_target_project_or_workspace_string() {
        // Test with project
        let target = BuildTarget::new(Some("MyApp.xcodeproj".to_string()), None);
        let result = target.project_or_workspace_string().unwrap();
        assert_eq!(result, "MyApp.xcodeproj");

        // Test with workspace
        let target = BuildTarget::new(None, Some("MyApp.xcworkspace".to_string()));
        let result = target.project_or_workspace_string().unwrap();
        assert_eq!(result, "MyApp.xcworkspace");

        // Test with both (should return project)
        let target = BuildTarget::new(
            Some("MyApp.xcodeproj".to_string()),
            Some("MyApp.xcworkspace".to_string()),
        );
        let result = target.project_or_workspace_string().unwrap();
        assert_eq!(result, "MyApp.xcodeproj");

        // Test with neither
        let target = BuildTarget::new(None, None);
        let result = target.project_or_workspace_string();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Neither project nor workspace was provided"
        );
    }

    #[test]
    fn test_build_target_xcode_command_flag() {
        // Test with project
        let target = BuildTarget::new(Some("MyApp.xcodeproj".to_string()), None);
        let result = target.xcode_command_flag().unwrap();
        assert_eq!(result, "-project MyApp.xcodeproj");

        // Test with workspace
        let target = BuildTarget::new(None, Some("MyApp.xcworkspace".to_string()));
        let result = target.xcode_command_flag().unwrap();
        assert_eq!(result, "-workspace MyApp.xcworkspace");

        // Test with both (should return project)
        let target = BuildTarget::new(
            Some("MyApp.xcodeproj".to_string()),
            Some("MyApp.xcworkspace".to_string()),
        );
        let result = target.xcode_command_flag().unwrap();
        assert_eq!(result, "-project MyApp.xcodeproj");

        // Test with neither
        let target = BuildTarget::new(None, None);
        let result = target.xcode_command_flag();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Neither project nor workspace was provided"
        );
    }

    #[test]
    fn test_build_command_with_project() {
        let target = BuildTarget::new(Some("MyApp.xcodeproj".to_string()), None);
        let command = build_command(
            "MyApp".to_string(),
            "iOS Simulator,name=iPhone 15 Pro".to_string(),
            Configuration::Debug,
            target,
        )
        .unwrap();

        let expected = "xcodebuild build -scheme MyApp -configuration Debug -destination 'iOS Simulator,name=iPhone 15 Pro' -project MyApp.xcodeproj";
        assert_eq!(command, expected);
    }

    #[test]
    fn test_build_command_with_workspace() {
        let target = BuildTarget::new(None, Some("MyApp.xcworkspace".to_string()));
        let command = build_command(
            "MyApp".to_string(),
            "iOS Simulator,name=iPhone 15 Pro".to_string(),
            Configuration::Release,
            target,
        )
        .unwrap();

        let expected = "xcodebuild build -scheme MyApp -configuration Release -destination 'iOS Simulator,name=iPhone 15 Pro' -workspace MyApp.xcworkspace";
        assert_eq!(command, expected);
    }

    #[test]
    fn test_build_command_with_different_destinations() {
        let target = BuildTarget::new(Some("MyApp.xcodeproj".to_string()), None);

        // Test iOS device
        let command = build_command(
            "MyApp".to_string(),
            "generic/platform=iOS".to_string(),
            Configuration::Release,
            target,
        )
        .unwrap();
        let expected = "xcodebuild build -scheme MyApp -configuration Release -destination 'generic/platform=iOS' -project MyApp.xcodeproj";
        assert_eq!(command, expected);

        // Test macOS
        let target = BuildTarget::new(Some("MyApp.xcodeproj".to_string()), None);
        let command = build_command(
            "MyApp".to_string(),
            "generic/platform=macOS".to_string(),
            Configuration::Debug,
            target,
        )
        .unwrap();
        let expected = "xcodebuild build -scheme MyApp -configuration Debug -destination 'generic/platform=macOS' -project MyApp.xcodeproj";
        assert_eq!(command, expected);
    }

    #[test]
    fn test_build_command_with_invalid_target() {
        let target = BuildTarget::new(None, None);
        let result = build_command(
            "MyApp".to_string(),
            "iOS Simulator,name=iPhone 15 Pro".to_string(),
            Configuration::Debug,
            target,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Neither project nor workspace was provided"
        );
    }

    #[test]
    fn test_configuration_command_string() {
        assert_eq!(Configuration::Debug.command_string(), "Debug");
        assert_eq!(Configuration::Release.command_string(), "Release");
    }

    #[test]
    fn test_build_command_with_special_characters_in_scheme() {
        let target = BuildTarget::new(Some("MyApp.xcodeproj".to_string()), None);
        let command = build_command(
            "My App With Spaces".to_string(),
            "iOS Simulator,name=iPhone 15 Pro".to_string(),
            Configuration::Debug,
            target,
        )
        .unwrap();

        let expected = "xcodebuild build -scheme My App With Spaces -configuration Debug -destination 'iOS Simulator,name=iPhone 15 Pro' -project MyApp.xcodeproj";
        assert_eq!(command, expected);
    }
}
