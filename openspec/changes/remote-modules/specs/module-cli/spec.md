## ADDED Requirements

### Requirement: splice module lock command

The system SHALL provide a `splice module lock` command to create or update lockfiles.

#### Scenario: Lock command creates lockfile
- **WHEN** user runs `splice module lock` in a directory with `splice.toml` containing remote modules
- **THEN** the system SHALL create `splice.lock` with resolved SHAs for all modules

#### Scenario: Lock command updates existing lockfile
- **WHEN** user runs `splice module lock` and `splice.lock` already exists
- **THEN** the system SHALL update all SHA values in the lockfile to current resolutions

#### Scenario: Lock command output
- **WHEN** user runs `splice module lock` successfully
- **THEN** the system SHALL print a summary of modules locked, including module names and resolved SHAs

### Requirement: splice module update command

The system SHALL provide a `splice module update [name]` command to re-resolve module versions.

#### Scenario: Update all modules
- **WHEN** user runs `splice module update` with no arguments
- **THEN** the system SHALL re-resolve all modules in `splice.lock` to their latest SHAs based on their `rev` values

#### Scenario: Update specific module
- **WHEN** user runs `splice module update foo`
- **THEN** the system SHALL re-resolve only module `foo` to its latest SHA

#### Scenario: Update with no lockfile
- **WHEN** user runs `splice module update` and no `splice.lock` exists
- **THEN** the system SHALL produce an error: "No splice.lock file found. Run `splice module lock` first to create a lockfile."

#### Scenario: Update output
- **WHEN** user runs `splice module update` successfully
- **THEN** the system SHALL print which modules were updated, showing old and new SHAs

### Requirement: CLI error messages

The system SHALL provide clear, actionable error messages for CLI commands.

#### Scenario: Update non-existent module
- **WHEN** user runs `splice module update nonexistent` and no module with that name exists
- **THEN** the system SHALL produce an error: "Module 'nonexistent' not found in splice.lock. Available modules: foo, bar, baz"

#### Scenario: Lock command with invalid config
- **WHEN** user runs `splice module lock` and `splice.toml` has syntax errors
- **THEN** the system SHALL produce an error with the parsing error details and file location
