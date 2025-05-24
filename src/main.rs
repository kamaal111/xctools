use clap::{ArgGroup, Parser, Subcommand};

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

        /// Xcode project file (.xcodeproj)
        #[arg(short, long)]
        project: Option<String>,

        /// Xcode workspace file (.xcworkspace)
        #[arg(short, long)]
        workspace: Option<String>,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Build {
            schema,
            destination,
            project,
            workspace,
        } => {
            println!("Building schema: {}", schema);
            println!("Destination: {}", destination);

            if let Some(project_path) = project {
                println!("Using project: {}", project_path);
            } else if let Some(workspace_path) = workspace {
                println!("Using workspace: {}", workspace_path);
            }

            // Add your build logic here
        }
    }
}
