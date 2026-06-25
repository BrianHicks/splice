## ADDED Requirements

### Requirement: Git repository as module source

The system SHALL support fetching modules from git repositories using the `git`, `rev`, and optional `path` fields in `splice.toml`.

#### Scenario: Fetch module from git repository with tag
- **WHEN** `splice.toml` contains `[module.foo]` with `git = "https://github.com/user/repo"` and `rev = "v1.0"`
- **THEN** the system SHALL clone the repository at tag `v1.0` and load the module from the repository root

#### Scenario: Fetch module from git repository with branch
- **WHEN** `splice.toml` contains `[module.foo]` with `git = "https://github.com/user/repo"` and `rev = "main"`
- **THEN** the system SHALL clone the repository at the latest commit on branch `main` and load the module from the repository root

#### Scenario: Fetch module from subpath within git repository
- **WHEN** `splice.toml` contains `[module.foo]` with `git = "https://github.com/user/repo"`, `rev = "v1.0"`, and `path = "modules/bar"`
- **THEN** the system SHALL clone the repository at tag `v1.0` and load the module from the `modules/bar` subdirectory

#### Scenario: Git URL without rev assumes default branch
- **WHEN** `splice.toml` contains `[module.foo]` with `git = "https://github.com/user/repo"` and no `rev` field
- **THEN** the system SHALL clone the repository at the default branch (as reported by `git ls-remote --symref`)

#### Scenario: Rev without git URL is an error
- **WHEN** `splice.toml` contains `[module.foo]` with `rev = "v1.0"` but no `git` field
- **THEN** the system SHALL produce an error indicating that `rev` requires `git` to be specified

### Requirement: Rev resolution follows git convention

The system SHALL resolve `rev` values by checking tags first, then branches, following git's standard convention.

#### Scenario: Rev matches both tag and branch
- **WHEN** `rev = "v1.0"` and both a tag `v1.0` and branch `v1.0` exist in the repository
- **THEN** the system SHALL use the tag `v1.0`

#### Scenario: Rev matches only branch
- **WHEN** `rev = "main"` and no tag `main` exists but branch `main` exists
- **THEN** the system SHALL use branch `main`

#### Scenario: Rev matches neither tag nor branch
- **WHEN** `rev = "nonexistent"` and no tag or branch with that name exists
- **THEN** the system SHALL produce an error indicating the rev could not be resolved

### Requirement: Module caching

The system SHALL cache git clones at `~/.cache/splice/<hash>/` where the hash is derived from `(git_url, sha)`.

#### Scenario: Cache hit for same URL and SHA
- **WHEN** a module with `git = "https://github.com/user/repo"` and resolved `sha = "abc123"` is requested and the cache already contains a clone at `~/.cache/splice/<hash(url, abc123)>/`
- **THEN** the system SHALL use the cached clone without fetching from the remote

#### Scenario: Cache miss for new SHA
- **WHEN** a module with `git = "https://github.com/user/repo"` and resolved `sha = "def456"` is requested and the cache does not contain a clone for that SHA
- **THEN** the system SHALL clone the repository to `~/.cache/splice/<hash(url, def456)>/` using a shallow clone (`--depth 1`)

#### Scenario: Multiple modules share same cache entry
- **WHEN** two modules reference the same `(git_url, sha)` but different `path` subpaths
- **THEN** the system SHALL use the same cached clone for both modules, loading from different subdirectories

### Requirement: Transitive dependencies

The system SHALL support modules that include other remote modules in their own `splice.toml`.

#### Scenario: Module includes remote dependency
- **WHEN** module `foo` (from git) has its own `splice.toml` with `[module.bar]` pointing to another git repository
- **THEN** the system SHALL fetch module `bar` and include it in the render pipeline

#### Scenario: Detect dependency cycles
- **WHEN** module `A` depends on module `B` and module `B` depends on module `A`
- **THEN** the system SHALL produce an error indicating a dependency cycle was detected

### Requirement: Authentication delegation

The system SHALL delegate authentication to the user's git configuration (SSH agent, credential helpers).

#### Scenario: Private repository with SSH
- **WHEN** a module specifies `git = "git@github.com:user/private-repo.git"`
- **THEN** the system SHALL invoke `git clone` which will use the user's SSH agent for authentication

#### Scenario: Private repository with HTTPS credentials
- **WHEN** a module specifies `git = "https://github.com/user/private-repo.git"` and the user has git credential helpers configured
- **THEN** the system SHALL invoke `git clone` which will use the configured credential helpers
