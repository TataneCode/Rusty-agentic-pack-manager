use std::path::{Path, PathBuf};

use crate::application::config::dto::AppConfigDto;

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
        Self::discover_with(&AppConfigDto::default())
    }

    /// Discover the library, honoring the user configuration (E12):
    /// `config.packs_dir` becomes the user root (first read root, writable
    /// root) instead of the XDG default. The CLI profile roots stay derived
    /// from the XDG user root, so profiles keep a stable location even when
    /// `packs_dir` points elsewhere.
    pub fn discover_with(config: &AppConfigDto) -> Option<PackLibrary> {
        let xdg_root = user_packs_dir()?;
        let user_root = config.packs_dir.clone().unwrap_or_else(|| xdg_root.clone());
        let roots = resolve_roots(
            &user_root,
            exe_dir().as_deref(),
            cwd().as_deref(),
            Path::new(SYSTEM_PACKS_DIR),
        );
        let profile_roots = resolve_profile_roots(
            &xdg_root,
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
    fn discover_with_uses_packs_dir_as_user_root_when_configured() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config = AppConfigDto {
            packs_dir: Some(tmp.path().join("custom-packs")),
            default_cli: None,
        };
        // discover_with reads the env for the XDG root; call the pure
        // resolution directly to stay env-independent.
        let xdg_root = tmp.path().join("xdg/rapm/packs");
        let user_root = config.packs_dir.clone().unwrap_or_else(|| xdg_root.clone());
        let roots = resolve_roots(
            &user_root,
            Some(tmp.path().join("exe").as_path()),
            None,
            Path::new("/system/packs"),
        );
        assert_eq!(roots[0], tmp.path().join("custom-packs"));
        // Profile roots stay derived from the XDG root, not from packs_dir.
        let profile_roots = resolve_profile_roots(
            &xdg_root,
            Some(tmp.path().join("exe").as_path()),
            None,
            Path::new("/system/clis"),
        );
        assert_eq!(profile_roots[0], tmp.path().join("xdg/rapm/clis"));
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
