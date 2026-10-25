use crate::presentation::tui::app::{App, Focus, Mode, Popup};
use crate::presentation::tui::common::keys::{map_key, AppAction, KeyContext};

/// Screen routing: maps user actions to handlers (pattern rusty-console
/// router.rs, synchronous). Screens: split screen, viewer, dialogs.
impl App {
    pub fn key_context(&self) -> KeyContext {
        if self.input.is_some() {
            return KeyContext::Dialog;
        }
        if self.viewer.is_some() {
            return KeyContext::Viewer;
        }
        match self.mode {
            Mode::Installing { .. } => KeyContext::InstallMode,
            Mode::Normal => match self.focus {
                Focus::Packs => KeyContext::PacksPanel,
                Focus::Explorer => KeyContext::ExplorerPanel,
            },
        }
    }

    pub fn dispatch(&mut self, key: &crossterm::event::KeyEvent) {
        let context = self.key_context();

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
        // Popup has priority: any non-quit key closes it.
        if self.popup.is_some() && action != AppAction::Quit {
            self.popup = None;
            return;
        }

        match self.mode {
            Mode::Normal => match self.focus {
                Focus::Packs => self.handle_packs_action(action),
                Focus::Explorer => self.handle_explorer_action(action),
            },
            Mode::Installing { .. } => self.handle_install_action(action),
        }
    }

    fn handle_packs_action(&mut self, action: AppAction) {
        match action {
            AppAction::NavigateUp => self.pack_list.select_previous(),
            AppAction::NavigateDown => self.pack_list.select_next(),
            AppAction::Apply => self.start_install(),
            AppAction::Help => self.popup = Some(Popup::Help),
            _ => {}
        }
    }

    fn handle_explorer_action(&mut self, action: AppAction) {
        match action {
            AppAction::NavigateUp => self.dir_entries.select_previous(),
            AppAction::NavigateDown => self.dir_entries.select_next(),
            AppAction::Enter => self.open_selected_dir(),
            AppAction::Back => self.go_parent(),
            AppAction::CreateFolder => self.input = Some(Default::default()),
            AppAction::Help => self.popup = Some(Popup::Help),
            _ => {}
        }
    }

    fn handle_install_action(&mut self, action: AppAction) {
        match action {
            AppAction::NavigateUp => self.dir_entries.select_previous(),
            AppAction::NavigateDown => self.dir_entries.select_next(),
            AppAction::Enter => self.open_selected_dir(),
            AppAction::Back => self.go_parent(),
            AppAction::Install => self.install_here(),
            AppAction::Close => {
                self.mode = Mode::Normal;
                self.focus = Focus::Packs;
            }
            AppAction::Help => self.popup = Some(Popup::Help),
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

    fn install_here(&mut self) {
        let Mode::Installing { pack, cli } = &self.mode else {
            return;
        };
        let pack = pack.clone();
        let cli = *cli;
        let root = self.current_dir.to_string_lossy().into_owned();
        match self.pack_service.install(&pack, cli, &root, false) {
            Ok(result) => {
                self.mode = Mode::Normal;
                self.focus = Focus::Packs;
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
            KeyCode::Esc => self.input = None,
            KeyCode::Enter => {
                if let Some(input) = self.input.take() {
                    let parent = self.current_dir.to_string_lossy().into_owned();
                    match self.explorer.create_dir(&parent, &input.value) {
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
                    viewer.content_scroll = viewer.content_scroll.saturating_add(1);
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
        let Some(viewer) = &mut self.viewer else {
            return;
        };
        if let Some(path) = viewer.files.selected() {
            if let Ok(content) = self.view_service.read(&viewer.pack, path) {
                viewer.content = content.lines().map(|l| l.to_string()).collect();
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
                    content: Vec::new(),
                    content_scroll: 0,
                    show_content: false,
                };
                if let Some(first) = viewer.files.items.first().cloned() {
                    if let Ok(content) = self.view_service.read(&pack_name, &first) {
                        viewer.content = content.lines().map(|l| l.to_string()).collect();
                    }
                }
                self.viewer = Some(viewer);
            }
            Err(err) => self.show_error(err.to_string()),
        }
    }
}
