# XCTools Constitution

## Core Principles

### I. Monorepo Library-First Architecture

**MUST**: Every feature MUST be implemented as a standalone library crate before CLI integration.

**MUST**: Each library crate MUST be independently buildable, testable, and documented with clear purpose.

**MUST NOT**: Create organizational-only crates without standalone functionality or business value.

**Rationale**: The monorepo structure (workspace with xctools_* crates) ensures modularity, reusability, and independent testing. Libraries like `xctools_build`, `xctools_test`, and `xctools_archive` can be used programmatically without the CLI wrapper, enabling future integrations (CI/CD scripts, other Rust tools, IDEs).

### II. CLI-Centric Interface Design

**MUST**: Every library MUST expose its functionality through the unified CLI (`xctools_cli`).

**MUST**: CLI commands MUST follow text-based protocol: arguments/stdin → stdout, errors → stderr.

**MUST**: Use structured argument parsing with clap derive API, mutual exclusivity groups, and value validation.

**Rationale**: The CLI provides unified access to all Xcode tooling operations. Consistent argument patterns (--scheme, --destination, --project/--workspace exclusivity) ensure predictable user experience. Text-based I/O enables shell scripting, CI/CD integration, and debuggability.

### III. Comprehensive Error Handling

**MUST**: All public library functions MUST return `anyhow::Result<T>` for error propagation.

**MUST**: Use `.context()` method to add actionable context to errors (e.g., file paths, command arguments).

**MUST**: Error messages MUST specify: what failed, why it failed, and how to fix it when possible.

**MUST NOT**: Use unwrap() or expect() in production code paths; reserved for tests only.

**Rationale**: The `anyhow` crate provides excellent error context chaining. All library functions (`build`, `test`, `archive`, etc.) use `anyhow::Result` consistently. Contextual errors help users diagnose issues with xcodebuild commands, missing files, or invalid configurations.

### IV. Test-Driven Development (NON-NEGOTIABLE)

**MUST**: Tests MUST be written BEFORE implementation for all new features.

**MUST**: Follow Red-Green-Refactor cycle: Write failing test → Implement until test passes → Refactor.

**MUST**: Tests MUST fail when run against unimplemented functionality before coding begins.

**MUST**: Integration tests MUST verify CLI contract before merging features.

**Rationale**: TDD ensures requirements are testable, prevents regression, and documents expected behavior. The integration_tests.rs file demonstrates this: CLI argument validation tests, error case tests, and success path tests all exist before feature changes.

### V. Multi-Layer Testing Strategy

**MUST**: Implement THREE test layers:
- **Doctests**: Embedded in library documentation (`///` comments with ```rust blocks) for API usage examples
- **Unit tests**: Library-level tests in `src/` using `#[test]` for business logic validation
- **Integration tests**: CLI-level tests in `tests/` using assert_cmd for end-to-end verification

**MUST**: New library contracts MUST have integration tests in `crates/xctools_cli/tests/integration_tests.rs`.

**MUST**: Breaking changes to shared libraries (xcbuild_common) MUST have contract tests.

**Rationale**: Multi-layer testing catches different bug classes. Doctests ensure examples stay valid, unit tests verify library logic, integration tests confirm CLI contract. Coverage tracking via `just test-cov` enforces comprehensive testing.

### VI. Documentation as Code

**MUST**: All public functions MUST have rustdoc comments with:
- Purpose summary
- Complete `# Arguments` section with types and descriptions
- `# Returns` section explaining success/error cases
- `# Examples` section with at least one doctest

**MUST**: Examples MUST demonstrate real usage patterns, not trivial cases.

**MUST**: Update README.md when adding new commands or changing CLI interfaces.

**Rationale**: Documentation ensures maintainability and usability. Extensive rustdoc comments (see `xctools_build::build` with 100+ line docs) serve as specification and living examples via doctests. README provides user-facing reference.

### VII. Performance & Resource Efficiency

**MUST**: CLI operations MUST complete in <30 seconds for typical projects (excluding xcodebuild execution time).

**MUST**: Minimize filesystem operations: batch reads, avoid repeated parsing of same files.

**MUST**: Use efficient data structures: BTreeMap for sorted iteration, HashSet for deduplication.

**SHOULD**: Profile operations processing >10k items (e.g., acknowledgements scanning package dependencies).

**Rationale**: CLI tools must be fast to integrate smoothly into development workflows. The acknowledgements command demonstrates efficient file scanning with glob patterns and deduplication using HashSet.

### VIII. Code Quality & Idiomatic Rust

**MUST**: Follow Rust idioms:
- Builder pattern for complex structs (see `XcodebuildParams`)
- Prefer `impl` blocks for methods over free functions
- Use `Option<T>` for optional values, not sentinel values
- Leverage type system: enums for variants (`Configuration`, `SDK`), structs for grouping

**MUST**: Run `cargo clippy` and address all warnings before PR submission.

**MUST**: Format code with `cargo fmt` (rustfmt) using default configuration.

**MUST**: Workspace dependencies MUST be defined in root Cargo.toml `[workspace.dependencies]`.

**Rationale**: Consistent code style reduces cognitive load. The project uses workspace dependency management, clap derive macros, and builder patterns throughout. Clippy catches common mistakes and non-idiomatic patterns.

## Quality Standards

### Error Handling Requirements

- **Context Chaining**: Use `anyhow::Context` to wrap errors with actionable information
- **User-Facing Messages**: Errors displayed to CLI users MUST be clear and suggest fixes
- **Graceful Degradation**: Where possible, continue operation with warnings rather than hard failures
- **Example Pattern**:
  ```rust
  File::open(path)
      .context(format!("Failed to open export options file at {}", path))?
  ```

### Documentation Requirements

- **Public APIs**: 100% coverage for public functions and types
- **Complex Logic**: Internal complex functions SHOULD have doc comments
- **Examples**: At least one doctest per public function showing typical usage
- **Maintenance**: Update docs when changing function signatures or behavior

### Code Style Requirements

- **Naming**: snake_case for functions/variables, PascalCase for types, SCREAMING_SNAKE_CASE for constants
- **Line Length**: Target <100 characters, hard limit 120 characters
- **Imports**: Group by std → external crates → internal crates, alphabetically sorted
- **Comments**: Use `//` for implementation notes, `///` for rustdoc, `//!` for module docs

## Development Workflow

### Test-First Workflow

1. **Write Specification**: Document expected behavior in spec.md with acceptance criteria
2. **Write Failing Tests**: Create integration/unit tests that encode specification
3. **Verify Failure**: Run tests, confirm they fail with expected error messages
4. **Implement Feature**: Write minimum code to make tests pass
5. **Refactor**: Improve implementation while keeping tests green
6. **Update Docs**: Add/update rustdoc comments and README if needed

### Pull Request Requirements

**MUST**: Every PR MUST include:
- Tests covering new/changed functionality (doctest + integration test minimum)
- Updated documentation (rustdoc comments + README if applicable)
- Clean `cargo clippy` run (no warnings)
- All tests passing (`cargo test`)
- Constitution compliance verification (checklist in PR description)

**SHOULD**: PRs SHOULD be focused: one feature or bug fix per PR, not multiple unrelated changes.

### CI/CD Expectations

**MUST**: CI pipeline MUST enforce:
- `cargo test` passes on all crates
- `cargo clippy` produces no warnings
- `cargo fmt --check` confirms formatting compliance
- Coverage metrics tracked (via llvm-cov, target >70% line coverage)

**SHOULD**: Use justfile commands for local verification:
- `just test` - run all tests
- `just test-cov` - generate coverage report
- `just build` - release build

## Governance

This constitution supersedes all other development practices and guidelines. Any deviation from core principles (I-VIII) MUST be explicitly documented with justification.

**Amendment Process**:
- Amendments require: written proposal, team review, approval from project maintainer
- MAJOR version bump: Removing/redefining principles, incompatible governance changes
- MINOR version bump: Adding new principles, expanding guidance materially
- PATCH version bump: Clarifications, typo fixes, non-semantic refinements

**Compliance Review**:
- All PRs MUST verify compliance with principles I-VIII before merge
- Complexity introduced (e.g., new dependencies, architectural changes) MUST be justified with reference to constitution principles
- Constitution violations found during review MUST be resolved or explicitly waived with documented rationale

**Living Document**: This constitution evolves with project needs. Propose amendments via PR to `.specify/memory/constitution.md`.

**Version**: 1.0.0 | **Ratified**: 2025-10-25 | **Last Amended**: 2025-10-25
