# XCTools Monorepo

This project has been restructured as a mini monorepo with separate libraries for each command.

## Structure

```
xctools/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── xcbuild_common/          # Shared Xcode build functionality
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── xctools_acknowledgements/ # Acknowledgements generation library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── xctools_archive/         # Archive creation library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── xctools_build/           # Build command library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── xctools_export_archive/  # Archive export library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── xctools_test/            # Test command library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── xctools_bump_version/    # Version bumping library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── xctools_upload/          # Upload command library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── xctools_cli/             # Main CLI application
│       ├── Cargo.toml
│       ├── src/
│       │   └── main.rs
│       └── tests/
│           └── integration_tests.rs
```

## Libraries

### `xcbuild_common`

Contains shared functionality for Xcode build operations:
- `Configuration` enum for Debug/Release builds with command string conversion
- `BuildTarget` struct for handling project/workspace targets
- `XcodebuildCommandAction` enum for Build/Test actions
- `run_xcodebuild_command()` function for executing xcodebuild commands
- `make_xcodebuild_command()` helper for constructing command strings

### `xctools_acknowledgements`

Contains the acknowledgements generation functionality:
- `acknowledgements()` function for generating acknowledgements files
- Scans Swift Package Manager workspace for dependencies
- Extracts package information (name, license, author, URL)
- Gathers git contributor information from commit history
- Outputs structured JSON acknowledgements file

### `xctools_archive`

Contains the Xcode archive functionality:
- `archive()` function for creating .xcarchive bundles using xcodebuild archive commands
- Supports iOS and macOS archive creation with proper SDK selection
- Creates archives for App Store submission, enterprise distribution, and testing
- Uses shared `Configuration`, `BuildTarget`, `SDK`, and `XcodebuildCommandAction` from `xcbuild_common`
- Generates archives with debug symbols (dSYMs) for crash symbolication

### `xctools_export_archive`

Contains the Xcode archive export functionality:
- `export_archive()` function for exporting .xcarchive bundles into distributable formats using xcodebuild -exportArchive commands
- Exports .ipa files for iOS applications and .app bundles for macOS applications
- Supports multiple distribution methods: App Store, TestFlight, Ad Hoc, Enterprise, and Development
- Uses ExportOptions.plist files to specify export method, team ID, and signing configuration
- Handles code signing and provisioning profile selection automatically
- Creates properly signed applications ready for distribution or App Store submission
- Uses shared `XcodebuildCommandAction` and `XcodebuildParams` from `xcbuild_common`

### `xctools_build`

Contains the Xcode build functionality:
- `build()` function for executing xcodebuild build commands  
- Uses shared `Configuration`, `BuildTarget`, and `XcodebuildCommandAction` from `xcbuild_common`

### `xctools_test`

Contains the Xcode test functionality:
- `test()` function for running xcodebuild test commands
- Support for unit tests, UI tests, integration tests, and performance tests
- Uses shared `Configuration`, `BuildTarget`, and `XcodebuildCommandAction` from `xcbuild_common`

### `xctools_bump_version`

Contains the version bumping functionality:
- `bump_version()` function for updating project.pbxproj files
- Support for updating both build numbers and marketing versions
- Automatic discovery of project.pbxproj files in the workspace

### `xctools_upload`

Contains the application upload functionality:
- `upload()` function for uploading application packages to distribution platforms
- Uses `xcrun altool` to upload iOS (.ipa) and macOS (.pkg, .dmg) files
- Supports App Store, TestFlight, and enterprise distribution workflows
- Handles authentication using Apple ID credentials
- Provides detailed output from the upload process

### `xctools_cli`

The main command-line interface that:
- Uses clap for argument parsing
- Imports and uses the other crates' functionality
- Provides the unified `xctools` binary

## Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p xcbuild_common
cargo build -p xctools_cli
cargo build -p xctools_acknowledgements
cargo build -p xctools_archive
cargo build -p xctools_export_archive
cargo build -p xctools_build
cargo build -p xctools_test
cargo build -p xctools_bump_version
cargo build -p xctools_upload
```

## Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p xcbuild_common
cargo test -p xctools_cli
cargo test -p xctools_acknowledgements
cargo test -p xctools_archive
cargo test -p xctools_export_archive
cargo test -p xctools_build
cargo test -p xctools_test
cargo test -p xctools_bump_version
cargo test -p xctools_upload
```

## Benefits of this Structure

1. **Modularity**: Each command is in its own library with clear boundaries
2. **Reusability**: Libraries can be used independently or in other projects
3. **Testing**: Each library can be tested in isolation
4. **Development**: Changes to one command don't affect others
5. **Documentation**: Each crate can have its own focused documentation

## Usage

The CLI interface remains unchanged:

```bash
# Build Xcode project
xctools build --scheme MyApp --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj

# Run Xcode tests
xctools test --scheme MyAppTests --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj

# Create Xcode archive
xctools archive --scheme MyApp --destination "generic/platform=iOS" --sdk iphoneos --output MyApp.xcarchive --project MyApp.xcodeproj --configuration release

# Export Xcode archive for distribution
xctools export-archive --archive-path MyApp.xcarchive --export-options ExportOptions.plist --export-path build/export

# Bump version
xctools bump-version --build-number 42 --version-number 2.1.0

# Generate acknowledgements
xctools acknowledgements --app-name MyApp --output ./acknowledgements.json

# Upload application to distribution platforms
xctools upload --target ios --app-file-path MyApp.ipa --username developer@example.com --password app-specific-password
```
