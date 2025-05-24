use std::process::Command;

use anyhow::Context;
use clap::{ArgGroup, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build Xcode project.
    #[command(group(
        ArgGroup::new("target")
            .required(true)
            .args(["project", "workspace"]),
    ))]
    Build {
        /// The Xcode scheme to build.
        #[arg(short, long)]
        schema: String,

        /// The build destination (e.g., "iOS Simulator,name=iPhone 15 Pro").
        #[arg(short, long)]
        destination: String,

        /// Configuration - "Debug" or "Release"
        #[arg(short, long, default_value_t = Configuration::default())]
        configuration: Configuration,

        /// Xcode project file (.xcodeproj)
        #[arg(short, long)]
        project: Option<String>,

        /// Xcode workspace file (.xcworkspace)
        #[arg(short, long)]
        workspace: Option<String>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum Configuration {
    Debug,
    Release,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Build {
            schema,
            destination,
            configuration,
            project,
            workspace,
        } => {
            if let Err(e) = build(schema, destination, configuration, project, workspace) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

impl Configuration {
    fn command_string(&self) -> String {
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

struct BuildTarget {
    project: Option<String>,
    workspace: Option<String>,
}

impl BuildTarget {
    fn project_or_workspace_string(&self) -> anyhow::Result<String> {
        match &self.project {
            Some(some_project) => Ok(some_project.clone()),
            None => self
                .workspace
                .clone()
                // Invariant, should actually have been pre validated by clap
                .ok_or_else(|| anyhow::anyhow!("Neither project nor workspace was provided")),
        }
    }

    fn xcode_command_flag(&self) -> anyhow::Result<String> {
        match &self.project {
            Some(some_project) => Ok(format!("-project {}", some_project.clone())),
            None => match self.workspace.clone() {
                None => Err(anyhow::anyhow!(
                    "Neither project nor workspace was provided"
                )),
                Some(some_workspace) => Ok(format!("-workspace {}", some_workspace)),
            },
        }
    }
}

fn build(
    schema: String,
    destination: String,
    configuration: Configuration,
    project: Option<String>,
    workspace: Option<String>,
) -> anyhow::Result<()> {
    let target = BuildTarget { project, workspace };
    let project_or_workspace = match target.project_or_workspace_string() {
        Ok(path) => path,
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
    println!("{:?}", String::from_utf8(output.stderr));
    Ok(())
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
        "xcodebuild build -scheme {} -configuration {} -destination \"{}\" {}",
        schema,
        configuration.command_string(),
        destination,
        xcode_command_flag
    );

    Ok(command)
}
