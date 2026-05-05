# XCTools

A command-line tool for Xcode project management, structured as a mini monorepo with separate libraries for each command.

- [XCTools](#xctools)
  - [Overview](#overview)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Build Command](#build-command)
    - [Test Command](#test-command)
    - [Archive Command](#archive-command)
    - [Export Archive Command](#export-archive-command)
    - [Upload Command](#upload-command)
    - [Notarize Command](#notarize-command)
    - [Setup Signing Command](#setup-signing-command)
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
- **Archive**: Create .xcarchive bundles for distribution and App Store submission
- **Export Archive**: Export .xcarchive bundles into distributable .ipa/.app files
- **Upload**: Upload application packages to distribution platforms like App Store and TestFlight
- **Notarize**: Notarize macOS applications for distribution outside the Mac App Store
- **Setup Signing**: Configure code signing in CI environments by importing certificates and installing provisioning profiles
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

XCTools is organized around the normal Xcode delivery flow:

1. `setup-signing` prepares a CI machine for code signing.
2. `build` compiles a project or workspace.
3. `test` validates the build on a simulator or macOS destination.
4. `archive` creates a distributable `.xcarchive`.
5. `export-archive` turns that archive into an `.ipa`, `.app`, or other export artifact.
6. `upload` sends the exported artifact to Apple distribution services.
7. `notarize` is the macOS-specific step for apps distributed outside the Mac App Store.
8. `bump-version` updates marketing/build versions in `project.pbxproj`.
9. `acknowledgements` generates dependency and contributor credits for shipping apps.

### Common command patterns

- `build`, `test`, and `archive` always require **exactly one** of `--project` or `--workspace`.
- `--configuration` defaults to `debug`; use `release` for distribution builds.
- `--destination` is passed through to `xcodebuild`, so use the same destination strings you would use with native Xcode CLI commands.
- Commands that accept credentials (`upload`, `notarize`, `setup-signing`) are intended for CI use. Prefer environment variables over hardcoded secrets.

### Build Command

**Why you use it:** compile an app, framework, or test target without opening Xcode. This is the fastest way to verify that a scheme can build in local scripts and CI jobs.

**When to use it:** before running tests, before archiving, or whenever you want a simple compile check for a specific scheme and destination.

**What it runs:** `xcodebuild build`

```bash
# Build with a project
xctools build --scheme MyApp --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj

# Build with a workspace
xctools build --scheme MyApp --destination "iOS Simulator,name=iPhone 15 Pro" --workspace MyApp.xcworkspace

# Build a Release configuration
xctools build --scheme MyApp --destination "platform=macOS" --project MyApp.xcodeproj --configuration release
```

Use `build` when you only need compilation output. If you need test execution, use `test` instead.

### Test Command

**Why you use it:** run Xcode-managed tests from the command line with the same scheme/destination model used by `xcodebuild test`.

**When to use it:** to validate unit tests, UI tests, or other test bundles in CI and local automation.

**What it runs:** `xcodebuild test`

```bash
# Run unit tests from a project
xctools test --scheme MyAppTests --destination "iOS Simulator,name=iPhone 15 Pro" --project MyApp.xcodeproj

# Run tests from a workspace
xctools test --scheme MyAppUITests --destination "iOS Simulator,name=iPhone 15 Pro" --workspace MyApp.xcworkspace

# Run macOS tests
xctools test --scheme MyMacAppTests --destination "platform=macOS" --project MyMacApp.xcodeproj
```

Use `test` instead of `build` when you need proof that the compiled app still behaves correctly.

### Archive Command

**Why you use it:** create the `.xcarchive` bundle required for exporting or distributing an app.

**When to use it:** after a successful build/test run and before `export-archive`.

**What it runs:** `xcodebuild archive`

```bash
# Archive an iOS app for distribution
xctools archive --scheme MyApp --destination "generic/platform=iOS" --sdk iphoneos --output MyApp.xcarchive --project MyApp.xcodeproj --configuration release

# Archive a macOS app from a workspace
xctools archive --scheme MyMacApp --destination "generic/platform=macOS" --sdk macosx --output MyMacApp.xcarchive --workspace MyMacApp.xcworkspace --configuration release

# Archive to a custom location
xctools archive --scheme MyApp --destination "generic/platform=iOS" --sdk iphoneos --output ./build/archives/MyApp-v1.0.xcarchive --project MyApp.xcodeproj --configuration release
```

Use a generic destination such as `generic/platform=iOS` or `generic/platform=macOS` for distribution archives.

### Export Archive Command

**Why you use it:** turn an `.xcarchive` into the package you actually distribute, such as an `.ipa` or signed macOS export.

**When to use it:** after `archive`, once you know which distribution method you need (App Store, TestFlight, ad hoc, enterprise, development, Developer ID, and so on).

**What it runs:** `xcodebuild -exportArchive`

```bash
# Export an App Store build
xctools export-archive --archive-path MyApp.xcarchive --export-options AppStoreExportOptions.plist --export-path build/appstore

# Export an ad hoc build
xctools export-archive --archive-path MyApp.xcarchive --export-options AdHocExportOptions.plist --export-path build/adhoc

# Export a macOS Developer ID build
xctools export-archive --archive-path MyMacApp.xcarchive --export-options DeveloperIDExportOptions.plist --export-path build/developerid
```

The `--export-options` plist controls signing and distribution behavior. A minimal App Store example looks like this:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>method</key>
    <string>app-store</string>
    <key>teamID</key>
    <string>YOUR_TEAM_ID</string>
</dict>
</plist>
```

Use `export-archive` when you need a ship-ready artifact rather than just an `.xcarchive`.

### Upload Command

**Why you use it:** push an exported package to Apple's distribution services from automation instead of manually using Transporter or App Store Connect UI.

**When to use it:** after `export-archive`, once you have an `.ipa`, `.pkg`, or similar uploadable artifact.

**What it runs:** `xcrun altool --upload-app`

```bash
# Upload an iOS build
xctools upload --target ios --app-file-path MyApp.ipa --username "$APPLE_ID" --password "$APP_SPECIFIC_PASSWORD"

# Upload a macOS package
xctools upload --target macos --app-file-path MyMacApp.pkg --username "$APPLE_ID" --password "$APP_SPECIFIC_PASSWORD"
```

Use environment variables for `--username` and `--password` so CI logs and shell history do not expose credentials.

### Notarize Command

**Why you use it:** satisfy Apple Gatekeeper requirements for macOS apps distributed outside the Mac App Store.

**When to use it:** after exporting a macOS `.dmg`, `.pkg`, or zipped `.app` that will be downloaded directly by users.

**What it runs:** `xcrun notarytool submit --wait` followed by `xcrun stapler staple`

```bash
# Notarize a DMG
xctools notarize --file-path MyApp.dmg --apple-id "$APPLE_ID" \
    --password "$APP_SPECIFIC_PASSWORD" --team-id A1B2C3D4E5

# Notarize a package
xctools notarize --file-path MyApp.pkg --apple-id "$APPLE_ID" \
    --password "$APP_SPECIFIC_PASSWORD" --team-id A1B2C3D4E5
```

Use `notarize` for direct macOS distribution. It is usually not part of an iOS release pipeline.

### Setup Signing Command

**Why you use it:** configure a clean CI machine so `xcodebuild`, `codesign`, and export steps can sign apps non-interactively.

**When to use it:** at the beginning of release jobs that need certificates or provisioning profiles.

**What it does:** creates a dedicated keychain, imports the P12 certificate, unlocks the keychain, and installs any provisioning profiles you pass.

```bash
# Import one certificate and two provisioning profiles
xctools setup-signing \
    --certificate-path signing.p12 \
    --certificate-password "$CERT_PASSWORD" \
    --provisioning-profile AppStore.mobileprovision \
    --provisioning-profile WatchApp.mobileprovision

# Import a Developer ID certificate only
xctools setup-signing \
    --certificate-path DeveloperID.p12 \
    --certificate-password "$CERT_PASSWORD"
```

This command is mainly useful in ephemeral CI environments where no signing state is preconfigured.

### Bump Version Command

**Why you use it:** update build metadata directly in `project.pbxproj` as part of release automation.

**When to use it:** before archiving or tagging a release when you need to advance `CURRENT_PROJECT_VERSION`, `MARKETING_VERSION`, or both.

**What it changes:** the first `project.pbxproj` it finds under the current directory tree.

```bash
# Update build number only
xctools bump-version --build-number 42

# Update marketing version only
xctools bump-version --version-number 2.1.0

# Update both values together
xctools bump-version --build-number 42 --version-number 2.1.0
```

Use `bump-version` when you want versioning to happen inside scripted release steps instead of by hand in Xcode.

### Acknowledgements Command

**Why you use it:** generate a credits file for shipped apps that includes both Swift Package Manager dependencies and project contributors.

**When to use it:** near the end of a release workflow, or anytime you need to refresh a bundled acknowledgements/credits artifact.

**What it reads:** Xcode DerivedData for Swift package metadata and `git log` for contributor history.

```bash
# Write to a file
xctools acknowledgements --app-name MyApp --output ./acknowledgements.json

# Write into a directory (creates acknowledgements.json)
xctools acknowledgements --app-name MyApp --output ./output-directory/

# Use a custom app name and file name
xctools acknowledgements --app-name "My iOS App" --output ./Credits.json
```

Run the app at least once before using this command so the necessary DerivedData package metadata exists.

## Development

### Monorepo Structure

This project is organized as a Cargo workspace with separate crates:

```
xctools/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── xcbuild_common/          # Shared Xcode build functionality
│   ├── xctools_acknowledgements/ # Acknowledgements generation library
│   ├── xctools_archive/          # Archive creation library
│   ├── xctools_build/            # Build command library
│   ├── xctools_export_archive/   # Archive export library
│   ├── xctools_notarize/         # macOS notarization library
│   ├── xctools_setup_signing/    # CI code signing setup library
│   ├── xctools_test/             # Test command library
│   ├── xctools_bump_version/     # Version bumping library
│   ├── xctools_upload/           # Upload command library
│   └── xctools_cli/              # Main CLI application
└── MONOREPO.md                   # Detailed monorepo documentation
```

- **`xcbuild_common`**: Shared library for Xcode build operations and common types
- **`xctools_acknowledgements`**: Library for generating acknowledgements files
- **`xctools_archive`**: Library for creating .xcarchive bundles for distribution
- **`xctools_build`**: Library for Xcode build operations
- **`xctools_export_archive`**: Library for exporting .xcarchive bundles into distributable formats
- **`xctools_notarize`**: Library for notarizing macOS applications
- **`xctools_setup_signing`**: Library for CI code signing setup (certificates and provisioning profiles)
- **`xctools_test`**: Library for running Xcode tests
- **`xctools_bump_version`**: Library for version management
- **`xctools_upload`**: Library for uploading applications to distribution platforms
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
