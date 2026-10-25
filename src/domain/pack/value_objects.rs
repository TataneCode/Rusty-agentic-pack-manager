use std::collections::BTreeMap;
use std::fmt;

use crate::domain::pack::error::DomainError;

/// CLI able to consume an agentic pack. Extensible: v1 ships Vibe and Gemini.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TargetCli {
    Vibe,
    Gemini,
}

impl TargetCli {
    pub const ALL: [TargetCli; 2] = [TargetCli::Vibe, TargetCli::Gemini];

    pub fn as_str(&self) -> &'static str {
        match self {
            TargetCli::Vibe => "vibe",
            TargetCli::Gemini => "gemini",
        }
    }

    pub fn parse(value: &str) -> Result<TargetCli, DomainError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "vibe" => Ok(TargetCli::Vibe),
            "gemini" => Ok(TargetCli::Gemini),
            other => Err(DomainError::UnknownCli(other.to_string())),
        }
    }

    /// Display label used by the UI (e.g. "Vibe CLI").
    pub fn label(&self) -> &'static str {
        match self {
            TargetCli::Vibe => "Vibe CLI",
            TargetCli::Gemini => "Gemini CLI",
        }
    }
}

impl fmt::Display for TargetCli {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Non-empty, filesystem-safe pack name: lowercase alphanumeric plus `-`/`_`,
/// must start with a letter.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackName(String);

impl PackName {
    pub fn new(value: &str) -> Result<PackName, DomainError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(DomainError::EmptyName);
        }
        let valid = value.chars().next().is_some_and(|c| c.is_ascii_lowercase())
            && value
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
        if !valid || value.len() > 64 {
            return Err(DomainError::InvalidName(value.to_string()));
        }
        Ok(PackName(value.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Semantic version (MAJOR.MINOR.PATCH), pre-release suffix allowed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackVersion(String);

impl PackVersion {
    pub fn new(value: &str) -> Result<PackVersion, DomainError> {
        let value = value.trim();
        let valid = value.len() <= 32 && Self::is_semver(value);
        if !valid {
            return Err(DomainError::InvalidVersion(value.to_string()));
        }
        Ok(PackVersion(value.to_string()))
    }

    fn is_semver(value: &str) -> bool {
        let mut parts = value.splitn(2, '-');
        let core = parts.next().unwrap_or("");
        let numbers = core.split('.');
        let mut count = 0;
        for n in numbers {
            if n.is_empty() || !n.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
            count += 1;
        }
        if count != 3 {
            return false;
        }
        match parts.next() {
            None => true,
            Some(suffix) => !suffix.is_empty(),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PackVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Kind of an agentic item shipped in a pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackItemKind {
    Skill,
    Agent,
    Command,
}

impl PackItemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PackItemKind::Skill => "skill",
            PackItemKind::Agent => "agent",
            PackItemKind::Command => "command",
        }
    }

    pub fn parse(value: &str) -> Result<PackItemKind, DomainError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "skill" => Ok(PackItemKind::Skill),
            "agent" => Ok(PackItemKind::Agent),
            "command" => Ok(PackItemKind::Command),
            other => Err(DomainError::UnknownCli(other.to_string())),
        }
    }
}

/// Per-CLI destinations of one pack item, e.g.
/// vibe -> ".agents/skills/tdd/", gemini -> ".gemini/commands/tdd".
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TargetCliMap(BTreeMap<TargetCli, String>);

impl TargetCliMap {
    pub fn from_pairs(pairs: &[(TargetCli, &str)]) -> TargetCliMap {
        TargetCliMap(
            pairs
                .iter()
                .map(|(cli, dest)| (*cli, (*dest).to_string()))
                .collect(),
        )
    }

    pub fn get(&self, cli: TargetCli) -> Option<&str> {
        self.0.get(&cli).map(|s| s.as_str())
    }

    pub fn insert(&mut self, cli: TargetCli, destination: String) {
        self.0.insert(cli, destination);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (TargetCli, &str)> {
        self.0.iter().map(|(cli, dest)| (*cli, dest.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_name_accepts_valid() {
        assert!(PackName::new("tdd").is_ok());
        assert!(PackName::new("code-review-2").is_ok());
        assert!(PackName::new("a").is_ok());
    }

    #[test]
    fn pack_name_rejects_invalid() {
        assert!(PackName::new("").is_err());
        assert!(PackName::new("  ").is_err());
        assert!(PackName::new("Tdd").is_err());
        assert!(PackName::new("2tdd").is_err());
        assert!(PackName::new("tdd!").is_err());
    }

    #[test]
    fn pack_version_accepts_semver() {
        assert!(PackVersion::new("1.0.0").is_ok());
        assert!(PackVersion::new("0.2.1").is_ok());
        assert!(PackVersion::new("2.0.0-rc.1").is_ok());
    }

    #[test]
    fn pack_version_rejects_invalid() {
        assert!(PackVersion::new("1.0").is_err());
        assert!(PackVersion::new("v1.0.0").is_err());
        assert!(PackVersion::new("").is_err());
        assert!(PackVersion::new("1.0.0.0").is_err());
    }

    #[test]
    fn target_cli_parse_and_display() {
        assert_eq!(TargetCli::parse("vibe").unwrap(), TargetCli::Vibe);
        assert_eq!(TargetCli::parse("Gemini").unwrap(), TargetCli::Gemini);
        assert!(TargetCli::parse("copilot").is_err());
        assert_eq!(TargetCli::Vibe.to_string(), "vibe");
        assert_eq!(TargetCli::Vibe.label(), "Vibe CLI");
    }

    #[test]
    fn pack_item_kind_parse() {
        assert_eq!(PackItemKind::parse("skill").unwrap(), PackItemKind::Skill);
        assert_eq!(
            PackItemKind::parse("COMMAND").unwrap(),
            PackItemKind::Command
        );
        assert!(PackItemKind::parse("macro").is_err());
    }
}
