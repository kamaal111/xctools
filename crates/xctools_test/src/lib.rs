use anyhow::Result;
use xcbuild_common::{BuildTarget, Configuration};

pub fn test(
    schema: &String,
    destination: &String,
    configuration: &Configuration,
    project: &Option<String>,
    workspace: &Option<String>,
) -> Result<String> {
    let target = BuildTarget::new(project.as_ref(), workspace.as_ref());
    let project_or_workspace = target.project_or_workspace_string()?;

    Ok("".to_string())
}
