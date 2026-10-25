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
        self.validate_name(parent, name)?;
        self.repository.create_dir(parent, name.trim())
    }

    /// Rename an entry inside `parent`; reject empty and path-like names.
    pub fn rename(&self, parent: &str, from: &str, to: &str) -> Result<(), ApplicationError> {
        self.validate_name(parent, to)?;
        let to = to.trim();
        if to.is_empty() {
            return Err(Self::invalid_name_error(parent, to));
        }
        self.repository.rename(parent, from, to)
    }

    /// Recursively remove an entry inside `parent`; path-like names rejected.
    pub fn remove(&self, parent: &str, name: &str) -> Result<(), ApplicationError> {
        self.validate_name(parent, name)?;
        self.repository.remove(parent, name)
    }

    fn validate_name(&self, parent: &str, name: &str) -> Result<(), ApplicationError> {
        let name = name.trim();
        if name.is_empty()
            || name == "."
            || name == ".."
            || name.contains('/')
            || name.contains('\\')
        {
            return Err(Self::invalid_name_error(parent, name));
        }
        Ok(())
    }

    fn invalid_name_error(parent: &str, name: &str) -> ApplicationError {
        ApplicationError::InvalidManifest {
            path: parent.to_string(),
            message: format!("invalid entry name: '{name}'"),
        }
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

    fn expect_no_write(repo: &mut MockExplorerRepository) {
        repo.expect_create_dir().never();
        repo.expect_rename().never();
        repo.expect_remove().never();
    }

    #[test]
    fn create_dir_rejects_invalid_names() {
        let mut repo = MockExplorerRepository::new();
        expect_no_write(&mut repo);
        let service = ExplorerService::new(Box::new(repo));
        assert!(service.create_dir("/x", "  ").is_err());
        assert!(service.create_dir("/x", "a/b").is_err());
        assert!(service.create_dir("/x", "a\\b").is_err());
        assert!(service.create_dir("/x", "..").is_err());
        assert!(service.create_dir("/x", ".").is_err());
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

    #[test]
    fn rename_validates_target_and_delegates() {
        let mut repo = MockExplorerRepository::new();
        repo.expect_rename()
            .with(
                mockall::predicate::eq("/x"),
                mockall::predicate::eq("old"),
                mockall::predicate::eq("renamed"),
            )
            .returning(|_, _, _| Ok(()));
        let service = ExplorerService::new(Box::new(repo));
        assert!(service.rename("/x", "old", " renamed ").is_ok());

        let mut repo = MockExplorerRepository::new();
        expect_no_write(&mut repo);
        let service = ExplorerService::new(Box::new(repo));
        assert!(service.rename("/x", "old", "../escape").is_err());
        assert!(service.rename("/x", "old", "  ").is_err());
    }

    #[test]
    fn remove_validates_name_and_delegates() {
        let mut repo = MockExplorerRepository::new();
        repo.expect_remove()
            .with(mockall::predicate::eq("/x"), mockall::predicate::eq("dir"))
            .returning(|_, _| Ok(()));
        let service = ExplorerService::new(Box::new(repo));
        assert!(service.remove("/x", "dir").is_ok());

        let mut repo = MockExplorerRepository::new();
        expect_no_write(&mut repo);
        let service = ExplorerService::new(Box::new(repo));
        assert!(service.remove("/x", "..").is_err());
        assert!(service.remove("/x", "/etc").is_err());
    }
}
