use anyhow::Result;
use clap::ValueEnum;

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
