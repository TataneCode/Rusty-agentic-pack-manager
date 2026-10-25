use crate::application::error::ApplicationError;
use crate::application::explorer::dto::DirectoryListingDto;
use crate::application::explorer::traits::ExplorerRepository;

/// Application service for the file-explorer panel.
pub struct ExplorerService {
    repository: Box<dyn ExplorerRepository>,
}

impl ExplorerService {
    pub fn new(repository: Box<dyn ExplorerRepository>) -> ExplorerService {
        ExplorerService { repository }
    }

    /// List a directory, sorted (dirs first, then names).
    pub fn list(&self, path: &str) -> Result<DirectoryListingDto, ApplicationError> {
        let mut listing = self.repository.list(path)?;
        listing.entries.sort_by(|a, b| {
            b.is_dir
                .cmp(&a.is_dir)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        Ok(listing)
    }

    /// Create a folder inside `parent`; reject empty and path-like names.
    pub fn create_dir(&self, parent: &str, name: &str) -> Result<(), ApplicationError> {
        let name = name.trim();
        if name.is_empty() || name.contains('/') || name.contains('\\') {
            return Err(ApplicationError::InvalidManifest {
                path: parent.to_string(),
                message: format!("invalid folder name: {name}"),
            });
        }
        self.repository.create_dir(parent, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::explorer::dto::FileNodeDto;
    use crate::application::explorer::traits::MockExplorerRepository;

    #[test]
    fn list_sorts_dirs_first_then_names() {
        let mut repo = MockExplorerRepository::new();
        repo.expect_list().returning(|_| {
            Ok(DirectoryListingDto {
                path: "/x".into(),
                entries: vec![
                    FileNodeDto {
                        name: "zdir".into(),
                        is_dir: true,
                    },
                    FileNodeDto {
                        name: "afile".into(),
                        is_dir: false,
                    },
                    FileNodeDto {
                        name: "bdir".into(),
                        is_dir: true,
                    },
                ],
            })
        });
        let service = ExplorerService::new(Box::new(repo));
        let listing = service.list("/x").unwrap();
        let names: Vec<&str> = listing.entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["bdir", "zdir", "afile"]);
    }

    #[test]
    fn create_dir_rejects_invalid_names() {
        let mut repo = MockExplorerRepository::new();
        repo.expect_create_dir().never();
        let service = ExplorerService::new(Box::new(repo));
        assert!(service.create_dir("/x", "  ").is_err());
        assert!(service.create_dir("/x", "a/b").is_err());
        assert!(service.create_dir("/x", "a\\b").is_err());
    }

    #[test]
    fn create_dir_trims_and_delegates() {
        let mut repo = MockExplorerRepository::new();
        repo.expect_create_dir()
            .with(mockall::predicate::eq("/x"), mockall::predicate::eq("new"))
            .returning(|_, _| Ok(()));
        let service = ExplorerService::new(Box::new(repo));
        assert!(service.create_dir("/x", " new ").is_ok());
    }
}
