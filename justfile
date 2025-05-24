set export

TEST_SCHEMA := "TestXcodeApp"
TEST_PROJECT := "TestXcodeApp/TestXcodeApp.xcodeproj"

MACOS_DESTINATION := "platform=macOS"
IOS_DESTINATION := "generic/platform=iOS"

# List available commands
default:
    just --list --unsorted

# Run dev script
dev-build-cmd:
    cargo run -- build --schema "$TEST_SCHEMA" --destination "$MACOS_DESTINATION" --project "$TEST_PROJECT"

# Run help command
help:
    cargo run -- --help

# Bootstrap project
Bootstrap:
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
