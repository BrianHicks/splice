## ADDED Requirements

### Requirement: Module names via TOML key structure

The system SHALL use `[module.<name>]` syntax in `splice.toml` to enforce unique module names at the TOML parser level.

#### Scenario: Valid module name syntax
- **WHEN** `splice.toml` contains `[module.foo]` with module configuration
- **THEN** the system SHALL parse this as a module named "foo" with the specified configuration

#### Scenario: Multiple modules with unique names
- **WHEN** `splice.toml` contains `[module.foo]` and `[module.bar]`
- **THEN** the system SHALL parse both modules with their respective names

#### Scenario: Duplicate module names rejected by TOML
- **WHEN** `splice.toml` attempts to define `[module.foo]` twice
- **THEN** the TOML parser SHALL reject the file with a duplicate key error before splice processes it

### Requirement: Required module names

All modules SHALL have names. The `[module.<name>]` syntax makes names mandatory.

#### Scenario: Module without name is impossible
- **WHEN** user attempts to define a module without a name
- **THEN** the TOML syntax `[module.<name>]` requires a name, making unnamed modules syntactically invalid

#### Scenario: Empty module name rejected
- **WHEN** `splice.toml` contains `[module.]` (empty name)
- **THEN** the TOML parser SHALL reject the file as invalid syntax

### Requirement: Module name in error messages

The system SHALL use module names in error messages for clarity.

#### Scenario: Error referencing module by name
- **WHEN** module "foo" fails to load due to missing templates directory
- **THEN** the system SHALL produce an error: "Module 'foo': templates directory not found at <path>"

#### Scenario: Error referencing transitive dependency
- **WHEN** module "foo" depends on module "bar" which has a missing lockfile
- **THEN** the system SHALL produce an error: "Module 'bar' (dependency of 'foo'): does not have a splice.lock file..."

### Requirement: Breaking change from array to table syntax

The system SHALL migrate from `[[module]]` array syntax to `[module.<name>]` table syntax.

#### Scenario: Old array syntax no longer supported
- **WHEN** `splice.toml` contains `[[module]]` with `path = "./foo"`
- **THEN** the system SHALL produce a parsing error indicating the old syntax is no longer supported

#### Scenario: Migration path
- **WHEN** user has existing config with `[[module]]` syntax
- **THEN** the user MUST update to `[module.<name>]` syntax, adding explicit names for each module

#### Scenario: Local module with new syntax
- **WHEN** user wants to define a local module named "my-module" at path "./modules/my-module"
- **THEN** the config SHALL be:
  ```toml
  [module.my-module]
  path = "./modules/my-module"
  ```

#### Scenario: Remote module with new syntax
- **WHEN** user wants to define a remote module named "foo" from git
- **THEN** the config SHALL be:
  ```toml
  [module.foo]
  git = "https://github.com/user/repo"
  rev = "v1.0"
  ```
