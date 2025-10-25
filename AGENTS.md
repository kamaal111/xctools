# Repository Guidelines

## Project Structure & Module Organization
- Root workspace: `Cargo.toml`, `justfile`, `.rustfmt.toml`.
- Crates live in `crates/`:
  - `xctools_cli` (binary entrypoint, integration tests in `crates/xctools_cli/tests/`).
  - Libraries: `xctools_build`, `xctools_test`, `xctools_archive`, `xctools_export_archive`, `xctools_upload`, `xctools_bump_version`, `xcbuild_common`.
- Example assets for testing live under `crates/xctools_cli/TestXcodeApp/`.
- See `MONOREPO.md` for a deeper overview.

## Build, Test, and Development Commands
- Build workspace (release): `cargo build --release`.
- Build a crate: `cargo build -p xctools_cli`.
- Run tests: `cargo test` (CI uses `--all-features`).
- Just recipes (recommended):
  - `just build` / `just build-dev` / `just build-crate <crate>`.
  - `just test`, `just test-units`, `just test-integration`.
  - Coverage: `just test-cov`, `just test-cov-output` (writes `coverage.lcov`).
  - Dev CLI examples: `just dev-build-mac-cmd`, `just dev-test-ios-cmd`.

## Coding Style & Naming Conventions
- Formatting: `rustfmt` (max line width 100 via `.rustfmt.toml`). Run `just format` or `cargo fmt`.
- Rust idioms: modules/files in `snake_case`; types/traits in `PascalCase`; functions/vars in `snake_case`.
- Keep crates focused; share primitives in `xcbuild_common`.

## Testing Guidelines
- Unit tests colocated in each crate (`src/**/*.rs` with `#[cfg(test)]`).
- Integration tests in `crates/xctools_cli/tests/` using `assert_cmd`, `predicates`, `tempfile`.
- Run all tests locally: `cargo test`; specific crate: `cargo test -p <crate>`.
- Coverage (optional): `just test-cov-html-open` to open HTML, or `just test-cov-output`.
- CI runs on `macos-latest` (see `.github/workflows/main.yml`).

## Commit & Pull Request Guidelines
- Commits: imperative mood, concise subject (≤72 chars), meaningful body when changing behavior.
- Format and test before pushing: `cargo fmt` and `cargo test`.
- PRs: clear description, scope the crates touched, include sample commands/output when changing CLI behavior.
- Link related issues, keep diffs minimal, and ensure CI is green.

## Security & Environment Notes
- macOS with Xcode is required to exercise `xcodebuild`-backed commands; avoid committing credentials.
- Prefer environment variables or local keychains for secrets; do not hardcode.
