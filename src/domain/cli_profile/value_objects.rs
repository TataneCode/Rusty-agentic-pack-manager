use std::fmt;

use crate::domain::pack::error::DomainError;

/// Identifier of a target CLI (e.g. "vibe", "claude"). Unlike the v1 closed
/// enum, any well-formed id is accepted: whether a CLI is usable is decided
/// by the CLI profile registry (E02), not by the compiler.
///
/// Non-empty, filesystem-safe: lowercase alphanumeric plus `-`/`_`, must
/// start with a letter (same rule as `PackName`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CliId(String);

impl CliId {
    pub fn new(value: &str) -> Result<CliId, DomainError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(DomainError::InvalidCliId(value.to_string()));
        }
        let valid = value.chars().next().is_some_and(|c| c.is_ascii_lowercase())
            && value
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
        if !valid || value.len() > 64 {
            return Err(DomainError::InvalidCliId(value.to_string()));
        }
        Ok(CliId(value.to_string()))
    }

    /// Parse a raw identifier (manifest, index, CLI flag): trimmed and
    /// lowercased, then validated.
    pub fn parse(value: &str) -> Result<CliId, DomainError> {
        CliId::new(&value.trim().to_ascii_lowercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CliId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_id_accepts_valid() {
        assert!(CliId::new("vibe").is_ok());
        assert!(CliId::new("claude").is_ok());
        assert!(CliId::new("opencode").is_ok());
        assert!(CliId::new("code-review-2").is_ok());
        assert_eq!(
            CliId::new("Claude").unwrap_err().to_string(),
            "invalid CLI id: Claude"
        );
    }

    #[test]
    fn cli_id_rejects_invalid() {
        assert!(CliId::new("").is_err());
        assert!(CliId::new("  ").is_err());
        assert!(CliId::new("Claude").is_err());
        assert!(CliId::new("2code").is_err());
        assert!(CliId::new("my cli").is_err());
        assert!(CliId::new("cli!").is_err());
        assert!(CliId::new(&"a".repeat(65)).is_err());
    }

    #[test]
    fn cli_id_parse_lowercases_and_trims() {
        assert_eq!(CliId::parse("  Gemini ").unwrap().as_str(), "gemini");
        assert!(CliId::parse("bad id").is_err());
    }

    #[test]
    fn cli_id_display() {
        assert_eq!(CliId::new("gemini").unwrap().to_string(), "gemini");
    }
}
