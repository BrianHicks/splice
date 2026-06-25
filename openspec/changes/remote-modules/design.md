## Context

Splice is a template-based config file synchronizer that reads `splice.toml` configs, loads modules (directories containing templates), renders them with Tera, and writes output files. Currently, modules are local-only (`path = "./my-module"`).

The system needs to support remote git repositories as module sources with proper version pinning, caching, and transitive dependencies to enable a composable module ecosystem.

## Goals / Non-Goals

**Goals:**
- Fetch modules from git repositories with reproducible builds via lockfiles
- Cache git clones efficiently (one clone per `(git_url, sha)` pair)
- Support transitive dependencies (modules can include other remote modules)
- Detect and report output path collisions as errors
- Provide CLI commands for lockfile management (`splice module lock`, `splice module update`)
- Require module authors to commit lockfiles for consistency

**Non-Goals:**
- Authentication handling (delegate to git's SSH agent / credential helpers)
- Version range resolution or constraint satisfaction (modules pin exact revs)
- HTTP/HTTPS module sources (git only for now)
- Lockfile flattening across module boundaries (each module has its own lockfile)
- CLI validation on `module add` (defer to `splice sync`)

## Decisions

### 1. Config format: `[module.<name>]` instead of `[[module]]`

**Decision**: Use TOML table syntax with the module name as the key.

**Rationale**: 
- Enforces uniqueness at the TOML parser level (can't have duplicate keys)
- Cleaner error messages ("module 'foo' not found" vs "module at index 2")
- Mirrors established patterns (Cargo's `[dependencies.name]`, npm)
- Simpler than array-of-tables with explicit `name` fields

**Alternatives considered**:
- `[[module]]` with `name = "..."` field: More verbose, requires manual uniqueness validation
- Identify by `(git_url, path)`: Awkward for CLI commands, no human-friendly identifier

### 2. Lockfile strategy: Module-authored lockfiles

**Decision**: Each module (app or remote) has its own `splice.lock` that pins its direct dependencies. App lockfiles only track direct modules, not transitive deps.

**Rationale**:
- Module authors control consistency of their dependencies
- Simpler app-level code (no transitive resolution logic)
- Avoids chicken-and-egg problem (modules must work without app lockfiles)
- Explicit: app owner sees exactly what they're pulling, module author controls their deps
- Easier to reason about: each lockfile is small and focused

**Alternatives considered**:
- Flatten all transitive deps into app lockfile: More complex, harder to push changes back to modules, app owner has too much control
- Require lockfiles only at app level: Module authors can't guarantee consistency

### 3. Cache key: `(git_url, sha)`

**Decision**: Cache clones at `~/.cache/splice/<hash(git_url, sha)>/` where the hash is derived from both URL and resolved SHA.

**Rationale**:
- Same commit from different URLs doesn't collide (security)
- Same repo at different revs gets separate checkouts (correctness)
- Multiple modules can reference different subpaths within the same clone (efficiency)
- SHA is immutable, so cache never needs invalidation

**Alternatives considered**:
- Cache by `(git_url, rev)`: Problematic because tags/branches can move, would need invalidation logic
- Cache by SHA only: Different repos could collide

### 4. Rev resolution: Tags first, then branches

**Decision**: When resolving `rev = "v1.0"`, check git tags first, then branches. Follow git's convention.

**Rationale**:
- Principle of least surprise for git users
- Tags are typically immutable releases, branches are mutable
- Git's own tools (e.g., `git checkout`) follow this order

### 5. Output collision detection: Hard error, collect all

**Decision**: Collect all output paths before writing, error if any collisions exist, report all collisions together.

**Rationale**:
- Silent data loss is worse than a loud failure
- Collecting all errors at once avoids whack-a-mole (fix one, run again, find another)
- Current "last writer wins" behavior is unpredictable and hard to debug

### 6. Git implementation: Shell out to `git` CLI

**Decision**: Use `std::process::Command` to invoke `git` rather than `git2` crate.

**Rationale**:
- Simpler dependency (no C library linking)
- Leverages user's git configuration (SSH keys, credential helpers, proxies)
- Easier to debug (users can run the same git commands)
- Performance is acceptable for this use case (not high-frequency)

**Alternatives considered**:
- `git2` crate: More control, but adds complexity and linking issues

### 7. Shallow clones

**Decision**: Use `git clone --depth 1 --branch <rev>` for initial clones, `git fetch --depth 1 origin <sha>` for updates.

**Rationale**:
- Faster than full clones (most repos are large, we only need one commit)
- Sufficient for our use case (we never need history)
- Reduces cache size

**Alternatives considered**:
- Full clones: Slower, more disk space, no benefit for our use case

## Risks / Trade-offs

**[Risk] Module authors forget to commit lockfiles**
→ Mitigation: Clear error message with actionable fix ("run `splice module lock` in the module repo")

**[Risk] Cache grows unbounded over time**
→ Mitigation: Document cache location, provide `splice cache clean` command in future (out of scope for now)

**[Risk] Transitive dependency cycles (A → B → A)**
→ Mitigation: Detect cycles during BFS traversal, error with clear message

**[Risk] Tag/branch namespacing conflicts (tag "v1" and branch "v1" both exist)**
→ Mitigation: Follow git convention (tags first), document this behavior

**[Risk] Shallow clones don't support all git operations**
→ Mitigation: We only need to read files, never need history or complex operations

**[Trade-off] Module-authored lockfiles limit app owner control**
→ App owner can fork the module or override with local path if needed. This is acceptable for the simplicity gain.

**[Trade-off] No version ranges means no automatic conflict resolution**
→ This is intentional. Splice modules are templates, not libraries. Explicit pinning is more predictable.

## Migration Plan

**Breaking change**: Config format changes from `[[module]]` to `[module.<name>]`.

**Migration steps**:
1. Update existing `splice.toml` files to use `[module.<name>]` syntax
2. Add `name` field if not already present (required for all modules)
3. For local modules: `[module.my-module]` with `path = "./my-module"`
4. Run `splice sync` to verify everything still works

**Rollback**: Not applicable (config format change is one-way). Users can keep old configs if they don't upgrade.

**Timeline**: This is a breaking change. Bump major version when releasing.

## Open Questions

None at this time. All major decisions have been resolved through exploration.
