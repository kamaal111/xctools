# xctools

A command-line tool for working with Xcode projects and workspaces. Provides utilities for building, managing, and automating common Xcode development tasks.

## Installation

Clone the repository and build the project:

```bash
git clone <repository-url>
cd xctools
cargo build --release
```

The binary will be available at `target/release/xctools`.

## Usage

xctools provides several commands for working with Xcode projects and workspaces.

### Build Command

Build an Xcode project or workspace with the specified configuration.

```bash
xctools build [OPTIONS] --schema <SCHEMA> --destination <DESTINATION> <--project <PROJECT>|--workspace <WORKSPACE>>
```

#### Required Arguments

-   `--schema, -s <SCHEMA>`: The Xcode scheme to build
-   `--destination, -d <DESTINATION>`: The build destination (e.g., "iOS Simulator,name=iPhone 15 Pro")
-   Either `--project` or `--workspace` (mutually exclusive):
    -   `--project, -p <PROJECT>`: Xcode project file (.xcodeproj)
    -   `--workspace, -w <WORKSPACE>`: Xcode workspace file (.xcworkspace)

#### Optional Arguments

-   `--configuration, -c <CONFIGURATION>`: Build configuration [default: debug] [possible values: debug, release]

#### Examples

Build a project with Debug configuration:

```bash
xctools build --schema MyApp --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj
```

Build a workspace with Release configuration:

```bash
xctools build --schema MyApp --destination "iOS Simulator,name=iPhone 15 Pro" --workspace MyApp.xcworkspace --configuration release
```

Build for a physical device:

```bash
xctools build --schema MyApp --destination "generic/platform=iOS" --project MyApp.xcodeproj --configuration release
```

#### Common Destinations

-   iOS Simulator: `"iOS Simulator,name=iPhone 15 Pro"`
-   iOS Device: `"generic/platform=iOS"`
-   macOS: `"generic/platform=macOS"`
-   Apple TV Simulator: `"tvOS Simulator,name=Apple TV"`
-   Apple Watch Simulator: `"watchOS Simulator,name=Apple Watch Series 9 (45mm)"`

### Bump Version Command

Bump the version of an Xcode project.

```bash
xctools bump-version [OPTIONS]
```

#### Options

You must provide at least one of the following options:

-   `--build-number, -b <BUILD_NUMBER>`: The build number (e.g., 123).
-   `--version-number, -v <VERSION_NUMBER>`: The version number in semver format (e.g., 1.2.3).

#### Examples

Set a new build number:

```bash
xctools bump-version --build-number 10
```

Set a new version number:

```bash
xctools bump-version --version-number 1.0.2
```

Set both build number and version number:

```bash
xctools bump-version --build-number 10 --version-number 1.0.2
```

## Development

This project is designed to be extensible with additional Xcode automation commands. Currently supports building projects and workspaces, with plans to add more utilities for common Xcode development workflows.

### Building from Source

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Development Build

```bash
cargo run -- build --help
```

## License

This project is licensed under the [MIT License](./LICENSE).
