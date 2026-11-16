use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Logical actions, decoupled from raw keys (pattern rusty-console AppAction).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    Quit,
    ToggleFocus,     // Tab: next panel
    ToggleFocusBack, // Shift+Tab: previous panel
    FocusPacks,      // Shift+P: direct access to the packs panel
    FocusInstalled,  // Shift+I: direct access to the installed panel
    FocusExplorer,   // Shift+E: direct access to the explorer panel
    NavigateUp,
    NavigateDown,
    Enter,
    Back,
    Apply,          // 'a' on packs panel: install in the explorer's current dir
    PlanPreview,    // 'p' on packs panel: preview the install plan (E04)
    View,           // 'v' on packs panel
    Uninstall,      // 'u' on installed panel: remove a recorded installation
    CreateFolder,   // 'r' on explorer panel
    Rename,         // 'm' on explorer panel
    Delete,         // 'd' on explorer panel (confirm dialog)
    ForceDelete,    // 'D' (shift-d) on explorer panel: force non-empty folder
    ToggleCliGroup, // 'c' on packs panel: collapse/expand the row's CLI group
    Help,           // spacebar
    Close,          // esc: close popup/dialog/exit mode
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

    // Normalize Shift+letter to its uppercase code: the kitty keyboard
    // protocol reports Shift+p as Char('p') + SHIFT, legacy terminals as
    // Char('P'). Both must reach the same action.
    let code = match key.code {
        KeyCode::Char(c)
            if key.modifiers.contains(KeyModifiers::SHIFT) && c.is_ascii_lowercase() =>
        {
            KeyCode::Char(c.to_ascii_uppercase())
        }
        other => other,
    };

    // Shift-D forces deletion without the confirmation dialog.
    if code == KeyCode::Char('D') && context == KeyContext::ExplorerPanel {
        return Some(AppAction::ForceDelete);
    }

    match (code, context) {
        (KeyCode::Tab, KeyContext::Dialog) => None,
        (KeyCode::Tab, _) => Some(AppAction::ToggleFocus),
        (KeyCode::BackTab, KeyContext::Dialog) => None,
        (KeyCode::BackTab, _) => Some(AppAction::ToggleFocusBack),
        // Direct panel access (Shift+P / Shift+I / Shift+E), any panel.
        (KeyCode::Char('P'), KeyContext::Dialog) => None,
        (KeyCode::Char('P'), _) => Some(AppAction::FocusPacks),
        (KeyCode::Char('I'), KeyContext::Dialog) => None,
        (KeyCode::Char('I'), _) => Some(AppAction::FocusInstalled),
        (KeyCode::Char('E'), KeyContext::Dialog) => None,
        (KeyCode::Char('E'), _) => Some(AppAction::FocusExplorer),
        (KeyCode::Up | KeyCode::Char('k'), KeyContext::Dialog) => Some(AppAction::NavigateUp),
        (KeyCode::Down | KeyCode::Char('j'), KeyContext::Dialog) => Some(AppAction::NavigateDown),
        (KeyCode::Up | KeyCode::Char('k'), _) => Some(AppAction::NavigateUp),
        (KeyCode::Down | KeyCode::Char('j'), _) => Some(AppAction::NavigateDown),
        (KeyCode::Enter, _) => Some(AppAction::Enter),
        (KeyCode::Backspace, KeyContext::Dialog) => None,
        (KeyCode::Backspace, _) => Some(AppAction::Back),
        (KeyCode::Char('a'), KeyContext::PacksPanel) => Some(AppAction::Apply),
        (KeyCode::Char('p'), KeyContext::PacksPanel) => Some(AppAction::PlanPreview),
        (KeyCode::Char('v'), KeyContext::PacksPanel) => Some(AppAction::View),
        (KeyCode::Char('c'), KeyContext::PacksPanel) => Some(AppAction::ToggleCliGroup),
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

    fn shifted(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::SHIFT)
    }

    #[test]
    fn packs_panel_maps_plan_preview_on_lowercase_p() {
        assert_eq!(
            map_key(&key(KeyCode::Char('p')), KeyContext::PacksPanel),
            Some(AppAction::PlanPreview)
        );
        // Uppercase 'P' is now the direct-access shortcut, not the plan.
        assert_eq!(
            map_key(&key(KeyCode::Char('P')), KeyContext::PacksPanel),
            Some(AppAction::FocusPacks)
        );
        // 'p' stays unmapped outside the packs panel.
        assert_eq!(
            map_key(&key(KeyCode::Char('p')), KeyContext::ExplorerPanel),
            None
        );
    }

    #[test]
    fn shift_letter_is_normalized_to_uppercase() {
        // Kitty protocol: Char('p') + SHIFT. Legacy: Char('P') alone.
        assert_eq!(
            map_key(&shifted('p'), KeyContext::ExplorerPanel),
            Some(AppAction::FocusPacks)
        );
        assert_eq!(
            map_key(&key(KeyCode::Char('P')), KeyContext::ExplorerPanel),
            Some(AppAction::FocusPacks)
        );
        assert_eq!(
            map_key(&shifted('i'), KeyContext::PacksPanel),
            Some(AppAction::FocusInstalled)
        );
        assert_eq!(
            map_key(&shifted('e'), KeyContext::InstalledPanel),
            Some(AppAction::FocusExplorer)
        );
    }

    #[test]
    fn direct_access_shortcuts_are_disabled_in_dialogs() {
        assert_eq!(map_key(&shifted('p'), KeyContext::Dialog), None);
        assert_eq!(map_key(&key(KeyCode::Char('I')), KeyContext::Dialog), None);
        assert_eq!(map_key(&key(KeyCode::Char('E')), KeyContext::Dialog), None);
    }

    #[test]
    fn backtab_maps_to_reverse_focus_outside_dialogs() {
        assert_eq!(
            map_key(&key(KeyCode::BackTab), KeyContext::PacksPanel),
            Some(AppAction::ToggleFocusBack)
        );
        assert_eq!(
            map_key(&key(KeyCode::BackTab), KeyContext::Viewer),
            Some(AppAction::ToggleFocusBack)
        );
        assert_eq!(map_key(&key(KeyCode::BackTab), KeyContext::Dialog), None);
    }

    #[test]
    fn packs_panel_maps_cli_group_toggle_on_c() {
        assert_eq!(
            map_key(&key(KeyCode::Char('c')), KeyContext::PacksPanel),
            Some(AppAction::ToggleCliGroup)
        );
        assert_eq!(
            map_key(&key(KeyCode::Char('c')), KeyContext::ExplorerPanel),
            None
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
