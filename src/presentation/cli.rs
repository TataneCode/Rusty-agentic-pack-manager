use std::io::{IsTerminal, Write};
use std::path::PathBuf;

use crate::application::error::ApplicationError;
use crate::application::pack_create::dto::{CreateItemDto, CreatePackDto};
use crate::application::pack_create::service::PackCreationService;

const USAGE: &str = "\
rapm — Rusty Agentic Pack Manager

Usage:
  rapm                          Launch the TUI
  rapm pack create [options]    Create a pack in the user library and exit
  rapm help                     Show this help

`pack create` options:
  --name <name>            Pack name (lowercase, digits, - and _); prompted if omitted
  --version <version>      Semver version (default: 1.0.0)
  --description <text>     Pack description; prompted if omitted
  --cli <vibe,gemini>      Target CLIs (default: union of the items' destinations)
  --force                  Replace an existing pack of the same name
  --skill <path>           Add a skill (file or directory); repeatable
  --agent <path>           Add an agent (file or directory); repeatable
  --command <path>         Add a command (file or directory); repeatable
  --vibe-dest <dest>       Install destination for Vibe CLI (applies to the last item)
  --gemini-dest <dest>     Install destination for Gemini CLI (applies to the last item)
  -h, --help               Show this help

Items are copied into the pack under <kind>s/<name>/ (skills/, agents/,
commands/). The pack is written to the user library
($XDG_DATA_HOME/rapm/packs or ~/.local/share/rapm/packs), which takes
precedence over the system library.

Example:
  rapm pack create --name my-pack \\
    --skill ./skills/tdd --vibe-dest .agents/skills/tdd/ --gemini-dest .gemini/skills/tdd/ \\
    --command ./commands/review.toml --vibe-dest .vibe/commands/review.toml --gemini-dest .gemini/commands/review.toml
";

/// Whether the invocation is a CLI call (any argument) rather than a bare
/// TUI launch.
pub fn is_cli_invocation(args: &[String]) -> bool {
    !args.is_empty()
}

/// Options of `rapm pack create`, as parsed from the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateOptions {
    pub name: Option<String>,
    pub version: String,
    pub description: Option<String>,
    pub clis: Vec<String>,
    pub items: Vec<CreateItemDto>,
    pub overwrite: bool,
}

impl Default for CreateOptions {
    fn default() -> Self {
        CreateOptions {
            name: None,
            version: "1.0.0".to_string(),
            description: None,
            clis: Vec::new(),
            items: Vec::new(),
            overwrite: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    Help,
    Create(CreateOptions),
}

/// Run the CLI: parse `args`, execute, and return the process exit code.
/// `service` and the library wiring are built at the DI point (`main.rs`).
pub fn run(args: &[String], service: &PackCreationService) -> Result<i32, ApplicationError> {
    run_with(args, service, &mut prompt_stdin)
}

fn run_with(
    args: &[String],
    service: &PackCreationService,
    prompt: &mut dyn FnMut(&str) -> Option<String>,
) -> Result<i32, ApplicationError> {
    match parse(args)? {
        None => Ok(0),
        Some(CliCommand::Help) => {
            print!("{USAGE}");
            Ok(0)
        }
        Some(CliCommand::Create(options)) => {
            let dto = resolve_dto(&options, prompt)?;
            let result = service.create(&dto)?;
            println!(
                "created pack '{}' in {} ({} item(s))",
                result.pack_name,
                result.pack_dir.display(),
                result.copied_count
            );
            Ok(0)
        }
    }
}

/// Fill the interactive fields of the DTO: `--name` and `--description`
/// prompt on the terminal when omitted; without a terminal, a missing name
/// is a usage error and a missing description defaults to empty.
fn resolve_dto(
    options: &CreateOptions,
    prompt: &mut dyn FnMut(&str) -> Option<String>,
) -> Result<CreatePackDto, ApplicationError> {
    let name = match &options.name {
        Some(name) => name.clone(),
        None => prompt("Pack name: ").ok_or_else(|| {
            ApplicationError::Usage(
                "missing pack name: pass --name <name> (or run in a terminal to be prompted)"
                    .to_string(),
            )
        })?,
    };
    let description = match &options.description {
        Some(description) => description.clone(),
        None => prompt("Description (optional): ").unwrap_or_default(),
    };
    Ok(CreatePackDto {
        name,
        version: options.version.clone(),
        description,
        clis: options.clis.clone(),
        items: options.items.clone(),
        overwrite: options.overwrite,
    })
}

/// Parse the CLI arguments (without the binary name). `None` = no
/// arguments: launch the TUI.
pub fn parse(args: &[String]) -> Result<Option<CliCommand>, ApplicationError> {
    let Some(first) = args.first() else {
        return Ok(None);
    };
    match first.as_str() {
        "-h" | "--help" | "help" => Ok(Some(CliCommand::Help)),
        "pack" => match args.get(1).map(String::as_str) {
            None | Some("-h" | "--help" | "help") => Ok(Some(CliCommand::Help)),
            Some("create") => parse_create(&args[2..]).map(Some),
            Some(other) => Err(ApplicationError::Usage(format!(
                "unknown 'pack' subcommand: {other}"
            ))),
        },
        other => Err(ApplicationError::Usage(format!("unknown command: {other}"))),
    }
}

fn parse_create(tokens: &[String]) -> Result<CliCommand, ApplicationError> {
    let mut options = CreateOptions::default();
    let mut parser = TokenParser { tokens, pos: 0 };
    while let Some(token) = parser.peek().map(str::to_string) {
        // Dispatch on the flag name, so `--flag=value` reaches the same arm.
        let flag = token.split('=').next().unwrap_or("");
        match flag {
            "-h" | "--help" => return Ok(CliCommand::Help),
            "--name" => options.name = Some(parser.value(flag)?),
            "--version" => options.version = parser.value(flag)?,
            "--description" => options.description = Some(parser.value(flag)?),
            "--cli" => {
                let raw = parser.value(flag)?;
                options.clis = raw.split(',').map(|c| c.trim().to_string()).collect();
            }
            "--force" => {
                parser.advance();
                options.overwrite = true;
            }
            "--skill" | "--agent" | "--command" => {
                let source = parser.value(flag)?;
                let kind = flag.trim_start_matches('-').to_string();
                options.items.push(CreateItemDto {
                    kind,
                    source: PathBuf::from(source),
                    destinations: Vec::new(),
                });
            }
            "--vibe-dest" | "--gemini-dest" => {
                let dest = parser.value(flag)?;
                let cli = flag.trim_start_matches('-').trim_end_matches("-dest");
                let Some(item) = options.items.last_mut() else {
                    return Err(ApplicationError::Usage(format!(
                        "{flag} applies to an item: pass --skill/--agent/--command first"
                    )));
                };
                item.destinations.push((cli.to_string(), dest));
            }
            _ => {
                return Err(ApplicationError::Usage(format!("unknown option: {token}")));
            }
        }
    }
    if options.items.is_empty() {
        return Err(ApplicationError::Usage(
            "no items: pass at least one --skill/--agent/--command <path>".to_string(),
        ));
    }
    Ok(CliCommand::Create(options))
}

/// Sequential token reader supporting both `--flag value` and `--flag=value`.
struct TokenParser<'a> {
    tokens: &'a [String],
    pos: usize,
}

impl<'a> TokenParser<'a> {
    fn peek(&self) -> Option<&'a str> {
        self.tokens.get(self.pos).map(String::as_str)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    /// Consume the value of a flag: either the inline `--flag=value` part or
    /// the next token.
    fn value(&mut self, flag: &str) -> Result<String, ApplicationError> {
        let token = self
            .tokens
            .get(self.pos)
            .map(String::as_str)
            .ok_or_else(|| ApplicationError::Usage(format!("missing value for {flag}")))?;
        if let Some((name, inline)) = token.split_once('=') {
            if name == flag {
                let value = inline.to_string();
                self.advance();
                return Ok(value);
            }
        }
        self.advance();
        self.tokens
            .get(self.pos)
            .map(|v| {
                self.advance();
                v.clone()
            })
            .ok_or_else(|| ApplicationError::Usage(format!("missing value for {flag}")))
    }
}

/// Interactive prompt: reads one line from the terminal. Returns `None`
/// when there is no terminal (piped stdin) or the line is empty.
fn prompt_stdin(label: &str) -> Option<String> {
    if !std::io::stdin().is_terminal() {
        return None;
    }
    print!("{label}");
    std::io::stdout().flush().ok()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).ok()?;
    let line = line.trim().to_string();
    if line.is_empty() {
        None
    } else {
        Some(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_args_means_tui() {
        assert_eq!(parse(&args(&[])).unwrap(), None);
    }

    #[test]
    fn help_variants() {
        for invocation in [
            args(&["help"]),
            args(&["--help"]),
            args(&["-h"]),
            args(&["pack"]),
            args(&["pack", "help"]),
            args(&["pack", "create", "--help"]),
        ] {
            assert_eq!(
                parse(&invocation).unwrap(),
                Some(CliCommand::Help),
                "invocation: {invocation:?}"
            );
        }
    }

    #[test]
    fn unknown_commands_are_usage_errors() {
        assert!(matches!(
            parse(&args(&["frobnicate"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
        assert!(matches!(
            parse(&args(&["pack", "frobnicate"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
    }

    #[test]
    fn create_parses_items_and_destinations() {
        let command = parse(&args(&[
            "pack",
            "create",
            "--name",
            "my-pack",
            "--version=2.1.0",
            "--description",
            "A pack",
            "--cli",
            "vibe,gemini",
            "--force",
            "--skill",
            "./skills/tdd",
            "--vibe-dest",
            ".agents/skills/tdd/",
            "--gemini-dest=.gemini/skills/tdd/",
            "--command",
            "./commands/review.toml",
            "--vibe-dest",
            ".vibe/commands/review.toml",
            "--gemini-dest",
            ".gemini/commands/review.toml",
        ]))
        .unwrap();
        let Some(CliCommand::Create(options)) = command else {
            panic!("expected create");
        };
        assert_eq!(options.name.as_deref(), Some("my-pack"));
        assert_eq!(options.version, "2.1.0");
        assert_eq!(options.description.as_deref(), Some("A pack"));
        assert_eq!(options.clis, vec!["vibe".to_string(), "gemini".to_string()]);
        assert!(options.overwrite);
        assert_eq!(options.items.len(), 2);
        assert_eq!(options.items[0].kind, "skill");
        assert_eq!(options.items[0].source, PathBuf::from("./skills/tdd"));
        assert_eq!(
            options.items[0].destinations,
            vec![
                ("vibe".to_string(), ".agents/skills/tdd/".to_string()),
                ("gemini".to_string(), ".gemini/skills/tdd/".to_string()),
            ]
        );
        assert_eq!(options.items[1].kind, "command");
        // Destinations attach to the last item only.
        assert_eq!(options.items[1].destinations.len(), 2);
    }

    #[test]
    fn create_uses_default_version() {
        let command = parse(&args(&[
            "pack",
            "create",
            "--skill",
            "./tdd",
            "--vibe-dest",
            ".agents/skills/tdd/",
        ]))
        .unwrap();
        let Some(CliCommand::Create(options)) = command else {
            panic!("expected create");
        };
        assert_eq!(options.version, "1.0.0");
        assert!(!options.overwrite);
        assert!(options.name.is_none());
    }

    #[test]
    fn create_requires_at_least_one_item() {
        let err = parse(&args(&["pack", "create", "--name", "x"])).unwrap_err();
        assert!(matches!(err, ApplicationError::Usage(_)));
    }

    #[test]
    fn destination_without_item_is_a_usage_error() {
        let err = parse(&args(&["pack", "create", "--vibe-dest", ".agents/x/"])).unwrap_err();
        assert!(matches!(err, ApplicationError::Usage(_)));
    }

    #[test]
    fn unknown_option_is_a_usage_error() {
        let err = parse(&args(&[
            "pack",
            "create",
            "--skill",
            "./tdd",
            "--vibe-dest",
            ".agents/x/",
            "--wat",
        ]))
        .unwrap_err();
        assert!(matches!(err, ApplicationError::Usage(_)));
    }

    #[test]
    fn missing_value_is_a_usage_error() {
        let err = parse(&args(&["pack", "create", "--name"])).unwrap_err();
        assert!(matches!(err, ApplicationError::Usage(_)));
    }

    #[test]
    fn resolve_dto_prompts_for_missing_metadata() {
        let options = CreateOptions {
            items: vec![CreateItemDto {
                kind: "skill".to_string(),
                source: PathBuf::from("./tdd"),
                destinations: vec![("vibe".to_string(), ".agents/skills/tdd/".to_string())],
            }],
            ..CreateOptions::default()
        };
        let mut answers = std::collections::VecDeque::from(["my-pack".to_string(), String::new()]);
        let mut prompt = move |_label: &str| answers.pop_front();
        let dto = resolve_dto(&options, &mut prompt).unwrap();
        assert_eq!(dto.name, "my-pack");
        // An empty answer means no description.
        assert_eq!(dto.description, "");
    }

    #[test]
    fn resolve_dto_requires_a_name_without_terminal() {
        let options = CreateOptions::default();
        let mut prompt = |_label: &str| None;
        let err = resolve_dto(&options, &mut prompt).unwrap_err();
        assert!(matches!(err, ApplicationError::Usage(_)));
    }

    #[test]
    fn resolve_dto_keeps_explicit_metadata() {
        let options = CreateOptions {
            name: Some("my-pack".to_string()),
            version: "2.0.0".to_string(),
            description: Some("desc".to_string()),
            clis: vec!["vibe".to_string()],
            items: vec![],
            overwrite: true,
        };
        let mut prompt = |_label: &str| panic!("no prompt expected");
        let dto = resolve_dto(&options, &mut prompt).unwrap();
        assert_eq!(dto.name, "my-pack");
        assert_eq!(dto.version, "2.0.0");
        assert_eq!(dto.description, "desc");
        assert_eq!(dto.clis, vec!["vibe".to_string()]);
        assert!(dto.overwrite);
    }
}
