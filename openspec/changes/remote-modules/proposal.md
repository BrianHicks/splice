## Why

Splice modules are currently local-only (`path = "./my-module"`), which limits reusability and sharing across projects. Users need to pull modules from remote git repositories with proper version pinning and caching to build a composable module ecosystem.

## What Changes

- Add git-based module sources with `git`, `rev`, and optional `path` (subpath within repo)
- **BREAKING**: Change config format from `[[module]]` to `[module.<name>]` to enforce unique names
- Require explicit `name` field for all modules (enforced by TOML key structure)
- Introduce `splice.lock` lockfiles that pin `(git_url, rev) → sha` for reproducible builds
- Module authors must commit `splice.lock` files; missing lockfiles produce hard errors
- Add caching layer at `~/.cache/splice/` keyed by `(git_url, sha)` — one clone per pair
- Add `splice module lock` command to create/update lockfiles
- Add `splice module update [name]` command to re-resolve revs to latest SHAs
- Detect output path collisions across modules and report as errors (collect all, surface together)
- Support transitive dependencies: modules can include other remote modules
- Rev resolution follows git convention: tags first, then branches

## Capabilities

### New Capabilities

- `git-module-source`: Fetching modules from git repositories with caching, rev resolution (tags/branches), and subpath support
- `module-lockfiles`: Lockfile format, creation via `splice module lock`, update via `splice module update`, and validation during sync
- `module-cli`: New CLI commands for module management (`splice module lock`, `splice module update [name]`)
- `module-naming`: Required module names via `[module.<name>]` config format, uniqueness enforcement, and error messages

### Modified Capabilities

(None — no existing specs to modify)

## Impact

- **Code**: `config.rs` (ModuleLocation enum, config parsing), `sync.rs` (module collection, collision detection), `module.rs` (loading from cache), new `cache.rs` or `remote.rs` module
- **Dependencies**: Add `git2` crate or shell out to `git` CLI for cloning/fetching
- **CLI**: New `module` subcommand group with `lock` and `update` subcommands
- **Filesystem**: New `splice.lock` files in app and module repos, cache directory at `~/.cache/splice/`
- **Config format**: Breaking change from `[[module]]` to `[module.<name>]`
