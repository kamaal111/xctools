CARGO := "cargo"

TEST_SCHEMA := "TestXcodeApp"
TEST_PROJECT := "crates/xctools_cli/TestXcodeApp/TestXcodeApp.xcodeproj"

MACOS_DESTINATION := "platform=macOS"
IOS_DESTINATION := "generic/platform=iOS"
IPHONE_16_PRO_DESTINATION := "platform=iOS Simulator,name=iPhone 16 Pro"

# List available commands
default:
    just --list --unsorted

# Test project
test:
    {{ CARGO }} test

# Test specific crate
test-crate crate:
    {{ CARGO }} test -p {{ crate }}

# Run integration tests
test-integration:
    {{ CARGO }} test --test integration_tests

# Run unit tests for all crates
test-units:
    {{ CARGO }} test --lib

# Test with coverage
test-cov:
    {{ CARGO }} llvm-cov

# Test with coverage and output coverage file
test-cov-output:
    {{ CARGO }} llvm-cov --lcov --output-path coverage.lcov

# Test with coverage as JSON format
test-cov-json:
    {{ CARGO }} llvm-cov --json | jq '{"coverage_pct": .data[0].totals.lines.percent, "lines_covered": .data[0].totals.lines.covered, "lines_total": .data[0].totals.lines.count, "functions_covered": .data[0].totals.functions.covered, "functions_total": .data[0].totals.functions.count}'

# Test with coverage and open HTML
test-cov-html-open:
    {{ CARGO }} llvm-cov --open

# Build project
build:
    {{ CARGO }} build --release

# Build specific crate
build-crate crate:
    {{ CARGO }} build -p {{ crate }} --release

# Build project in debug
build-dev:
    {{ CARGO }} build

# Format code
format:
    {{ CARGO }} fmt

# Build specific crate in debug
build-dev-crate crate:
    {{ CARGO }} build -p {{ crate }}

# Run dev command to build for macOS
dev-build-mac-cmd:
    {{ CARGO }} run -- build --schema "{{ TEST_SCHEMA }}" --destination "{{ MACOS_DESTINATION }}" \
        --project "{{ TEST_PROJECT }}" --configuration debug

# Run dev command to build for iOS
dev-build-ios-cmd:
    {{ CARGO }} run -- build --schema "{{ TEST_SCHEMA }}" --destination "{{ IOS_DESTINATION }}" \
        --project "{{ TEST_PROJECT }}" --configuration release

# Run dev command to bump version
dev-bump-version-cmd:
    {{ CARGO }} run -- bump-version --version-number 1.0.1 --build-number 2

# Run dev command to test for macOS
dev-test-mac-cmd:
    {{ CARGO }} run -- test --schema "{{ TEST_SCHEMA }}" --destination "{{ MACOS_DESTINATION }}" \
        --project "{{ TEST_PROJECT }}" --configuration debug

# Run dev command to test for iOS
dev-test-ios-cmd:
    {{ CARGO }} run -- test --schema "{{ TEST_SCHEMA }}" --destination "{{ IPHONE_16_PRO_DESTINATION }}" \
        --project "{{ TEST_PROJECT }}" --configuration debug

# Run dev command to archive macOS app
dev-archive-mac-cmd:
    {{ CARGO }} run -- archive --schema "{{ TEST_SCHEMA }}" --destination "{{ MACOS_DESTINATION }}" \
        --project "{{ TEST_PROJECT }}" --configuration debug --sdk macosx --output ./archival.xcarchive

# Run help command
help:
    {{ CARGO }} run -- --help

# Run help command for subcommand
help-cmd command:
    {{ CARGO }} run -- {{ command }} --help

# Bootstrap project
Bootstrap: install-rust install-cov-tool

[private]
install-rust:
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

[private]
install-cov-tool:
    {{ CARGO }} install cargo-llvm-cov
