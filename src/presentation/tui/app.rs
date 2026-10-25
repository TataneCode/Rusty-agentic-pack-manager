use std::path::PathBuf;

use crate::application::explorer::service::ExplorerService;
use crate::application::pack::dto::PackSummaryDto;
use crate::application::pack::service::PackService;
use crate::application::pack_view::service::PackViewService;
use crate::domain::pack::value_objects::TargetCli;
use crate::presentation::tui::common::keys::AppAction;
use crate::presentation::tui::common::widgets::{SelectList, TextInput};

/// Which panel owns the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Packs,
    Explorer,
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

/// Full application state (pattern rusty-console App, synchronous).
pub struct App {
    pub packs: Vec<PackSummaryDto>,
    pub pack_list: SelectList<PackSummaryDto>,
    pub explorer: ExplorerService,
    pub pack_service: PackService,
    pub view_service: PackViewService,
    pub current_dir: PathBuf,
    pub dir_entries: SelectList<(String, bool)>,
    pub focus: Focus,
    pub mode: Mode,
    pub popup: Option<Popup>,
    pub input: Option<TextInput>,
    /// Viewer screen state; Some => viewer is open.
    pub viewer: Option<ViewerState>,
    pub quit: bool,
}

/// State of the 'v' viewer screen.
pub struct ViewerState {
    pub pack: String,
    pub files: SelectList<String>,
    pub content: Vec<String>,
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
        let packs = pack_service.list().unwrap_or_default();
        let pack_list = SelectList::new(packs.clone());
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
            explorer,
            pack_service,
            view_service,
            current_dir: start_dir,
            dir_entries,
            focus: Focus::Packs,
            mode: Mode::Normal,
            popup: None,
            input: None,
            viewer: None,
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
    }

    pub fn selected_entry(&self) -> Option<&(String, bool)> {
        self.dir_entries.selected()
    }

    pub fn selected_pack(&self) -> Option<&PackSummaryDto> {
        self.pack_list.selected()
    }

    pub fn start_install(&mut self) {
        let Some(pack) = self.selected_pack() else {
            return;
        };
        // Install for the pack's first target CLI (v1: single-CLI packs).
        let cli = pack.clis.first().copied().unwrap_or(TargetCli::Vibe);
        self.mode = Mode::Installing {
            pack: pack.name.clone(),
            cli,
        };
        self.focus = Focus::Explorer;
    }

    pub fn show_error(&mut self, message: String) {
        self.popup = Some(Popup::Error(message));
    }

    pub fn handle(&mut self, action: AppAction) {
        // Popup has priority: any key closes it.
        if self.popup.is_some() && action != AppAction::Quit {
            self.popup = None;
            return;
        }
        match action {
            AppAction::Quit => self.quit = true,
            AppAction::ToggleFocus => {
                if self.mode == Mode::Normal {
                    self.focus = match self.focus {
                        Focus::Packs => Focus::Explorer,
                        Focus::Explorer => Focus::Packs,
                    };
                }
            }
            AppAction::Close => self.mode = Mode::Normal,
            _ => {}
        }
    }
}
