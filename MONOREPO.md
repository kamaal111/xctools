# XCTools Monorepo

This project has been restructured as a mini monorepo with separate libraries for each command.

## Structure

```
xctools/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── xctools_acknowledgements/ # Acknowledgements generation library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── xctools_build/           # Build command library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   ├── xctools_bump_version/    # Version bumping library
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

### `xctools_acknowledgements`

Contains the acknowledgements generation functionality:
- `acknowledgements()` function for generating acknowledgements files
- Scans Swift Package Manager workspace for dependencies
- Extracts package information (name, license, author, URL)
- Gathers git contributor information from commit history
- Outputs structured JSON acknowledgements file

### `xctools_build`

Contains the Xcode build functionality:
- `Configuration` enum for Debug/Release builds
- `build()` function for executing xcodebuild commands
- `BuildTarget` struct for handling project/workspace targets

### `xctools_bump_version`

Contains the version bumping functionality:
- `bump_version()` function for updating project.pbxproj files
- Support for updating both build numbers and marketing versions
- Automatic discovery of project.pbxproj files in the workspace

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
cargo build -p xctools_cli
cargo build -p xctools_acknowledgements
cargo build -p xctools_build
cargo build -p xctools_bump_version
```

## Testing

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p xctools_cli
cargo test -p xctools_acknowledgements
cargo test -p xctools_build
cargo test -p xctools_bump_version
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
xctools build --schema MyApp --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj

# Bump version
xctools bump-version --build-number 42 --version-number 2.1.0

# Generate acknowledgements
xctools acknowledgements --app-name MyApp --output ./acknowledgements.json
```
