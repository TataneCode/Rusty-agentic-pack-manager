use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::application::explorer::service::ExplorerService;
use crate::application::pack::dto::{InstallPlanDto, InstalledPackDto, PackSummaryDto};
use crate::application::pack::service::PackService;
use crate::application::pack_view::service::PackViewService;
use crate::presentation::tui::common::widgets::{SelectList, TextInput};

/// Which panel owns the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Packs,
    Installed,
    Explorer,
}

/// One display line of the packs panel: a CLI group header (carrying the
/// CLI id, always visible with its available-pack count) or a pack row
/// (hidden while its CLI group is collapsed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackLine {
    Cli(String),
    Pack(PackSummaryDto),
}

/// What a yes/no confirmation dialog is being used for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmKind {
    Delete { name: String },
    Uninstall { name: String, cli: String },
}

/// Modal state of the split screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    /// A pack was selected for installation; explorer drives target choice.
    Installing {
        pack: String,
        cli: String,
    },
}

/// Popups layered over the split screen.
pub enum Popup {
    Help,
    Error(String),
    Success(String),
    /// Install plan preview (E04, 'p' on the packs panel).
    Plan(InstallPlanDto),
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
    pub pack_list: SelectList<PackLine>,
    pub installed: SelectList<InstalledPackDto>,
    pub explorer: ExplorerService,
    pub pack_service: PackService,
    pub view_service: PackViewService,
    pub current_dir: PathBuf,
    pub dir_entries: SelectList<(String, bool)>,
    pub focus: Focus,
    /// CLI ids whose group is collapsed in the packs panel ('c' toggles).
    pub collapsed_clis: BTreeSet<String>,
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
        let mut app = App {
            packs,
            pack_list: SelectList::new(Vec::new()),
            installed,
            explorer,
            pack_service,
            view_service,
            current_dir: start_dir,
            dir_entries,
            focus: Focus::Packs,
            collapsed_clis: BTreeSet::new(),
            popup: None,
            input: None,
            input_kind: None,
            confirm: None,
            viewer: None,
            highlighter: crate::presentation::tui::viewer::highlight::Highlighter::new(),
            quit: false,
        };
        app.apply_cli_visibility();
        app
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
    /// re-filter the available pack rows: a (pack, CLI) row installed here
    /// no longer appears as available (the pack stays listed under its
    /// other target CLIs).
    pub fn refresh_installed(&mut self) {
        let root = self.current_dir.to_string_lossy().into_owned();
        let installed = self.pack_service.installed_in(&root).unwrap_or_default();
        let all_packs = self.pack_service.list().unwrap_or_default();
        let available = filter_available(&all_packs, &installed);
        self.packs = available;
        self.apply_cli_visibility();
        self.installed = SelectList::new(installed);
    }

    /// Rebuild the packs panel lines from `packs`: one CLI header per group
    /// (always visible), plus the pack rows of the non-collapsed groups.
    /// Rows are sorted by CLI then name, so groups are consecutive. The
    /// selection index is kept, clamped.
    fn apply_cli_visibility(&mut self) {
        let selected = self.pack_list.selected_index();
        let mut lines: Vec<PackLine> = Vec::new();
        let mut current_cli: Option<String> = None;
        for pack in &self.packs {
            let cli = pack.clis.first().cloned().unwrap_or_default();
            if current_cli.as_deref() != Some(cli.as_str()) {
                current_cli = Some(cli.clone());
                lines.push(PackLine::Cli(cli.clone()));
            }
            if !self.collapsed_clis.contains(&cli) {
                lines.push(PackLine::Pack(pack.clone()));
            }
        }
        self.pack_list = SelectList::new(lines);
        match selected {
            Some(index) if index < self.pack_list.items.len() => {
                self.pack_list.state.select(Some(index));
            }
            // Fresh list: start on the first pack row, not on a CLI header.
            _ => {
                if let Some(first_pack) = self
                    .pack_list
                    .items
                    .iter()
                    .position(|line| matches!(line, PackLine::Pack(_)))
                {
                    self.pack_list.state.select(Some(first_pack));
                }
            }
        }
    }

    /// 'c' on the packs panel: collapse the CLI group under the cursor (a
    /// group header or one of its rows), or expand it back. The cursor is
    /// placed on the group header so a second 'c' toggles the same group.
    pub fn toggle_cli_group(&mut self) {
        let Some(cli) = self.pack_list.selected().map(|line| match line {
            PackLine::Cli(cli) => cli.clone(),
            PackLine::Pack(pack) => pack.clis.first().cloned().unwrap_or_default(),
        }) else {
            return;
        };
        if !self.collapsed_clis.remove(&cli) {
            self.collapsed_clis.insert(cli.clone());
        }
        self.apply_cli_visibility();
        if let Some(header) = self
            .pack_list
            .items
            .iter()
            .position(|line| matches!(line, PackLine::Cli(c) if *c == cli))
        {
            self.pack_list.state.select(Some(header));
        }
    }

    pub fn selected_entry(&self) -> Option<&(String, bool)> {
        self.dir_entries.selected()
    }

    pub fn selected_pack(&self) -> Option<&PackSummaryDto> {
        match self.pack_list.selected() {
            Some(PackLine::Pack(pack)) => Some(pack),
            _ => None,
        }
    }

    pub fn selected_installed(&self) -> Option<&InstalledPackDto> {
        self.installed.selected()
    }

    pub fn show_error(&mut self, message: String) {
        self.popup = Some(Popup::Error(message));
    }
}

/// Pack rows still available for installation: the list holds one row per
/// (pack, target CLI) — a row is unavailable once the index records that
/// exact (pack, CLI) couple.
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

    fn pack(name: &str, clis: &[&str]) -> PackSummaryDto {
        PackSummaryDto {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            clis: clis.iter().map(|c| c.to_string()).collect(),
            item_count: 1,
        }
    }

    fn installed(name: &str, cli: &str) -> InstalledPackDto {
        InstalledPackDto {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            cli: cli.to_string(),
            paths: vec![PathBuf::from(".agents/skills/x")],
        }
    }

    #[test]
    fn filter_available_excludes_only_the_rows_installed_for_that_cli() {
        // Rows as produced by PackService::list(): one per (pack, CLI).
        let rows = vec![
            pack("tdd", &["vibe"]),
            pack("review", &["gemini"]),
            pack("review", &["vibe"]),
            pack("free", &["gemini"]),
        ];
        let installed = vec![installed("tdd", "vibe"), installed("review", "vibe")];
        let available = filter_available(&rows, &installed);
        let keys: Vec<(&str, &str)> = available
            .iter()
            .map(|p| (p.name.as_str(), p.clis[0].as_str()))
            .collect();
        assert_eq!(keys, vec![("review", "gemini"), ("free", "gemini")]);
    }

    #[test]
    fn filter_available_keeps_the_other_cli_rows_when_one_cli_is_installed() {
        let rows = vec![pack("tdd", &["gemini"]), pack("tdd", &["vibe"])];
        // The index records gemini: only the gemini row is hidden.
        let installed = vec![installed("tdd", "gemini")];
        let available = filter_available(&rows, &installed);
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].clis[0], "vibe");
    }

    #[test]
    fn filter_available_with_no_install_keeps_everything() {
        let packs = vec![pack("tdd", &["vibe"])];
        assert_eq!(filter_available(&packs, &[]).len(), 1);
    }
}
