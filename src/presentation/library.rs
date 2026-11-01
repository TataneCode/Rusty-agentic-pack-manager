use std::path::{Path, PathBuf};

/// System-wide pack library shipped by the RPM/DEB packages.
const SYSTEM_PACKS_DIR: &str = "/usr/share/rusty-agentic-pack-manager/packs";
/// System-wide CLI profiles shipped alongside the packs (E02).
const SYSTEM_CLIS_DIR: &str = "/usr/share/rusty-agentic-pack-manager/clis";

/// The pack library: an ordered list of read roots (user library first)
/// plus the writable root where new packs are created.
///
/// Read order: user library → `packs` next to the binary → `packs` in the
/// cwd → the system location. The first root containing a pack wins, so a
/// user pack overrides a system pack of the same name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackLibrary {
    pub roots: Vec<PathBuf>,
    /// Writable root: new packs are always created here.
    pub user_root: PathBuf,
    /// Read roots of the CLI profiles (`clis/<id>.toml`, E02), same order
    /// and priority as `roots` (user library first).
    pub profile_roots: Vec<PathBuf>,
}

impl PackLibrary {
    pub fn new(
        roots: Vec<PathBuf>,
        user_root: PathBuf,
        profile_roots: Vec<PathBuf>,
    ) -> PackLibrary {
        PackLibrary {
            roots,
            user_root,
            profile_roots,
        }
    }

    /// Discover the library from the environment. The user root is always
    /// the first root. Creation of the user root is delegated to the
    /// `LibraryInitializer` port (E01: no direct I/O in the presentation).
    pub fn discover() -> Option<PackLibrary> {
        let user_root = user_packs_dir()?;
        let roots = resolve_roots(
            &user_root,
            exe_dir().as_deref(),
            cwd().as_deref(),
            Path::new(SYSTEM_PACKS_DIR),
        );
        let profile_roots = resolve_profile_roots(
            &user_root,
            exe_dir().as_deref(),
            cwd().as_deref(),
            Path::new(SYSTEM_CLIS_DIR),
        );
        Some(PackLibrary {
            roots,
            user_root,
            profile_roots,
        })
    }
}

/// User pack library: `$XDG_DATA_HOME/rapm/packs`, else
/// `$HOME/.local/share/rapm/packs`, else `%APPDATA%\rapm\packs` on Windows.
fn user_packs_dir() -> Option<PathBuf> {
    user_packs_dir_from(
        std::env::var_os("XDG_DATA_HOME").as_deref(),
        std::env::var_os("HOME").as_deref(),
        std::env::var_os("APPDATA").as_deref(),
    )
}

/// User CLI profile library: the `clis` directory next to the user packs
/// (E02), so a user profile overrides a shipped one of the same name.
fn user_clis_dir(user_packs_root: &Path) -> PathBuf {
    user_packs_root
        .parent()
        .map(|p| p.join("clis"))
        .unwrap_or_else(|| user_packs_root.join("clis"))
}

#[cfg_attr(not(windows), allow(unused_variables))]
fn user_packs_dir_from(
    xdg_data_home: Option<&std::ffi::OsStr>,
    home: Option<&std::ffi::OsStr>,
    appdata: Option<&std::ffi::OsStr>,
) -> Option<PathBuf> {
    if let Some(xdg) = xdg_data_home {
        if !xdg.is_empty() {
            return Some(Path::new(xdg).join("rapm").join("packs"));
        }
    }
    #[cfg(windows)]
    if let Some(appdata) = appdata {
        if !appdata.is_empty() {
            return Some(Path::new(appdata).join("rapm").join("packs"));
        }
    }
    let home = home?;
    if home.is_empty() {
        return None;
    }
    Some(
        Path::new(home)
            .join(".local")
            .join("share")
            .join("rapm")
            .join("packs"),
    )
}

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(Path::to_path_buf)
}

fn cwd() -> Option<PathBuf> {
    std::env::current_dir().ok()
}

/// Ordered, de-duplicated CLI profile roots: the user `clis` dir first,
/// then `clis` next to the binary, in the cwd, and the system location.
/// Same rule as `resolve_roots`, on the `clis` directory (E02).
fn resolve_profile_roots(
    user_packs_root: &Path,
    exe_dir: Option<&Path>,
    cwd: Option<&Path>,
    system: &Path,
) -> Vec<PathBuf> {
    let user = user_clis_dir(user_packs_root);
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(dir) = exe_dir {
        candidates.push(dir.join("clis"));
    }
    if let Some(dir) = cwd {
        candidates.push(dir.join("clis"));
    }
    candidates.push(system.to_path_buf());
    let mut roots: Vec<PathBuf> = vec![user.clone()];
    for candidate in candidates {
        if candidate.is_dir() && !roots.contains(&candidate) && candidate != user {
            roots.push(candidate);
        }
    }
    roots
}

/// Ordered, de-duplicated read roots: the user root first, then every
/// existing candidate location. Pure: takes the candidates as parameters
/// so tests never touch the real environment.
fn resolve_roots(
    user_root: &Path,
    exe_dir: Option<&Path>,
    cwd: Option<&Path>,
    system: &Path,
) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = vec![user_root.to_path_buf()];
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(dir) = exe_dir {
        candidates.push(dir.join("packs"));
    }
    if let Some(dir) = cwd {
        candidates.push(dir.join("packs"));
    }
    candidates.push(system.to_path_buf());
    for candidate in candidates {
        if candidate.is_dir() && !roots.contains(&candidate) {
            roots.push(candidate);
        }
    }
    roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_root_prefers_xdg_data_home() {
        let dir = user_packs_dir_from(
            Some(std::ffi::OsStr::new("/xdg")),
            Some(std::ffi::OsStr::new("/home/u")),
            Some(std::ffi::OsStr::new("C:/appdata")),
        )
        .unwrap();
        assert_eq!(dir, PathBuf::from("/xdg/rapm/packs"));
    }

    #[test]
    fn user_root_falls_back_to_home() {
        let dir = user_packs_dir_from(
            None,
            Some(std::ffi::OsStr::new("/home/u")),
            Some(std::ffi::OsStr::new("C:/appdata")),
        )
        .unwrap();
        assert_eq!(dir, PathBuf::from("/home/u/.local/share/rapm/packs"));
    }

    #[test]
    fn user_root_ignores_empty_env_values() {
        let dir = user_packs_dir_from(
            Some(std::ffi::OsStr::new("")),
            Some(std::ffi::OsStr::new("/home/u")),
            None,
        )
        .unwrap();
        assert_eq!(dir, PathBuf::from("/home/u/.local/share/rapm/packs"));
        assert!(user_packs_dir_from(None, None, None).is_none());
        assert!(user_packs_dir_from(None, Some(std::ffi::OsStr::new("")), None).is_none());
    }

    #[test]
    fn resolve_roots_keeps_user_root_first_and_only_existing_candidates() {
        let tmp = tempfile::TempDir::new().unwrap();
        let user_root = tmp.path().join("user-packs");
        let exe_packs = tmp.path().join("exe/packs");
        let system = tmp.path().join("system/packs");
        // Only the exe and system candidates exist.
        std::fs::create_dir_all(&exe_packs).unwrap();
        std::fs::create_dir_all(&system).unwrap();

        let roots = resolve_roots(
            &user_root,
            Some(tmp.path().join("exe").as_path()),
            Some(tmp.path().join("cwd").as_path()),
            &system,
        );
        // The user root is first even though it does not exist yet.
        assert_eq!(roots, vec![user_root, exe_packs, system]);
    }

    #[test]
    fn user_clis_dir_sits_next_to_the_user_packs() {
        assert_eq!(
            user_clis_dir(Path::new("/xdg/rapm/packs")),
            PathBuf::from("/xdg/rapm/clis")
        );
    }

    #[test]
    fn resolve_profile_roots_keeps_user_clis_first() {
        let tmp = tempfile::TempDir::new().unwrap();
        let user_packs = tmp.path().join("rapm/packs");
        let user_clis = tmp.path().join("rapm/clis");
        let exe_clis = tmp.path().join("exe/clis");
        let system = tmp.path().join("system/clis");
        std::fs::create_dir_all(&user_clis).unwrap();
        std::fs::create_dir_all(&exe_clis).unwrap();
        std::fs::create_dir_all(&system).unwrap();

        let roots = resolve_profile_roots(
            &user_packs,
            Some(tmp.path().join("exe").as_path()),
            Some(tmp.path().join("cwd").as_path()),
            &system,
        );
        // The user clis dir is first even though only it exists.
        assert_eq!(roots, vec![user_clis, exe_clis, system]);
    }

    #[test]
    fn resolve_roots_dedupes_identical_candidates() {
        let tmp = tempfile::TempDir::new().unwrap();
        let packs = tmp.path().join("packs");
        std::fs::create_dir_all(&packs).unwrap();
        let roots = resolve_roots(&packs, Some(tmp.path()), Some(tmp.path()), &packs);
        assert_eq!(roots, vec![packs]);
    }
}
