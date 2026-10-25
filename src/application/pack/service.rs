use std::path::{Path, PathBuf};

use crate::application::error::ApplicationError;
use crate::application::pack::dto::{InstallResultDto, PackSummaryDto};
use crate::application::pack::traits::{InstallDestination, PackRepository};
use crate::domain::pack::entity::Pack;
use crate::domain::pack::error::plan_install;
use crate::domain::pack::value_objects::TargetCli;

/// Application service for the packs panel: list, and install into a target.
pub struct PackService {
    repository: Box<dyn PackRepository>,
    destination: Box<dyn InstallDestination>,
}

impl PackService {
    pub fn new(
        repository: Box<dyn PackRepository>,
        destination: Box<dyn InstallDestination>,
    ) -> PackService {
        PackService {
            repository,
            destination,
        }
    }

    /// All packs, sorted by CLI then name for UI grouping.
    pub fn list(&self) -> Result<Vec<PackSummaryDto>, ApplicationError> {
        let mut packs = self.repository.list()?;
        packs.sort_by(|a, b| {
            a.clis
                .cmp(&b.clis)
                .then_with(|| a.name.as_str().cmp(b.name.as_str()))
        });
        Ok(packs.iter().map(PackSummaryDto::from_pack).collect())
    }

    pub fn find(&self, name: &str) -> Result<Pack, ApplicationError> {
        self.repository.find(name)
    }

    /// Install `pack_name` for `cli` inside `target_root`. The domain install
    /// plan is computed first (no partial writes on validation errors);
    /// existing destinations abort the install unless `overwrite` is set.
    pub fn install(
        &self,
        pack_name: &str,
        cli: TargetCli,
        target_root: &str,
        overwrite: bool,
    ) -> Result<InstallResultDto, ApplicationError> {
        let pack = self.repository.find(pack_name)?;
        if !pack.supports(cli) {
            return Err(ApplicationError::Domain(
                crate::domain::pack::error::DomainError::UnknownCli(cli.as_str().to_string()),
            ));
        }

        let root = Path::new(target_root);
        let plan = plan_install(&pack, cli, root)?;

        let mut copied = 0usize;
        for copy in &plan {
            if !overwrite && self.destination.exists(&copy.destination) {
                return Err(ApplicationError::DestinationExists(
                    copy.destination.to_string_lossy().into_owned(),
                ));
            }
        }
        for copy in &plan {
            self.destination
                .copy(pack_name, &copy.source, &copy.destination)?;
            copied += 1;
        }

        Ok(InstallResultDto {
            pack_name: pack.name.as_str().to_string(),
            target_root: target_root.to_string(),
            copied_count: copied,
        })
    }
}

/// Resolve the pack directory backing a repository entry (helper for
/// infrastructure wiring and the viewer).
pub fn pack_dir_hint(_name: &str) -> PathBuf {
    PathBuf::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::pack::traits::MockInstallDestination;
    use crate::application::pack::traits::MockPackRepository;
    use crate::domain::pack::entity::{Pack, PackItem};
    use crate::domain::pack::error::DomainError;
    use crate::domain::pack::value_objects::{PackItemKind, PackName, PackVersion, TargetCliMap};

    fn sample_pack(name: &str, cli: TargetCli) -> Pack {
        Pack::new(
            PackName::new(name).unwrap(),
            PackVersion::new("1.0.0").unwrap(),
            "desc".to_string(),
            vec![cli],
            vec![PackItem::new(
                PackItemKind::Skill,
                PathBuf::from("skills/tdd/SKILL.md"),
                TargetCliMap::from_pairs(&[(cli, ".agents/skills/tdd/")]),
            )
            .unwrap()],
        )
        .unwrap()
    }

    fn cli_of(pack: &Pack) -> TargetCli {
        pack.clis[0]
    }

    #[test]
    fn list_sorts_by_cli_then_name() {
        let mut repo = MockPackRepository::new();
        let zeta = sample_pack("zeta", TargetCli::Vibe);
        let alpha = sample_pack("alpha", TargetCli::Gemini);
        repo.expect_list()
            .returning(move || Ok(vec![zeta.clone(), alpha.clone()]));
        let service = PackService::new(Box::new(repo), Box::new(MockInstallDestination::new()));
        let dtos = service.list().unwrap();
        assert_eq!(dtos[0].name, "zeta");
        assert_eq!(dtos[0].clis[0], TargetCli::Vibe);
        assert_eq!(dtos[1].name, "alpha");
    }

    #[test]
    fn install_copies_plan_and_reports_count() {
        let pack = sample_pack("tdd", TargetCli::Vibe);
        let cli = cli_of(&pack);
        let pack_for_repo = pack.clone();
        let mut repo = MockPackRepository::new();
        repo.expect_find()
            .with(mockall::predicate::eq("tdd"))
            .returning(move |_| Ok(pack_for_repo.clone()));

        let mut dest = MockInstallDestination::new();
        dest.expect_exists().returning(|_| false);
        dest.expect_copy().times(1).returning(|_p, _s, _d| Ok(()));

        let service = PackService::new(Box::new(repo), Box::new(dest));
        let result = service.install("tdd", cli, "/repo", false).unwrap();
        assert_eq!(result.copied_count, 1);
        assert_eq!(result.pack_name, "tdd");
    }

    #[test]
    fn install_aborts_when_destination_exists_without_overwrite() {
        let pack = sample_pack("tdd", TargetCli::Vibe);
        let cli = cli_of(&pack);
        let pack_for_repo = pack.clone();
        let mut repo = MockPackRepository::new();
        repo.expect_find()
            .returning(move |_| Ok(pack_for_repo.clone()));

        let mut dest = MockInstallDestination::new();
        dest.expect_exists().returning(|_| true);
        dest.expect_copy().never();

        let service = PackService::new(Box::new(repo), Box::new(dest));
        let err = service.install("tdd", cli, "/repo", false).unwrap_err();
        assert!(matches!(err, ApplicationError::DestinationExists(_)));
    }

    #[test]
    fn install_with_overwrite_proceeds_on_existing_destination() {
        let pack = sample_pack("tdd", TargetCli::Vibe);
        let cli = cli_of(&pack);
        let pack_for_repo = pack.clone();
        let mut repo = MockPackRepository::new();
        repo.expect_find()
            .returning(move |_| Ok(pack_for_repo.clone()));

        let mut dest = MockInstallDestination::new();
        dest.expect_exists().returning(|_| true);
        dest.expect_copy().times(1).returning(|_p, _s, _d| Ok(()));

        let service = PackService::new(Box::new(repo), Box::new(dest));
        assert!(service.install("tdd", cli, "/repo", true).is_ok());
    }

    #[test]
    fn install_rejects_unsupported_cli() {
        let pack = sample_pack("tdd", TargetCli::Vibe);
        let pack_for_repo = pack.clone();
        let mut repo = MockPackRepository::new();
        repo.expect_find()
            .returning(move |_| Ok(pack_for_repo.clone()));

        let service = PackService::new(Box::new(repo), Box::new(MockInstallDestination::new()));
        let err = service
            .install("tdd", TargetCli::Gemini, "/repo", false)
            .unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::UnknownCli(_))
        ));
    }
}
