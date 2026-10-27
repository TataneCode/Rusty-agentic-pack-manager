use crate::domain::pack::value_objects::TargetCli;
use crate::presentation::tui::app::{App, ConfirmDialog, ConfirmKind, Focus, InputKind, Popup};
use crate::presentation::tui::common::keys::{map_key, AppAction, KeyContext};

/// Screen routing: maps user actions to handlers (pattern rusty-console
/// router.rs, synchronous). Screens: split screen, viewer, dialogs.
impl App {
    pub fn highlighter(&self) -> &crate::presentation::tui::viewer::highlight::Highlighter {
        &self.highlighter
    }

    pub fn key_context(&self) -> KeyContext {
        if self.input.is_some() {
            return KeyContext::Dialog;
        }
        if self.viewer.is_some() {
            return KeyContext::Viewer;
        }
        match self.focus {
            Focus::Packs => KeyContext::PacksPanel,
            Focus::Installed => KeyContext::InstalledPanel,
            Focus::Explorer => KeyContext::ExplorerPanel,
        }
    }

    pub fn dispatch(&mut self, key: &crossterm::event::KeyEvent) {
        let context = self.key_context();

        if self.confirm.is_some() {
            self.handle_confirm_key(key);
            return;
        }

        if self.input.is_some() {
            self.handle_input_key(key);
            return;
        }

        let Some(action) = map_key(key, context) else {
            return;
        };

        if self.viewer.is_some() {
            self.handle_viewer_action(action);
            return;
        }

        self.handle_action(action);
    }

    fn handle_action(&mut self, action: AppAction) {
        // Quit works from any mode/focus (advertised as Ctrl-C).
        if let AppAction::Quit = action {
            self.quit = true;
            return;
        }
        // Popup has priority: any key closes it.
        if self.popup.is_some() {
            self.popup = None;
            return;
        }

        match self.focus {
            Focus::Packs => self.handle_packs_action(action),
            Focus::Installed => self.handle_installed_action(action),
            Focus::Explorer => self.handle_explorer_action(action),
        }
    }

    fn handle_packs_action(&mut self, action: AppAction) {
        match action {
            AppAction::NavigateUp => self.pack_list.select_previous(),
            AppAction::NavigateDown => self.pack_list.select_next(),
            AppAction::Apply => self.install_here(),
            AppAction::View => self.open_viewer(),
            AppAction::Help => self.popup = Some(Popup::Help),
            AppAction::ToggleFocus => self.focus = Focus::Installed,
            _ => {}
        }
    }

    fn handle_installed_action(&mut self, action: AppAction) {
        match action {
            AppAction::NavigateUp => self.installed.select_previous(),
            AppAction::NavigateDown => self.installed.select_next(),
            AppAction::Uninstall => self.ask_uninstall(),
            AppAction::Help => self.popup = Some(Popup::Help),
            AppAction::ToggleFocus => self.focus = Focus::Explorer,
            _ => {}
        }
    }

    fn handle_explorer_action(&mut self, action: AppAction) {
        match action {
            AppAction::NavigateUp => self.dir_entries.select_previous(),
            AppAction::NavigateDown => self.dir_entries.select_next(),
            AppAction::Enter => self.open_selected_dir(),
            AppAction::Back => self.go_parent(),
            AppAction::CreateFolder => {
                self.input = Some(Default::default());
                self.input_kind = Some(InputKind::CreateFolder);
            }
            AppAction::Rename => self.start_rename(),
            AppAction::Delete => self.ask_delete(false),
            AppAction::ForceDelete => self.ask_delete(true),
            AppAction::Help => self.popup = Some(Popup::Help),
            AppAction::ToggleFocus => self.focus = Focus::Packs,
            _ => {}
        }
    }

    fn ask_uninstall(&mut self) {
        let Some(entry) = self.selected_installed() else {
            return;
        };
        let (name, cli, count) = (entry.name.clone(), entry.cli, entry.paths.len());
        self.confirm = Some(ConfirmDialog {
            message: format!("Uninstall '{name}' ({cli}) — remove {count} item(s)?"),
            selected_yes: false,
            kind: ConfirmKind::Uninstall { name, cli },
        });
    }

    fn execute_uninstall(&mut self, name: &str, cli: TargetCli) {
        let root = self.current_dir.to_string_lossy().into_owned();
        match self.pack_service.uninstall(name, cli, &root) {
            Ok(result) => {
                self.refresh_dir();
                self.popup = Some(Popup::Success(format!(
                    "{} uninstalled: {} item(s) removed from {}",
                    result.pack_name, result.removed_count, root
                )));
            }
            Err(err) => self.show_error(err.to_string()),
        }
    }

    fn start_rename(&mut self) {
        let Some((name, _)) = self.dir_entries.selected() else {
            return;
        };
        let mut input = crate::presentation::tui::common::widgets::TextInput::new();
        input.value = name.clone();
        input.cursor = name.len();
        self.input = Some(input);
        self.input_kind = Some(InputKind::Rename {
            original: name.clone(),
        });
    }

    fn ask_delete(&mut self, force: bool) {
        let Some((name, _)) = self.dir_entries.selected() else {
            return;
        };
        if force {
            let name = name.clone();
            self.execute_delete(&name);
        } else {
            self.confirm = Some(ConfirmDialog {
                message: format!("Delete '{name}'?"),
                selected_yes: false,
                kind: ConfirmKind::Delete { name: name.clone() },
            });
        }
    }

    fn execute_delete(&mut self, name: &str) {
        let parent = self.current_dir.to_string_lossy().into_owned();
        match self.explorer.remove(&parent, name) {
            Ok(()) => self.refresh_dir(),
            Err(err) => self.show_error(err.to_string()),
        }
    }

    fn handle_confirm_key(&mut self, key: &crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        let Some(dialog) = &mut self.confirm else {
            return;
        };
        match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                dialog.selected_yes = !dialog.selected_yes;
            }
            KeyCode::Up | KeyCode::Char('k') => dialog.selected_yes = true,
            KeyCode::Down | KeyCode::Char('j') => dialog.selected_yes = false,
            KeyCode::Enter => {
                let dialog = self.confirm.take().unwrap();
                if dialog.selected_yes {
                    match &dialog.kind {
                        ConfirmKind::Delete { name } => {
                            let name = name.clone();
                            self.execute_delete(&name);
                        }
                        ConfirmKind::Uninstall { name, cli } => {
                            let (name, cli) = (name.clone(), *cli);
                            self.execute_uninstall(&name, cli);
                        }
                    }
                }
            }
            KeyCode::Esc => self.confirm = None,
            _ => {}
        }
    }

    fn open_selected_dir(&mut self) {
        let Some((name, is_dir)) = self.dir_entries.selected() else {
            return;
        };
        if !*is_dir {
            return;
        }
        let next = self.current_dir.join(name);
        if next.is_dir() {
            self.current_dir = next;
            self.refresh_dir();
        }
    }

    fn go_parent(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.refresh_dir();
        }
    }

    /// Install the selected pack in the explorer's current directory
    /// (user decision: 'A' alone, no separate install mode).
    fn install_here(&mut self) {
        let Some(pack) = self.selected_pack() else {
            return;
        };
        let pack_name = pack.name.clone();
        // Install for the pack's first target CLI (CLI picker is v1.1).
        let cli = pack.clis.first().copied().unwrap_or(TargetCli::Vibe);
        let root = self.current_dir.to_string_lossy().into_owned();
        match self.pack_service.install(&pack_name, cli, &root, false) {
            Ok(result) => {
                self.refresh_dir();
                self.popup = Some(Popup::Success(format!(
                    "{} installed: {} item(s) into {}",
                    result.pack_name, result.copied_count, result.target_root
                )));
            }
            Err(err) => self.show_error(err.to_string()),
        }
    }

    fn handle_input_key(&mut self, key: &crossterm::event::KeyEvent) {
        use crossterm::event::KeyCode;
        if self.input.is_none() {
            return;
        }
        match key.code {
            KeyCode::Esc => {
                self.input = None;
                self.input_kind = None;
            }
            KeyCode::Enter => {
                if let Some(input) = self.input.take() {
                    let kind = self.input_kind.take();
                    let parent = self.current_dir.to_string_lossy().into_owned();
                    let result = match kind {
                        Some(InputKind::Rename { original }) => {
                            self.explorer.rename(&parent, &original, &input.value)
                        }
                        _ => self.explorer.create_dir(&parent, &input.value),
                    };
                    match result {
                        Ok(()) => self.refresh_dir(),
                        Err(err) => self.show_error(err.to_string()),
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(input) = &mut self.input {
                    input.backspace();
                }
            }
            KeyCode::Char(c) => {
                if let Some(input) = &mut self.input {
                    input.input(c);
                }
            }
            _ => {}
        }
    }

    fn handle_viewer_action(&mut self, action: AppAction) {
        let Some(viewer) = &mut self.viewer else {
            return;
        };
        match action {
            AppAction::ToggleFocus => {
                if !viewer.files.items.is_empty() {
                    viewer.show_content = !viewer.show_content
                }
            }
            AppAction::NavigateUp => {
                if viewer.show_content {
                    viewer.content_scroll = viewer.content_scroll.saturating_sub(1);
                } else {
                    viewer.files.select_previous();
                    self.load_viewer_content();
                }
            }
            AppAction::NavigateDown => {
                if viewer.show_content {
                    let max_scroll = viewer.lines.len().saturating_sub(1) as u16;
                    viewer.content_scroll = (viewer.content_scroll + 1).min(max_scroll);
                } else {
                    viewer.files.select_next();
                    self.load_viewer_content();
                }
            }
            AppAction::Back => viewer.show_content = false,
            AppAction::Close => self.viewer = None,
            AppAction::Enter => viewer.show_content = true,
            AppAction::Help => self.popup = Some(Popup::Help),
            _ => {}
        }
    }

    fn load_viewer_content(&mut self) {
        let Some(viewer) = &self.viewer else {
            return;
        };
        let Some(path) = viewer.files.selected().cloned() else {
            return;
        };
        let pack = viewer.pack.clone();
        if let Ok(content) = self.view_service.read(&pack, &path) {
            let lines = self.highlighter.highlight(&path, &content);
            if let Some(viewer) = &mut self.viewer {
                viewer.lines = lines;
                viewer.content_scroll = 0;
            }
        }
    }

    pub fn open_viewer(&mut self) {
        let Some(pack) = self.selected_pack() else {
            return;
        };
        let pack_name = pack.name.clone();
        match self.view_service.list_viewable(&pack_name) {
            Ok(files) => {
                let paths: Vec<String> = files.into_iter().map(|f| f.path).collect();
                let mut viewer = crate::presentation::tui::app::ViewerState {
                    pack: pack_name.clone(),
                    files: crate::presentation::tui::common::widgets::SelectList::new(paths),
                    lines: Vec::new(),
                    content_scroll: 0,
                    show_content: false,
                };
                if let Some(first) = viewer.files.items.first().cloned() {
                    if let Ok(content) = self.view_service.read(&pack_name, &first) {
                        viewer.lines = self.highlighter.highlight(&first, &content);
                    }
                }
                self.viewer = Some(viewer);
            }
            Err(err) => self.show_error(err.to_string()),
        }
    }
}
