use std::path::PathBuf;

use crate::application::explorer::service::ExplorerService;
use crate::application::pack::dto::{InstalledPackDto, PackSummaryDto};
use crate::application::pack::service::PackService;
use crate::application::pack_view::service::PackViewService;
use crate::domain::pack::value_objects::TargetCli;
use crate::presentation::tui::common::widgets::{SelectList, TextInput};

/// Which panel owns the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Packs,
    Installed,
    Explorer,
}

/// What a yes/no confirmation dialog is being used for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmKind {
    Delete { name: String },
    Uninstall { name: String, cli: TargetCli },
}

/// Modal state of the split screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    /// A pack was selected for installation; explorer drives target choice.
    Installing {
        pack: String,
        cli: TargetCli,
    },
}

/// Popups layered over the split screen.
pub enum Popup {
    Help,
    Error(String),
    Success(String),
}

/// What a text-input dialog is being used for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputKind {
    CreateFolder,
    Rename { original: String },
}

/// Yes/no confirmation dialog (destructive actions).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmDialog {
    pub message: String,
    pub selected_yes: bool,
    pub kind: ConfirmKind,
}

/// Full application state (pattern rusty-console App, synchronous).
pub struct App {
    pub packs: Vec<PackSummaryDto>,
    pub pack_list: SelectList<PackSummaryDto>,
    pub installed: SelectList<InstalledPackDto>,
    pub explorer: ExplorerService,
    pub pack_service: PackService,
    pub view_service: PackViewService,
    pub current_dir: PathBuf,
    pub dir_entries: SelectList<(String, bool)>,
    pub focus: Focus,
    pub popup: Option<Popup>,
    pub input: Option<TextInput>,
    pub input_kind: Option<InputKind>,
    pub confirm: Option<ConfirmDialog>,
    /// Viewer screen state; Some => viewer is open.
    pub viewer: Option<ViewerState>,
    pub highlighter: crate::presentation::tui::viewer::highlight::Highlighter,
    pub quit: bool,
}

/// State of the 'v' viewer screen.
pub struct ViewerState {
    pub pack: String,
    pub files: SelectList<String>,
    pub lines: Vec<Vec<ratatui::text::Span<'static>>>,
    pub content_scroll: u16,
    pub show_content: bool,
}

impl App {
    pub fn new(
        pack_service: PackService,
        explorer: ExplorerService,
        view_service: PackViewService,
        start_dir: PathBuf,
    ) -> App {
        let all_packs = pack_service.list().unwrap_or_default();
        let installed = pack_service
            .installed_in(start_dir.to_string_lossy().as_ref())
            .unwrap_or_default();
        let packs = filter_available(&all_packs, &installed);
        let pack_list = SelectList::new(packs.clone());
        let installed = SelectList::new(installed);
        let entries: Vec<(String, bool)> = explorer
            .list(start_dir.to_string_lossy().as_ref())
            .map(|l| {
                l.entries
                    .iter()
                    .map(|e| (e.name.clone(), e.is_dir))
                    .collect()
            })
            .unwrap_or_default();
        let dir_entries = SelectList::new(entries);
        App {
            packs,
            pack_list,
            installed,
            explorer,
            pack_service,
            view_service,
            current_dir: start_dir,
            dir_entries,
            focus: Focus::Packs,
            popup: None,
            input: None,
            input_kind: None,
            confirm: None,
            viewer: None,
            highlighter: crate::presentation::tui::viewer::highlight::Highlighter::new(),
            quit: false,
        }
    }

    pub fn refresh_dir(&mut self) {
        if let Ok(listing) = self
            .explorer
            .list(self.current_dir.to_string_lossy().as_ref())
        {
            let entries: Vec<(String, bool)> = listing
                .entries
                .iter()
                .map(|e| (e.name.clone(), e.is_dir))
                .collect();
            self.dir_entries = SelectList::new(entries);
        }
        self.refresh_installed();
    }

    /// Reload the installations recorded for the current directory, and
    /// re-filter the available pack list: a pack installed here (for its
    /// first target CLI, like `A` installs it) no longer appears as
    /// available.
    pub fn refresh_installed(&mut self) {
        let root = self.current_dir.to_string_lossy().into_owned();
        let installed = self.pack_service.installed_in(&root).unwrap_or_default();
        let all_packs = self.pack_service.list().unwrap_or_default();
        let available = filter_available(&all_packs, &installed);
        self.packs = available.clone();
        self.pack_list = SelectList::new(available);
        self.installed = SelectList::new(installed);
    }

    pub fn selected_entry(&self) -> Option<&(String, bool)> {
        self.dir_entries.selected()
    }

    pub fn selected_pack(&self) -> Option<&PackSummaryDto> {
        self.pack_list.selected()
    }

    pub fn selected_installed(&self) -> Option<&InstalledPackDto> {
        self.installed.selected()
    }

    pub fn show_error(&mut self, message: String) {
        self.popup = Some(Popup::Error(message));
    }
}

/// Packs still available for installation: a pack is unavailable once the
/// index records it for its first target CLI (the CLI `A` installs for).
pub fn filter_available(
    packs: &[PackSummaryDto],
    installed: &[InstalledPackDto],
) -> Vec<PackSummaryDto> {
    packs
        .iter()
        .filter(|pack| {
            pack.clis.first().is_none_or(|cli| {
                !installed
                    .iter()
                    .any(|e| e.name == pack.name && e.cli == *cli)
            })
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn pack(name: &str, clis: &[TargetCli]) -> PackSummaryDto {
        PackSummaryDto {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            clis: clis.to_vec(),
            item_count: 1,
        }
    }

    fn installed(name: &str, cli: TargetCli) -> InstalledPackDto {
        InstalledPackDto {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            cli,
            paths: vec![PathBuf::from(".agents/skills/x")],
        }
    }

    #[test]
    fn filter_available_excludes_packs_installed_for_their_first_cli() {
        let packs = vec![
            pack("tdd", &[TargetCli::Vibe]),
            pack("review", &[TargetCli::Vibe, TargetCli::Gemini]),
            pack("free", &[TargetCli::Gemini]),
        ];
        let installed = vec![
            installed("tdd", TargetCli::Vibe),
            installed("review", TargetCli::Vibe),
        ];
        let available = filter_available(&packs, &installed);
        let names: Vec<&str> = available.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["free"]);
    }

    #[test]
    fn filter_available_keeps_pack_when_only_another_cli_is_installed() {
        let packs = vec![pack("tdd", &[TargetCli::Vibe, TargetCli::Gemini])];
        // The index records gemini, but `A` installs for the first CLI (vibe):
        // the pack stays available.
        let installed = vec![installed("tdd", TargetCli::Gemini)];
        let available = filter_available(&packs, &installed);
        assert_eq!(available.len(), 1);
    }

    #[test]
    fn filter_available_with_no_install_keeps_everything() {
        let packs = vec![pack("tdd", &[TargetCli::Vibe])];
        assert_eq!(filter_available(&packs, &[]).len(), 1);
    }
}
