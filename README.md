# XCTools

A command-line tool for Xcode project management, structured as a mini monorepo with separate libraries for each command.

- [XCTools](#xctools)
  - [Overview](#overview)
  - [Monorepo Structure](#monorepo-structure)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Build Command](#build-command)
    - [Bump Version Command](#bump-version-command)
  - [Development](#development)
    - [Building](#building)
    - [Testing](#testing)
    - [Using Just](#using-just)
  - [License](#license)

## Overview

XCTools provides utilities for working with Xcode projects:
- **Build**: Execute xcodebuild commands with various configurations
- **Bump Version**: Update project version numbers and build numbers

## Monorepo Structure

This project is organized as a Cargo workspace with separate crates:

```
xctools/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── xctools-build/           # Build command library
│   ├── xctools-bump-version/    # Version bumping library  
│   └── xctools-cli/             # Main CLI application
└── MONOREPO.md                  # Detailed monorepo documentation
```

- **`xctools-build`**: Library for Xcode build operations
- **`xctools-bump-version`**: Library for version management  
- **`xctools-cli`**: Main CLI application that combines the libraries

See [MONOREPO.md](MONOREPO.md) for detailed information about the structure and benefits.

## Installation

```bash
cargo install --path crates/xctools-cli
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

### Bump Version Command

```bash
# Bump build number only
xctools bump-version --build-number 42

# Bump version number only
xctools bump-version --version-number 2.1.0

# Bump both
xctools bump-version --build-number 42 --version-number 2.1.0
```

## Development

### Building

```bash
# Build all crates
cargo build

# Build specific crate
just build-crate xctools-cli
```

### Testing

```bash
# Run all tests
cargo test

# Run unit tests only
just test-units

# Run tests for specific crate
just test-crate xctools-build
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
just test-crate xctools-bump-version
```

## License

This project is licensed under the [MIT License](./LICENSE).
