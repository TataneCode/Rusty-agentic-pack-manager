# Rules — Infrastructure layer (`src/03_infrastructure/`)

## Scope

`std::fs` and `toml` adapters implementing the application repository traits; manifest and filesystem mappers.

## Rules

1. **Adapters implement application traits** — `FsPackRepository`, `FsPackContentRepository`, `FsExplorer`, `FsInstallDestination`. This is the only layer that touches the filesystem and `toml` parsing.
2. **Errors: mapped to `ApplicationError`** — `Io { path, source }`, `InvalidManifest`, `NotADirectory` carry the failing path.
3. **Malformed pack dirs are skipped** — a directory without `pack.toml` is ignored in listings, not an error.
4. **Stateless mappers** — manifest → entity conversion in associated functions.
5. **Constructed at the DI point** (`main.rs`) — the rest of the code depends on traits only.
6. **Tests on tempdirs** — `tempfile::TempDir` fixtures; no hardcoded system paths.
7. **Copy semantics** — recursive tree copy creates parent directories; existing destinations are never silently overwritten.

## Forbidden anti-patterns

- Exposing a `toml` or raw `std::fs` type out of an adapter (returns are always domain entities or DTOs via the application).
- Containing business logic — an adapter translates, it does not decide.
- Importing `04_presentation` or touching rendering.
- Writing outside the target root or the packs directory.
