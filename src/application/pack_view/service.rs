use crate::application::error::ApplicationError;
use crate::application::pack_view::traits::{PackContentRepository, PackFileDto};

/// Application service backing the pack viewer ('v' key).
pub struct PackViewService {
    repository: Box<dyn PackContentRepository>,
}

impl PackViewService {
    pub fn new(repository: Box<dyn PackContentRepository>) -> PackViewService {
        PackViewService { repository }
    }

    /// Files viewable in a pack: only text files, sorted with the
    /// manifest first (the viewer opens on pack.toml).
    pub fn list_viewable(&self, pack_name: &str) -> Result<Vec<PackFileDto>, ApplicationError> {
        let mut files = self.repository.list_files(pack_name)?;
        files.retain(|f| f.is_text);
        files.sort_by(
            |a, b| match (a.path == "pack.toml", b.path == "pack.toml") {
                (true, true) | (false, false) => a.path.cmp(&b.path),
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
            },
        );
        Ok(files)
    }

    pub fn read(&self, pack_name: &str, path: &str) -> Result<String, ApplicationError> {
        if path.contains("..") || path.starts_with('/') {
            return Err(ApplicationError::FileNotFound(path.to_string()));
        }
        self.repository.read_file(pack_name, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::pack_view::traits::MockPackContentRepository;

    fn file(path: &str, is_text: bool) -> PackFileDto {
        PackFileDto {
            path: path.to_string(),
            is_text,
        }
    }

    #[test]
    fn list_viewable_filters_non_text_and_sorts() {
        let mut repo = MockPackContentRepository::new();
        repo.expect_list_files().returning(|_| {
            Ok(vec![
                file("z/SKILL.md", true),
                file("logo.png", false),
                file("a/SKILL.md", true),
            ])
        });
        let service = PackViewService::new(Box::new(repo));
        let files = service.list_viewable("tdd").unwrap();
        assert_eq!(
            files.iter().map(|f| f.path.clone()).collect::<Vec<_>>(),
            vec!["a/SKILL.md", "z/SKILL.md"]
        );
        // with the manifest present, it sorts first
        let mut repo = MockPackContentRepository::new();
        repo.expect_list_files()
            .returning(|_| Ok(vec![file("z/SKILL.md", true), file("pack.toml", true)]));
        let service = PackViewService::new(Box::new(repo));
        let files = service.list_viewable("tdd").unwrap();
        assert_eq!(files[0].path, "pack.toml");
    }

    #[test]
    fn read_rejects_escaping_paths() {
        let mut repo = MockPackContentRepository::new();
        repo.expect_read_file().never();
        let service = PackViewService::new(Box::new(repo));
        assert!(service.read("tdd", "../outside.md").is_err());
        assert!(service.read("tdd", "/etc/passwd").is_err());
    }

    #[test]
    fn read_delegates_to_repository() {
        let mut repo = MockPackContentRepository::new();
        repo.expect_read_file()
            .with(
                mockall::predicate::eq("tdd"),
                mockall::predicate::eq("a/SKILL.md"),
            )
            .returning(|_, _| Ok("# TDD".to_string()));
        let service = PackViewService::new(Box::new(repo));
        assert_eq!(service.read("tdd", "a/SKILL.md").unwrap(), "# TDD");
    }
}
