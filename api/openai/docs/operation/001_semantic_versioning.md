# Operation: Semantic Versioning

### Scope

- **Purpose**: Documents the Semantic Versioning operation — classifying changes and applying the correct version increment to api_openai releases.
- **Responsibility**: Specifies the classification rules for MAJOR, MINOR, and PATCH increments and the steps to apply a version bump.
- **In Scope**: Change classification, version field update in Cargo.toml, release validation gate.
- **Out of Scope**: Changelog authoring, crates.io publishing mechanics, CI/CD automation, documentation versioning.

### Prerequisites

- All changes since the last release tag are identified and listed.
- Tests pass at Level 3 (`ctest3` — nextest, doc tests, clippy with pedantic lints).
- The working tree contains no uncommitted changes.

### Procedure Steps

#### Classify each change

1. Review all changes since the last release tag.
2. Assign each change one of three labels:
   - **MAJOR** — breaks backward compatibility: removes or renames public items; changes function signatures, error types, or error enum variants; removes support for a deprecated OpenAI API version; increases MSRV by more than 6 months.
   - **MINOR** — backward-compatible addition: new public functions, methods, types, or cargo features; new endpoint coverage; new optional fields in non-exhaustive structs; new non-exhaustive error variants.
   - **PATCH** — backward-compatible fix: corrects incorrect behavior, fixes error messages, refactors internals without changing the public surface, updates internal dependencies without API impact.
3. The highest label among all changes determines the version increment.

#### Apply the version increment

4. Update the `version` field in `api/openai/Cargo.toml` according to the determined increment: MAJOR resets MINOR and PATCH to zero; MINOR resets PATCH to zero; PATCH increments only the third digit.
5. If workspace-level dependency entries reference this crate by exact version, update them to match.

#### Validate the release

6. Run `ctest3` on the updated crate. Confirm zero warnings, zero clippy findings, and all integration tests pass with real API credentials.

### Expected Outcome

`Cargo.toml` reflects the new semver string. All tests pass with pedantic clippy enabled. No API surface regressions for MINOR or PATCH increments.

### Rollback Procedure

If a published version is found to contain a regression, publish a corrective PATCH release. Yanking a version from crates.io is reserved for security vulnerabilities — prefer a fix release.
