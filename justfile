set export

TEST_SCHEMA := "TestXcodeApp"
TEST_PROJECT := "TestXcodeApp/TestXcodeApp.xcodeproj"

MACOS_DESTINATION := "platform=macOS"
IOS_DESTINATION := "generic/platform=iOS"

# List available commands
default:
    just --list --unsorted

# Test project
test:
    cargo test

# Run integration tests
test-integration:
    cargo test --test integration_tests

# Test with coverage
test-cov:
    cargo llvm-cov

# Test with coverage and output coverage file
test-cov-output:
    cargo llvm-cov --lcov --output-path coverage.lcov

# Test with coverage as JSON format
test-cov-json:
    cargo llvm-cov --json | jq '{"coverage_pct": .data[0].totals.lines.percent, "lines_covered": .data[0].totals.lines.covered, "lines_total": .data[0].totals.lines.count, "functions_covered": .data[0].totals.functions.covered, "functions_total": .data[0].totals.functions.count}'

# Build project
build:
    cargo build --release

# Build project in debug
build-dev:
    cargo build

# Test with coverage and open HTML
test-cov-html-open:
    cargo llvm-cov --open

# Run dev command to build for macOS
dev-build-mac-cmd:
    cargo run -- build --schema "$TEST_SCHEMA" --destination "$MACOS_DESTINATION" --project "$TEST_PROJECT" --configuration debug

# Run dev command to build for iOS
dev-build-ios-cmd:
    cargo run -- build --schema "$TEST_SCHEMA" --destination "$IOS_DESTINATION" --project "$TEST_PROJECT" --configuration release

# Run dev command to bump version
dev-bump-version-cmd:
    cargo run -- bump-version --version-number 1.0.1 --build-number 2

# Run help command
help:
    cargo run -- --help

# Bootstrap project
Bootstrap: install-rust install-cov-tool

[private]
install-rust:
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

[private]
install-cov-tool:
    cargo install cargo-llvm-cov
