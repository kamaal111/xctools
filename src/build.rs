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
        "xcodebuild build -scheme {} -configuration {} -destination {} {}",
        schema,
        configuration.command_string(),
        destination,
        xcode_command_flag
    );

    Ok(command)
}
