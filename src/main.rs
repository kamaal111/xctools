mod build;
mod bump_version;

use crate::build::build;
use crate::bump_version::bump_version;
use clap::{ArgGroup, Parser, Subcommand, ValueEnum, builder::ValueParser};

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

    /// Bump version of Xcode project
    BumpVersion {
        /// Build number
        #[arg(short, long)]
        build_number: i32,

        /// Version number
        #[arg(short, long, value_parser = ValueParser::new(semver::Version::parse))]
        version_number: semver::Version,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Configuration {
    Debug,
    Release,
}

fn main() {
    let args = Args::parse();
    let output_result: anyhow::Result<String> = match args.command {
        Commands::Build {
            schema,
            destination,
            configuration,
            project,
            workspace,
        } => build(&schema, &destination, &configuration, &project, &workspace),
        Commands::BumpVersion {
            build_number,
            version_number,
        } => bump_version(&build_number, &version_number),
    };
    match output_result {
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
        Ok(output) => print!("{}", output),
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
