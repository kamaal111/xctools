set export

TEST_SCHEMA := "TestXcodeApp"
TEST_PROJECT := "TestXcodeApp/TestXcodeApp.xcodeproj"

MACOS_DESTINATION := "platform=macOS"
IOS_DESTINATION := "generic/platform=iOS"

# List available commands
default:
    just --list --unsorted

# Run dev command for macOS
dev-build-mac-cmd:
    cargo run -- build --schema "$TEST_SCHEMA" --destination "$MACOS_DESTINATION" --project "$TEST_PROJECT" --configuration debug

# Run dev command for iOS
dev-build-ios-cmd:
    cargo run -- build --schema "$TEST_SCHEMA" --destination "$IOS_DESTINATION" --project "$TEST_PROJECT" --configuration release

# Run help command
help:
    cargo run -- --help

# Bootstrap project
Bootstrap:
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
