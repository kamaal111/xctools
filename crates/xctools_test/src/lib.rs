use anyhow::Result;
use xcbuild_common::{BuildTarget, Configuration, XcodebuildCommandAction, run_xcodebuild_command};

pub fn test(
    schema: &String,
    destination: &String,
    configuration: &Configuration,
    project: &Option<String>,
    workspace: &Option<String>,
) -> Result<String> {
    let target = BuildTarget::new(project.as_ref(), workspace.as_ref());
    let output = run_xcodebuild_command(
        &XcodebuildCommandAction::Test,
        schema,
        destination,
        configuration,
        &target,
    )?;

    Ok(output)
}

#[cfg(test)]
mod tests {}
