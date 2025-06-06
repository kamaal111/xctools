use clap::{ArgGroup, Parser, Subcommand, builder::ValueParser};
use xcbuild_common::{Configuration, SDK};
use xctools_acknowledgements::acknowledgements;
use xctools_archive::archive;
use xctools_build::build;
use xctools_bump_version::bump_version;
use xctools_test::test;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build Xcode project
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

        /// Xcode project folder (.xcodeproj)
        #[arg(short, long)]
        project: Option<String>,

        /// Xcode workspace file (.xcworkspace)
        #[arg(short, long)]
        workspace: Option<String>,
    },

    /// Test Xcode project
    #[command(group(
        ArgGroup::new("target")
            .required(true)
            .args(["project", "workspace"]),
    ))]
    Test {
        /// The Xcode scheme to build.
        #[arg(short, long)]
        schema: String,

        /// The build destination (e.g., "iOS Simulator,name=iPhone 15 Pro").
        #[arg(short, long)]
        destination: String,

        /// Configuration - "Debug" or "Release"
        #[arg(short, long, default_value_t = Configuration::default())]
        configuration: Configuration,

        /// Xcode project folder (.xcodeproj)
        #[arg(short, long)]
        project: Option<String>,

        /// Xcode workspace file (.xcworkspace)
        #[arg(short, long)]
        workspace: Option<String>,
    },

    /// Bump version of Xcode project
    #[command(group(
        ArgGroup::new("version_params")
            .required(true)
            .multiple(true)
            .args(["build_number", "version_number"]),
    ))]
    BumpVersion {
        /// Build number
        #[arg(short, long)]
        build_number: Option<i32>,

        /// Version number
        #[arg(short, long, value_parser = ValueParser::new(semver::Version::parse))]
        version_number: Option<semver::Version>,
    },

    /// Generate acknowledgements file
    #[command()]
    Acknowledgements {
        /// App name
        #[arg(short, long)]
        app_name: String,

        /// Generated acknowledgements file output destination
        #[arg(short, long)]
        output: String,
    },

    /// Archive Xcode project
    #[command(group(
        ArgGroup::new("target")
            .required(true)
            .args(["project", "workspace"]),
    ))]
    Archive {
        /// The Xcode scheme to build.
        #[arg(long)]
        schema: String,

        /// The build destination (e.g., "iOS Simulator,name=iPhone 15 Pro").
        #[arg(short, long)]
        destination: String,

        /// SDK to use to perform the archiving - "iphoneos" or "macosx"
        #[arg(long)]
        sdk: SDK,

        /// Configuration - "debug" or "release"
        #[arg(short, long, default_value_t = Configuration::default())]
        configuration: Configuration,

        /// Where to output the archive
        #[arg(short, long)]
        output: String,

        /// Xcode project folder (.xcodeproj)
        #[arg(short, long)]
        project: Option<String>,

        /// Xcode workspace file (.xcworkspace)
        #[arg(short, long)]
        workspace: Option<String>,
    },
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
        Commands::Acknowledgements { app_name, output } => acknowledgements(&app_name, &output),
        Commands::Test {
            schema,
            destination,
            configuration,
            project,
            workspace,
        } => test(&schema, &destination, &configuration, &project, &workspace),
        Commands::Archive {
            schema,
            destination,
            configuration,
            sdk,
            output,
            project,
            workspace,
        } => archive(
            &schema,
            &destination,
            &configuration,
            &sdk,
            &output,
            &project,
            &workspace,
        ),
    };
    match output_result {
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
        Ok(output) => print!("{}", output),
    }
}
