# XCTools

A command-line tool for Xcode project management, structured as a mini monorepo with separate libraries for each command.

- [XCTools](#xctools)
  - [Overview](#overview)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Build Command](#build-command)
    - [Test Command](#test-command)
    - [Bump Version Command](#bump-version-command)
    - [Acknowledgements Command](#acknowledgements-command)
  - [Development](#development)
    - [Monorepo Structure](#monorepo-structure)
    - [Building](#building)
    - [Testing](#testing)
    - [Using Just](#using-just)
  - [License](#license)

## Overview

XCTools provides utilities for working with Xcode projects:
- **Build**: Execute xcodebuild commands with various configurations
- **Test**: Run unit tests, UI tests, and integration tests for Xcode projects
- **Bump Version**: Update project version numbers and build numbers
- **Acknowledgements**: Generate acknowledgements files for Swift Package Manager dependencies and git contributors

## Installation

```bash
cargo install --path crates/xctools_cli
```

Or build from source:

```bash
cargo build --release
# Binary will be at target/release/xctools
```

## Usage

### Build Command

```bash
# Build with project file
xctools build --schema MyApp --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj

# Build with workspace file  
xctools build --schema MyApp --destination "iOS Simulator,name=iPhone 15 Pro" --workspace MyApp.xcworkspace

# Build with specific configuration
xctools build --schema MyApp --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj --configuration release
```

### Test Command

```bash
# Run unit tests with project file
xctools test --schema MyAppTests --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj

# Run UI tests with workspace file  
xctools test --schema MyAppUITests --destination "iOS Simulator,name=iPhone 15 Pro" --workspace MyApp.xcworkspace

# Run tests with specific configuration
xctools test --schema MyAppTests --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj --configuration release

# Run tests for macOS
xctools test --schema MyAppTests --destination "platform=macOS" --project MyApp.xcodeproj
```

### Bump Version Command

```bash
# Bump build number only
xctools bump-version --build-number 42

# Bump version number only
xctools bump-version --version-number 2.1.0

# Bump both
xctools bump-version --build-number 42 --version-number 2.1.0
```

### Acknowledgements Command

```bash
# Generate acknowledgements to a specific file
xctools acknowledgements --app-name MyApp --output ./acknowledgements.json

# Generate acknowledgements to a directory (creates acknowledgements.json)
xctools acknowledgements --app-name MyApp --output ./output-directory/

# Generate acknowledgements for a specific app
xctools acknowledgements --app-name "My iOS App" --output ./Credits.json
```

The acknowledgements command:
- Scans your Swift Package Manager workspace for dependencies
- Extracts package information including name, license, author, and repository URL
- Analyzes git commit history to identify project contributors
- Generates a structured JSON file with all acknowledgements
- Automatically merges contributors with similar names
- Sorts contributors alphabetically for consistent output

## Development

### Monorepo Structure

This project is organized as a Cargo workspace with separate crates:

```
xctools/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── xcbuild_common/          # Shared Xcode build functionality
│   ├── xctools_acknowledgements/ # Acknowledgements generation library
│   ├── xctools_build/            # Build command library
│   ├── xctools_test/             # Test command library
│   ├── xctools_bump_version/     # Version bumping library
│   └── xctools_cli/              # Main CLI application
└── MONOREPO.md                   # Detailed monorepo documentation
```

- **`xcbuild_common`**: Shared library for Xcode build operations and common types
- **`xctools_acknowledgements`**: Library for generating acknowledgements files
- **`xctools_build`**: Library for Xcode build operations
- **`xctools_test`**: Library for running Xcode tests
- **`xctools_bump_version`**: Library for version management
- **`xctools_cli`**: Main CLI application that combines the libraries

See [MONOREPO.md](MONOREPO.md) for detailed information about the structure and benefits.

### Building

```bash
# Build all crates
cargo build

# Build specific crate
just build-crate xctools_cli
```

### Testing

```bash
# Run all tests
cargo test

# Run unit tests only
just test-units

# Run tests for specific crate
just test-crate xctools_build
```

### Using Just

This project includes a justfile with common commands:

```bash
# See available commands
just

# Run tests with coverage
just test-cov

# Build release version
just build

# Test specific crate
just test-crate xctools_acknowledgements
```

## License

This project is licensed under the [MIT License](./LICENSE).
