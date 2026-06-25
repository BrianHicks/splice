## ADDED Requirements

### Requirement: Lockfile format

The system SHALL use `splice.lock` files to pin `(git_url, rev) → sha` mappings for reproducible builds.

#### Scenario: Lockfile structure for direct modules
- **WHEN** an app has `[module.foo]` with `git = "https://github.com/user/A"` and `rev = "v1"`
- **THEN** the `splice.lock` file SHALL contain `[module.foo]` with `sha = "<resolved-sha>"`

#### Scenario: Lockfile pins exact commit
- **WHEN** `splice.lock` contains `[module.foo]` with `sha = "abc123..."`
- **THEN** the system SHALL fetch the module at exactly that commit SHA, regardless of what `rev` points to in the remote

### Requirement: Module authors must commit lockfiles

Remote modules SHALL include a `splice.lock` file in their repository. Missing lockfiles SHALL produce a hard error.

#### Scenario: Remote module with lockfile
- **WHEN** a remote module's repository contains both `splice.toml` and `splice.lock`
- **THEN** the system SHALL read the lockfile and use the pinned SHAs for transitive dependencies

#### Scenario: Remote module without lockfile
- **WHEN** a remote module's repository contains `splice.toml` but no `splice.lock`
- **THEN** the system SHALL produce an error: "Module '<name>' (git: <url> at <rev>) does not have a splice.lock file. Remote modules must include a lockfile to ensure reproducible builds. If you are the module author, run `splice module lock` in the module repository to generate a lockfile, then commit it."

### Requirement: Lockfile creation via splice module lock

The `splice module lock` command SHALL create or update a `splice.lock` file by resolving all `rev` values to SHAs.

#### Scenario: Create lockfile for app
- **WHEN** user runs `splice module lock` in a directory with `splice.toml` but no `splice.lock`
- **THEN** the system SHALL resolve all module `rev` values to SHAs and write `splice.lock`

#### Scenario: Update lockfile for app
- **WHEN** user runs `splice module lock` in a directory with both `splice.toml` and `splice.lock`
- **THEN** the system SHALL re-resolve all module `rev` values to current SHAs and update `splice.lock`

#### Scenario: Create lockfile for module
- **WHEN** user runs `splice module lock` in a module repository (with `type = "module"` in `splice.toml`)
- **THEN** the system SHALL resolve all transitive dependencies and write `splice.lock` including all direct module dependencies

### Requirement: Lockfile update via splice module update

The `splice module update [name]` command SHALL re-resolve specific or all modules to their latest SHAs.

#### Scenario: Update all modules
- **WHEN** user runs `splice module update` with no arguments
- **THEN** the system SHALL re-resolve all module `rev` values to current SHAs and update `splice.lock`

#### Scenario: Update specific module
- **WHEN** user runs `splice module update foo`
- **THEN** the system SHALL re-resolve only module `foo`'s `rev` to the current SHA and update `splice.lock`

#### Scenario: Update non-existent module
- **WHEN** user runs `splice module update nonexistent`
- **THEN** the system SHALL produce an error indicating the module was not found

### Requirement: Lockfile validation during sync

The `splice sync` command SHALL use lockfiles for reproducible builds and auto-create them if missing.

#### Scenario: Sync with existing lockfile
- **WHEN** user runs `splice sync` and `splice.lock` exists
- **THEN** the system SHALL use the SHAs from the lockfile to fetch modules, ignoring remote changes

#### Scenario: Sync without lockfile
- **WHEN** user runs `splice sync` and no `splice.lock` exists
- **THEN** the system SHALL auto-create `splice.lock` by resolving all `rev` values to SHAs, then proceed with sync

#### Scenario: Sync with stale lockfile
- **WHEN** user runs `splice sync` and `splice.lock` exists but a module in `splice.toml` is not in the lockfile
- **THEN** the system SHALL add the missing module to the lockfile and proceed with sync
