use clap::{ArgGroup, Parser, Subcommand, builder::ValueParser};
use xcbuild_common::{Configuration, SDK, UploadTarget};
use xctools_acknowledgements::acknowledgements;
use xctools_archive::archive;
use xctools_build::build;
use xctools_bump_version::bump_version;
use xctools_export_archive::export_archive;
use xctools_notarize::notarize;
use xctools_setup_signing::setup_signing;
use xctools_test::test;
use xctools_upload::upload;

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
        scheme: String,

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
        scheme: String,

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
        scheme: String,

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

    /// Upload archive to distribution platforms
    #[command()]
    Upload {
        /// Target choices are "ios" and "macos"
        #[arg(short, long)]
        target: UploadTarget,

        /// Path to the application file to upload
        #[arg(short, long)]
        app_file_path: String,

        /// Username for authentication
        #[arg(short, long)]
        username: String,

        /// Password for authentication
        #[arg(short, long)]
        password: String,
    },

    /// Export archive to various formats
    #[command()]
    ExportArchive {
        /// Path to the archive file to export
        #[arg(short, long)]
        archive_path: String,

        /// Path to export options plist file
        #[arg(short, long)]
        export_options: String,

        /// Export path
        #[arg(long, default_value = ".")]
        export_path: String,
    },

    /// Notarize a macOS application, disk image, or package
    #[command()]
    Notarize {
        /// Path to the file to notarize (.dmg, .pkg, or zipped .app)
        #[arg(short, long)]
        file_path: String,

        /// Apple ID email address for authentication
        #[arg(short, long)]
        apple_id: String,

        /// App-specific password for the Apple ID
        #[arg(short, long)]
        password: String,

        /// Apple Developer Team ID (10-character identifier)
        #[arg(short, long)]
        team_id: String,
    },

    /// Set up code signing for CI by importing a certificate and installing provisioning profiles
    #[command()]
    SetupSigning {
        /// Path to the P12 certificate file
        #[arg(short, long)]
        certificate_path: String,

        /// Password for the P12 certificate
        #[arg(long)]
        certificate_password: String,

        /// Paths to provisioning profile files to install (may be specified multiple times)
        #[arg(long)]
        provisioning_profile: Vec<String>,
    },
}

fn main() {
    let args = Args::parse();
    let output_result: anyhow::Result<String> = match args.command {
        Commands::Build {
            scheme,
            destination,
            configuration,
            project,
            workspace,
        } => build(&scheme, &destination, &configuration, &project, &workspace),
        Commands::BumpVersion {
            build_number,
            version_number,
        } => bump_version(&build_number, &version_number),
        Commands::Acknowledgements { app_name, output } => acknowledgements(&app_name, &output),
        Commands::Test {
            scheme,
            destination,
            configuration,
            project,
            workspace,
        } => test(&scheme, &destination, &configuration, &project, &workspace),
        Commands::Archive {
            scheme,
            destination,
            configuration,
            sdk,
            output,
            project,
            workspace,
        } => archive(
            &scheme,
            &destination,
            &configuration,
            &sdk,
            &output,
            &project,
            &workspace,
        ),
        Commands::Upload {
            target,
            app_file_path,
            username,
            password,
        } => upload(&target, &app_file_path, &username, &password),
        Commands::ExportArchive {
            archive_path,
            export_options,
            export_path,
        } => export_archive(&archive_path, &export_options, &export_path),
        Commands::Notarize {
            file_path,
            apple_id,
            password,
            team_id,
        } => notarize(&file_path, &apple_id, &password, &team_id),
        Commands::SetupSigning {
            certificate_path,
            certificate_password,
            provisioning_profile,
        } => setup_signing(
            &certificate_path,
            &certificate_password,
            &provisioning_profile,
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
