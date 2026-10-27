use std::path::{Path, PathBuf};

use crate::application::error::ApplicationError;
use crate::application::pack::dto::{
    InstallResultDto, InstalledPackDto, PackSummaryDto, UninstallResultDto,
};
use crate::application::pack::traits::{
    InstallDestination, InstallIndexRepository, PackRepository,
};
use crate::domain::pack::entity::Pack;
use crate::domain::pack::error::plan_install;
use crate::domain::pack::installed::InstalledPack;
use crate::domain::pack::value_objects::TargetCli;

/// Application service for the packs panel: list, install (traced in the
/// target's install index), uninstall from the index, and list the
/// installations recorded for a target.
pub struct PackService {
    repository: Box<dyn PackRepository>,
    destination: Box<dyn InstallDestination>,
    index: Box<dyn InstallIndexRepository>,
}

impl PackService {
    pub fn new(
        repository: Box<dyn PackRepository>,
        destination: Box<dyn InstallDestination>,
        index: Box<dyn InstallIndexRepository>,
    ) -> PackService {
        PackService {
            repository,
            destination,
            index,
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

    /// Installations recorded in the install index of `target_root`.
    pub fn installed_in(
        &self,
        target_root: &str,
    ) -> Result<Vec<InstalledPackDto>, ApplicationError> {
        Ok(self
            .index
            .load(target_root)?
            .iter()
            .map(InstalledPackDto::from_installed)
            .collect())
    }

    /// Install `pack_name` for `cli` inside `target_root`. The domain install
    /// plan is computed first (no partial writes on validation errors);
    /// existing destinations abort the install unless `overwrite` is set.
    /// On success the installation is recorded in the target's install
    /// index — after the copies, never before.
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

        let paths = plan
            .iter()
            .map(|copy| relative_to_root(root, &copy.destination))
            .collect::<Result<Vec<PathBuf>, _>>()?;
        let entry = InstalledPack::new(pack.name.clone(), pack.version.clone(), cli, paths)?;
        self.index.record(target_root, &entry)?;

        Ok(InstallResultDto {
            pack_name: pack.name.as_str().to_string(),
            target_root: target_root.to_string(),
            copied_count: copied,
        })
    }

    /// Uninstall a pack recorded in the index of `target_root`: remove every
    /// listed asset (files or directory trees), then drop the index entry.
    /// Assets already missing on disk are tolerated; the entry is only
    /// dropped when no removal failed.
    pub fn uninstall(
        &self,
        pack_name: &str,
        cli: TargetCli,
        target_root: &str,
    ) -> Result<UninstallResultDto, ApplicationError> {
        let root = Path::new(target_root);
        let entries = self.index.load(target_root)?;
        let entry = entries
            .iter()
            .find(|e| e.matches(pack_name, cli))
            .ok_or_else(|| ApplicationError::InstallNotFound(format!("{pack_name} ({cli})")))?;

        let mut removed = 0usize;
        for rel in &entry.paths {
            let destination = root.join(rel);
            if !self.destination.exists(&destination) {
                continue;
            }
            self.destination.remove(&destination)?;
            removed += 1;
        }
        self.index.remove(target_root, pack_name, cli)?;

        Ok(UninstallResultDto {
            pack_name: pack_name.to_string(),
            cli,
            removed_count: removed,
        })
    }
}

/// Path of a planned destination relative to the target root (for the
/// install index). The plan guarantees the destination is inside the root.
fn relative_to_root(root: &Path, destination: &Path) -> Result<PathBuf, ApplicationError> {
    destination
        .strip_prefix(root)
        .map(Path::to_path_buf)
        .map_err(|_| {
            ApplicationError::Domain(
                crate::domain::pack::error::DomainError::InstallPathOutsideTarget(
                    destination.to_string_lossy().into_owned(),
                ),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::pack::traits::{
        MockInstallDestination, MockInstallIndexRepository, MockPackRepository,
    };
    use crate::domain::pack::entity::{Pack, PackItem};
    use crate::domain::pack::error::DomainError;
    use crate::domain::pack::value_objects::{PackItemKind, PackName, PackVersion, TargetCliMap};
    use std::path::PathBuf;

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

    fn entry_for(pack_name: &str, cli: TargetCli) -> InstalledPack {
        InstalledPack::new(
            PackName::new(pack_name).unwrap(),
            PackVersion::new("1.0.0").unwrap(),
            cli,
            vec![PathBuf::from(".agents/skills/tdd")],
        )
        .unwrap()
    }

    fn service_with(
        pack: Pack,
    ) -> (
        MockPackRepository,
        MockInstallDestination,
        MockInstallIndexRepository,
    ) {
        let pack_for_repo = pack.clone();
        let mut repo = MockPackRepository::new();
        repo.expect_find()
            .returning(move |_| Ok(pack_for_repo.clone()));
        (
            repo,
            MockInstallDestination::new(),
            MockInstallIndexRepository::new(),
        )
    }

    #[test]
    fn list_sorts_by_cli_then_name() {
        let mut repo = MockPackRepository::new();
        let zeta = sample_pack("zeta", TargetCli::Vibe);
        let alpha = sample_pack("alpha", TargetCli::Gemini);
        repo.expect_list()
            .returning(move || Ok(vec![zeta.clone(), alpha.clone()]));
        let service = PackService::new(
            Box::new(repo),
            Box::new(MockInstallDestination::new()),
            Box::new(MockInstallIndexRepository::new()),
        );
        let dtos = service.list().unwrap();
        assert_eq!(dtos[0].name, "zeta");
        assert_eq!(dtos[0].clis[0], TargetCli::Vibe);
        assert_eq!(dtos[1].name, "alpha");
    }

    #[test]
    fn install_copies_plan_records_index_and_reports_count() {
        let pack = sample_pack("tdd", TargetCli::Vibe);
        let cli = cli_of(&pack);
        let (repo, mut dest, mut index) = service_with(pack);

        dest.expect_exists().returning(|_| false);
        dest.expect_copy().times(1).returning(|_p, _s, _d| Ok(()));
        index
            .expect_record()
            .times(1)
            .withf(|root, entry| {
                root == "/repo"
                    && entry.name.as_str() == "tdd"
                    && entry.cli == TargetCli::Vibe
                    && entry.paths == vec![PathBuf::from(".agents/skills/tdd")]
            })
            .returning(|_, _| Ok(()));

        let service = PackService::new(Box::new(repo), Box::new(dest), Box::new(index));
        let result = service.install("tdd", cli, "/repo", false).unwrap();
        assert_eq!(result.copied_count, 1);
        assert_eq!(result.pack_name, "tdd");
    }

    #[test]
    fn install_does_not_record_index_when_copy_fails() {
        let pack = sample_pack("tdd", TargetCli::Vibe);
        let cli = cli_of(&pack);
        let (repo, mut dest, mut index) = service_with(pack);

        dest.expect_exists().returning(|_| false);
        dest.expect_copy()
            .returning(|_p, _s, _d| Err(ApplicationError::DestinationExists("boom".into())));
        index.expect_record().never();

        let service = PackService::new(Box::new(repo), Box::new(dest), Box::new(index));
        assert!(service.install("tdd", cli, "/repo", false).is_err());
    }

    #[test]
    fn install_aborts_when_destination_exists_without_overwrite() {
        let pack = sample_pack("tdd", TargetCli::Vibe);
        let cli = cli_of(&pack);
        let (repo, mut dest, mut index) = service_with(pack);

        dest.expect_exists().returning(|_| true);
        dest.expect_copy().never();
        index.expect_record().never();

        let service = PackService::new(Box::new(repo), Box::new(dest), Box::new(index));
        let err = service.install("tdd", cli, "/repo", false).unwrap_err();
        assert!(matches!(err, ApplicationError::DestinationExists(_)));
    }

    #[test]
    fn install_with_overwrite_proceeds_on_existing_destination() {
        let pack = sample_pack("tdd", TargetCli::Vibe);
        let cli = cli_of(&pack);
        let (repo, mut dest, mut index) = service_with(pack);

        dest.expect_exists().returning(|_| true);
        dest.expect_copy().times(1).returning(|_p, _s, _d| Ok(()));
        index.expect_record().times(1).returning(|_, _| Ok(()));

        let service = PackService::new(Box::new(repo), Box::new(dest), Box::new(index));
        assert!(service.install("tdd", cli, "/repo", true).is_ok());
    }

    #[test]
    fn install_rejects_unsupported_cli() {
        let pack = sample_pack("tdd", TargetCli::Vibe);
        let (repo, dest, index) = service_with(pack);

        let service = PackService::new(Box::new(repo), Box::new(dest), Box::new(index));
        let err = service
            .install("tdd", TargetCli::Gemini, "/repo", false)
            .unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::UnknownCli(_))
        ));
    }

    #[test]
    fn installed_in_maps_index_entries_to_dtos() {
        let mut index = MockInstallIndexRepository::new();
        index
            .expect_load()
            .with(mockall::predicate::eq("/repo"))
            .returning(|_| Ok(vec![entry_for("tdd", TargetCli::Vibe)]));

        let service = PackService::new(
            Box::new(MockPackRepository::new()),
            Box::new(MockInstallDestination::new()),
            Box::new(index),
        );
        let dtos = service.installed_in("/repo").unwrap();
        assert_eq!(dtos.len(), 1);
        assert_eq!(dtos[0].name, "tdd");
        assert_eq!(dtos[0].cli, TargetCli::Vibe);
        assert_eq!(dtos[0].paths, vec![PathBuf::from(".agents/skills/tdd")]);
    }

    #[test]
    fn uninstall_removes_assets_then_index_entry() {
        let mut index = MockInstallIndexRepository::new();
        let entry = entry_for("tdd", TargetCli::Vibe);
        let entry_for_load = entry.clone();
        index
            .expect_load()
            .returning(move |_| Ok(vec![entry_for_load.clone()]));
        index
            .expect_remove()
            .times(1)
            .with(
                mockall::predicate::eq("/repo"),
                mockall::predicate::eq("tdd"),
                mockall::predicate::eq(TargetCli::Vibe),
            )
            .returning(|_, _, _| Ok(()));

        let mut dest = MockInstallDestination::new();
        dest.expect_exists().returning(|_| true);
        dest.expect_remove()
            .times(1)
            .with(mockall::predicate::eq(Path::new(
                "/repo/.agents/skills/tdd",
            )))
            .returning(|_| Ok(()));

        let service = PackService::new(
            Box::new(MockPackRepository::new()),
            Box::new(dest),
            Box::new(index),
        );
        let result = service.uninstall("tdd", TargetCli::Vibe, "/repo").unwrap();
        assert_eq!(result.removed_count, 1);
        assert_eq!(result.pack_name, "tdd");
        assert_eq!(result.cli, TargetCli::Vibe);
    }

    #[test]
    fn uninstall_tolerates_assets_already_missing() {
        let mut index = MockInstallIndexRepository::new();
        let entry_for_load = entry_for("tdd", TargetCli::Vibe);
        index
            .expect_load()
            .returning(move |_| Ok(vec![entry_for_load.clone()]));
        index.expect_remove().times(1).returning(|_, _, _| Ok(()));

        let mut dest = MockInstallDestination::new();
        dest.expect_exists().returning(|_| false);
        dest.expect_remove().never();

        let service = PackService::new(
            Box::new(MockPackRepository::new()),
            Box::new(dest),
            Box::new(index),
        );
        let result = service.uninstall("tdd", TargetCli::Vibe, "/repo").unwrap();
        assert_eq!(result.removed_count, 0);
    }

    #[test]
    fn uninstall_keeps_index_entry_when_a_removal_fails() {
        let mut index = MockInstallIndexRepository::new();
        let entry_for_load = entry_for("tdd", TargetCli::Vibe);
        index
            .expect_load()
            .returning(move |_| Ok(vec![entry_for_load.clone()]));
        index.expect_remove().never();

        let mut dest = MockInstallDestination::new();
        dest.expect_exists().returning(|_| true);
        dest.expect_remove()
            .returning(|_| Err(ApplicationError::DestinationExists("locked".into())));

        let service = PackService::new(
            Box::new(MockPackRepository::new()),
            Box::new(dest),
            Box::new(index),
        );
        assert!(service.uninstall("tdd", TargetCli::Vibe, "/repo").is_err());
    }

    #[test]
    fn uninstall_rejects_unknown_entry() {
        let mut index = MockInstallIndexRepository::new();
        index.expect_load().returning(|_| Ok(vec![]));

        let service = PackService::new(
            Box::new(MockPackRepository::new()),
            Box::new(MockInstallDestination::new()),
            Box::new(index),
        );
        let err = service
            .uninstall("ghost", TargetCli::Vibe, "/repo")
            .unwrap_err();
        assert!(matches!(err, ApplicationError::InstallNotFound(_)));
    }
}
