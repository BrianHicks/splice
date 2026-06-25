## 1. Config Format Migration (Breaking Change)

- [ ] 1.1 Update `ModuleLocation` enum to support `Git { git: String, rev: Option<String>, path: Option<PathBuf> }` variant
- [ ] 1.2 Change config parsing from `[[module]]` array to `[module.<name>]` table syntax
- [ ] 1.3 Update `ModuleInvocation` to include `name: String` field derived from TOML key
- [ ] 1.4 Update all existing tests to use new `[module.<name>]` syntax
- [ ] 1.5 Add migration guide or error message for old `[[module]]` syntax

## 2. Git Module Source Implementation

- [ ] 2.1 Create `src/remote.rs` module for git operations
- [ ] 2.2 Implement `resolve_rev()` function to resolve tag/branch to SHA (tags first, then branches)
- [ ] 2.3 Implement `clone_repo()` function using `git clone --depth 1 --branch <rev>`
- [ ] 2.4 Implement `fetch_sha()` function using `git fetch --depth 1 origin <sha>`
- [ ] 2.5 Add error handling for git command failures with actionable messages
- [ ] 2.6 Implement default branch detection using `git ls-remote --symref`

## 3. Cache Layer

- [ ] 3.1 Create `src/cache.rs` module for cache management
- [ ] 3.2 Implement cache directory creation at `~/.cache/splice/`
- [ ] 3.3 Implement cache key generation: `hash(git_url, sha)` for directory naming
- [ ] 3.4 Implement `get_or_clone()` function to check cache before cloning
- [ ] 3.5 Implement `get_module_path()` to return path to module within cached clone (handling subpath)

## 4. Lockfile Handling

- [ ] 4.1 Define lockfile format: `[module.<name>]` with `sha = "..."` fields
- [ ] 4.2 Implement lockfile parsing in `src/lockfile.rs`
- [ ] 4.3 Implement lockfile writing/serialization
- [ ] 4.4 Implement lockfile validation during `splice sync` (auto-create if missing)
- [ ] 4.5 Implement transitive dependency lockfile reading for remote modules
- [ ] 4.6 Add error for missing lockfile in remote modules with actionable message

## 5. Module Collection with Remote Support

- [ ] 5.1 Update `collect_modules()` in `sync.rs` to handle `ModuleLocation::Git` variant
- [ ] 5.2 Implement BFS traversal that resolves git modules via cache layer
- [ ] 5.3 Add cycle detection for transitive dependencies
- [ ] 5.4 Implement lockfile-based SHA resolution (use locked SHA instead of re-resolving)
- [ ] 5.5 Update module loading to read from cached clone paths

## 6. Output Collision Detection

- [ ] 6.1 Modify `render_templates()` to collect all output paths before writing
- [ ] 6.2 Detect path collisions and collect all errors
- [ ] 6.3 Surface all collisions together in a single error message
- [ ] 6.4 Update existing tests to verify collision detection

## 7. CLI Commands

- [ ] 7.1 Add `module` subcommand group to CLI in `main.rs`
- [ ] 7.2 Implement `splice module lock` command to create/update lockfiles
- [ ] 7.3 Implement `splice module update [name]` command to re-resolve SHAs
- [ ] 7.4 Add error handling for `update` with non-existent module name
- [ ] 7.5 Add output messages showing what was locked/updated

## 8. Integration and Testing

- [ ] 8.1 Create integration test for fetching module from public git repo
- [ ] 8.2 Create integration test for transitive dependencies
- [ ] 8.3 Create integration test for lockfile creation and validation
- [ ] 8.4 Create integration test for cache hits (same module fetched twice)
- [ ] 8.5 Create integration test for output collision detection
- [ ] 8.6 Update existing CLI tests for new config format
- [ ] 8.7 Test error messages for missing lockfiles, cycles, and invalid revs

## 9. Documentation

- [ ] 9.1 Update README with remote module examples
- [ ] 9.2 Document lockfile format and workflow
- [ ] 9.3 Document CLI commands (`splice module lock`, `splice module update`)
- [ ] 9.4 Add migration guide for config format change
