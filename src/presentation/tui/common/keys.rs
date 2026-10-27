use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Logical actions, decoupled from raw keys (pattern rusty-console AppAction).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    Quit,
    ToggleFocus,
    NavigateUp,
    NavigateDown,
    Enter,
    Back,
    Apply,        // 'a' on packs panel: install in the explorer's current dir
    View,         // 'v' on packs panel
    Uninstall,    // 'u' on installed panel: remove a recorded installation
    CreateFolder, // 'r' on explorer panel
    Rename,       // 'm' on explorer panel
    Delete,       // 'd' on explorer panel (confirm dialog)
    ForceDelete,  // 'D' (shift-d) on explorer panel: force non-empty folder
    Help,         // spacebar
    Close,        // esc: close popup/dialog/exit mode
}

/// Key-to-action mapping, context-dependent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyContext {
    PacksPanel,
    InstalledPanel,
    ExplorerPanel,
    Viewer,
    Dialog,
}

pub fn map_key(key: &KeyEvent, context: KeyContext) -> Option<AppAction> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if key.code == KeyCode::Char('c') {
            return Some(AppAction::Quit);
        }
        return None;
    }

    // Shift-D forces deletion without the confirmation dialog.
    if key.code == KeyCode::Char('D') && context == KeyContext::ExplorerPanel {
        return Some(AppAction::ForceDelete);
    }

    match (key.code, context) {
        (KeyCode::Tab, KeyContext::Dialog) => None,
        (KeyCode::Tab, _) => Some(AppAction::ToggleFocus),
        (KeyCode::Up | KeyCode::Char('k'), KeyContext::Dialog) => Some(AppAction::NavigateUp),
        (KeyCode::Down | KeyCode::Char('j'), KeyContext::Dialog) => Some(AppAction::NavigateDown),
        (KeyCode::Up | KeyCode::Char('k'), _) => Some(AppAction::NavigateUp),
        (KeyCode::Down | KeyCode::Char('j'), _) => Some(AppAction::NavigateDown),
        (KeyCode::Enter, _) => Some(AppAction::Enter),
        (KeyCode::Backspace, KeyContext::Dialog) => None,
        (KeyCode::Backspace, _) => Some(AppAction::Back),
        (KeyCode::Char('a'), KeyContext::PacksPanel) => Some(AppAction::Apply),
        (KeyCode::Char('v'), KeyContext::PacksPanel) => Some(AppAction::View),
        (KeyCode::Char('u'), KeyContext::InstalledPanel) => Some(AppAction::Uninstall),
        (KeyCode::Char('r'), KeyContext::ExplorerPanel) => Some(AppAction::CreateFolder),
        (KeyCode::Char('m'), KeyContext::ExplorerPanel) => Some(AppAction::Rename),
        (KeyCode::Char('d'), KeyContext::ExplorerPanel) => Some(AppAction::Delete),
        (KeyCode::Char(' '), _) => Some(AppAction::Help),
        (KeyCode::Esc, _) => Some(AppAction::Close),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn explorer_panel_maps_rename_and_delete() {
        assert_eq!(
            map_key(&key(KeyCode::Char('m')), KeyContext::ExplorerPanel),
            Some(AppAction::Rename)
        );
        assert_eq!(
            map_key(&key(KeyCode::Char('d')), KeyContext::ExplorerPanel),
            Some(AppAction::Delete)
        );
        assert_eq!(
            map_key(&key(KeyCode::Char('D')), KeyContext::ExplorerPanel),
            Some(AppAction::ForceDelete)
        );
    }

    #[test]
    fn packs_panel_has_no_rename_delete() {
        assert_eq!(
            map_key(&key(KeyCode::Char('m')), KeyContext::PacksPanel),
            None
        );
        assert_eq!(
            map_key(&key(KeyCode::Char('d')), KeyContext::PacksPanel),
            None
        );
    }

    #[test]
    fn installed_panel_maps_uninstall_only() {
        assert_eq!(
            map_key(&key(KeyCode::Char('u')), KeyContext::InstalledPanel),
            Some(AppAction::Uninstall)
        );
        assert_eq!(
            map_key(&key(KeyCode::Char('a')), KeyContext::InstalledPanel),
            None
        );
        assert_eq!(
            map_key(&key(KeyCode::Char('v')), KeyContext::InstalledPanel),
            None
        );
        assert_eq!(
            map_key(&key(KeyCode::Char('u')), KeyContext::PacksPanel),
            None
        );
    }
}
