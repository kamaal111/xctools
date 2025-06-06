use anyhow::Result;
use xcbuild_common::{
    BuildTarget, Configuration, SDK, XcodebuildCommandAction, run_xcodebuild_command,
};

pub fn archive(
    schema: &String,
    destination: &String,
    configuration: &Configuration,
    sdk: &SDK,
    output: &String,
    project: &Option<String>,
    workspace: &Option<String>,
) -> Result<String> {
    let target = BuildTarget::new(project.as_ref(), workspace.as_ref());
    let output = run_xcodebuild_command(
        &XcodebuildCommandAction::Archive,
        schema,
        destination,
        configuration,
        &target,
        Some(sdk),
        Some(output),
    )?;

    Ok(output)
}
