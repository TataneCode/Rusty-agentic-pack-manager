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
    Apply,        // 'a' on packs panel
    Install,      // 'i' in install mode
    View,         // 'v' on packs panel
    CreateFolder, // 'r' on explorer panel
    Help,         // spacebar
    Close,        // esc: close popup/dialog/exit mode
}

/// Key-to-action mapping, context-dependent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyContext {
    PacksPanel,
    ExplorerPanel,
    InstallMode,
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
        (KeyCode::Char('r'), KeyContext::ExplorerPanel) => Some(AppAction::CreateFolder),
        (KeyCode::Char('i'), KeyContext::InstallMode) => Some(AppAction::Install),
        (KeyCode::Char(' '), _) => Some(AppAction::Help),
        (KeyCode::Esc, _) => Some(AppAction::Close),
        _ => None,
    }
}
