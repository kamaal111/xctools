use anyhow::Result;
use xcbuild_common::{Configuration, SDK};

pub fn archive(
    schema: &String,
    destination: &String,
    configuration: &Configuration,
    sdk: &SDK,
    output: &String,
    project: &Option<String>,
    workspace: &Option<String>,
) -> Result<String> {
    Ok("Yes".to_string())
}
