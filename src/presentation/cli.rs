use std::io::{IsTerminal, Write};
use std::path::PathBuf;

use crate::application::error::ApplicationError;
use crate::application::pack::dto::{InstallPlanDto, PlanEntryStatus};
use crate::application::pack::service::PackService;
use crate::application::pack_create::dto::{CreateItemDto, CreatePackDto};
use crate::application::pack_create::service::PackCreationService;

const USAGE: &str = "\
rapm — Rusty Agentic Pack Manager

Usage:
  rapm                          Launch the TUI
  rapm list [options]           List available packs (or installed ones)
  rapm show <pack>              Show a pack's manifest and items
  rapm install <pack> [options] Install a pack into a target directory
  rapm uninstall <pack> [options] Uninstall a pack from a target directory
  rapm pack create [options]    Create a pack in the user library and exit
  rapm help                     Show this help

`list` options:
  --cli <cli>              Only packs supporting this CLI
  --installed              List installations recorded in the index instead
  --dest <dir>             Target directory (default: current directory); with --installed, the repository whose packs_index.toml is read

`install` options:
  --dest <dir>             Target directory (default: current directory)
  --cli <cli>              Target CLI (default: the pack's first declared CLI)
  --force                  Overwrite existing destinations
  --dry-run                Print the install plan (status  source -> destination)
                           without writing anything; exit 3 when a destination
                           already exists and --force is absent

`uninstall` options:
  --dest <dir>             Target directory (default: current directory)
  --cli <cli>              Target CLI (default: the pack's first declared CLI)

Exit codes: 0 success, 1 runtime error, 2 usage error, 3 dry-run with existing destinations without --force.

`pack create` options:
  --name <name>            Pack name (lowercase, digits, - and _); prompted if omitted
  --version <version>      Semver version (default: 1.0.0)
  --description <text>     Pack description; prompted if omitted
  --cli <id,...>           Target CLIs (any CLI with a loaded profile, default: union of the items' destinations)
  --force                  Replace an existing pack of the same name
  --skill <path>           Add a skill (file or directory); repeatable
  --agent <path>           Add an agent (file or directory); repeatable
  --command <path>         Add a command (file or directory); repeatable
  --dest <cli>=<dest>      Install destination for any profiled CLI (applies to the last item); repeatable
  --vibe-dest <dest>       Shortcut for --dest vibe=<dest> (kept for compatibility)
  --gemini-dest <dest>     Shortcut for --dest gemini=<dest> (kept for compatibility)
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
    List(ListOptions),
    Show { pack: String },
    Install(InstallOptions),
    Uninstall(UninstallOptions),
}

/// Options of `rapm list`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ListOptions {
    pub cli: Option<String>,
    pub installed: bool,
    pub dest: Option<PathBuf>,
}

/// Options of `rapm install <pack>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstallOptions {
    pub pack: String,
    pub dest: Option<PathBuf>,
    pub cli: Option<String>,
    pub force: bool,
    pub dry_run: bool,
}

/// Options of `rapm uninstall <pack>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UninstallOptions {
    pub pack: String,
    pub dest: Option<PathBuf>,
    pub cli: Option<String>,
}

/// Run the CLI: parse `args`, execute, and return the process exit code.
/// `creation` and `pack` services and the library wiring are built at the
/// DI point (`main.rs`).
pub fn run(
    args: &[String],
    creation: &PackCreationService,
    pack: &PackService,
) -> Result<i32, ApplicationError> {
    run_with(args, creation, pack, &mut prompt_stdin)
}

fn run_with(
    args: &[String],
    creation: &PackCreationService,
    pack: &PackService,
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
            let result = creation.create(&dto)?;
            println!(
                "created pack '{}' in {} ({} item(s))",
                result.pack_name,
                result.pack_dir.display(),
                result.copied_count
            );
            Ok(0)
        }
        Some(CliCommand::List(options)) => {
            run_list(pack, &options)?;
            Ok(0)
        }
        Some(CliCommand::Show { pack: name }) => {
            run_show(pack, &name)?;
            Ok(0)
        }
        Some(CliCommand::Install(options)) => {
            let dest = resolve_dest(&options.dest)?;
            let cli = match &options.cli {
                Some(cli) => cli.clone(),
                None => pack.default_cli(&options.pack)?,
            };
            if options.dry_run {
                let plan = pack.plan_install(&options.pack, &cli, &dest)?;
                print_plan(&plan);
                return Ok(dry_run_exit_code(&plan, options.force));
            }
            let result = pack.install(&options.pack, &cli, &dest, options.force)?;
            println!(
                "installed '{}' for {} into {} ({} path(s))",
                result.pack_name, cli, result.target_root, result.copied_count
            );
            Ok(0)
        }
        Some(CliCommand::Uninstall(options)) => {
            let dest = resolve_dest(&options.dest)?;
            let cli = match &options.cli {
                Some(cli) => cli.clone(),
                None => pack.default_cli(&options.pack)?,
            };
            let result = pack.uninstall(&options.pack, &cli, &dest)?;
            println!(
                "uninstalled '{}' for {} from {} ({} path(s) removed)",
                result.pack_name, result.cli, dest, result.removed_count
            );
            Ok(0)
        }
    }
}

/// Print an install plan (E04 dry-run): one line per entry,
/// `<status>  <source> -> <destination>`.
fn print_plan(plan: &InstallPlanDto) {
    println!(
        "plan for '{}' ({}) in {}:",
        plan.pack_name, plan.cli, plan.target_root
    );
    for entry in &plan.entries {
        println!(
            "{}  {} -> {}",
            entry.status.as_str(),
            entry.source,
            entry.destination
        );
    }
}

/// Exit code of `install --dry-run` (E04): 3 when at least one destination
/// already exists (status other than `ok`) and `--force` is absent — the real
/// install would abort; 0 otherwise (with `--force` the plan is printed
/// unchanged and the install would overwrite).
fn dry_run_exit_code(plan: &InstallPlanDto, force: bool) -> i32 {
    let has_existing = plan
        .entries
        .iter()
        .any(|entry| entry.status != PlanEntryStatus::Ok);
    if has_existing && !force {
        3
    } else {
        0
    }
}

/// `--dest` defaults to the current directory (same convention as the TUI,
/// which starts in the cwd).
fn resolve_dest(dest: &Option<PathBuf>) -> Result<String, ApplicationError> {
    match dest {
        Some(dir) => Ok(dir.to_string_lossy().into_owned()),
        None => std::env::current_dir()
            .map(|dir| dir.to_string_lossy().into_owned())
            .map_err(|err| ApplicationError::Io {
                path: ".".to_string(),
                source: err,
            }),
    }
}

fn run_list(pack: &PackService, options: &ListOptions) -> Result<(), ApplicationError> {
    if options.installed {
        let dest = resolve_dest(&options.dest)?;
        let entries = pack.installed_in(&dest)?;
        let entries: Vec<_> = match &options.cli {
            Some(cli) => entries.into_iter().filter(|e| e.cli == *cli).collect(),
            None => entries,
        };
        for entry in entries {
            let paths = entry
                .paths
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join(", ");
            println!("{}  {}  {}", entry.name, entry.cli, paths);
        }
    } else {
        let packs = pack.list()?;
        for summary in packs.iter().filter(|p| match &options.cli {
            Some(cli) => p.clis.iter().any(|c| c == cli),
            None => true,
        }) {
            let clis = summary.clis.join(",");
            println!(
                "{}  {}  {}  {} item(s)",
                summary.name, summary.version, clis, summary.item_count
            );
        }
    }
    Ok(())
}

fn run_show(pack: &PackService, name: &str) -> Result<(), ApplicationError> {
    let detail = pack.detail(name)?;
    println!("name: {}", detail.name);
    println!("version: {}", detail.version);
    println!("description: {}", detail.description);
    println!("clis: {}", detail.clis.join(", "));
    for item in &detail.items {
        let destinations = item
            .destinations
            .iter()
            .map(|(cli, dest)| format!("{cli}={dest}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("{}  {}  {}", item.kind, item.source, destinations);
    }
    Ok(())
}

/// True when `path` looks like a CLI flag (starts with `-`).
fn is_flag(token: &str) -> bool {
    token.starts_with('-') && token.len() > 1
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
        "list" => parse_list(&args[1..]).map(Some),
        "show" => parse_show(&args[1..]).map(Some),
        "install" => parse_install(&args[1..]).map(Some),
        "uninstall" => parse_uninstall(&args[1..]).map(Some),
        other => Err(ApplicationError::Usage(format!("unknown command: {other}"))),
    }
}

fn parse_list(tokens: &[String]) -> Result<CliCommand, ApplicationError> {
    let mut options = ListOptions::default();
    let mut parser = TokenParser { tokens, pos: 0 };
    while let Some(token) = parser.peek().map(str::to_string) {
        let flag = token.split('=').next().unwrap_or("");
        match flag {
            "-h" | "--help" => return Ok(CliCommand::Help),
            "--cli" => options.cli = Some(parser.value(flag)?),
            "--installed" => {
                parser.advance();
                options.installed = true;
            }
            "--dest" => options.dest = Some(PathBuf::from(parser.value(flag)?)),
            _ => return Err(ApplicationError::Usage(format!("unknown option: {token}"))),
        }
    }
    Ok(CliCommand::List(options))
}

fn parse_show(tokens: &[String]) -> Result<CliCommand, ApplicationError> {
    let mut parser = TokenParser { tokens, pos: 0 };
    match parser.peek() {
        None => Err(ApplicationError::Usage(
            "missing pack name: rapm show <pack>".to_string(),
        )),
        Some("-h" | "--help") => Ok(CliCommand::Help),
        Some(token) if is_flag(token) => {
            Err(ApplicationError::Usage(format!("unknown option: {token}")))
        }
        Some(_) => {
            parser.advance();
            let pack = tokens[0].clone();
            if let Some(extra) = parser.peek() {
                return Err(ApplicationError::Usage(format!(
                    "unexpected argument: {extra}"
                )));
            }
            Ok(CliCommand::Show { pack })
        }
    }
}

fn parse_install(tokens: &[String]) -> Result<CliCommand, ApplicationError> {
    match parse_pack_target(tokens, true)? {
        None => Ok(CliCommand::Help),
        Some(target) => Ok(CliCommand::Install(InstallOptions {
            pack: target.pack,
            dest: target.dest,
            cli: target.cli,
            force: target.force,
            dry_run: target.dry_run,
        })),
    }
}

fn parse_uninstall(tokens: &[String]) -> Result<CliCommand, ApplicationError> {
    match parse_pack_target(tokens, false)? {
        None => Ok(CliCommand::Help),
        Some(target) => Ok(CliCommand::Uninstall(UninstallOptions {
            pack: target.pack,
            dest: target.dest,
            cli: target.cli,
        })),
    }
}

/// Positional pack name plus the shared `--dest`/`--cli` options of
/// `install` and `uninstall` (plus install-only flags).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PackTargetOptions {
    pack: String,
    dest: Option<PathBuf>,
    cli: Option<String>,
    force: bool,
    dry_run: bool,
}

/// Shared parser of `install`/`uninstall`: one positional pack name plus
/// `--dest`, `--cli` and (for install only) `--force`/`--dry-run`.
/// `None` = help requested.
fn parse_pack_target(
    tokens: &[String],
    allow_install_flags: bool,
) -> Result<Option<PackTargetOptions>, ApplicationError> {
    let mut target = PackTargetOptions::default();
    let mut parser = TokenParser { tokens, pos: 0 };
    while let Some(token) = parser.peek().map(str::to_string) {
        let flag = token.split('=').next().unwrap_or("");
        match flag {
            "-h" | "--help" => return Ok(None),
            "--dest" => target.dest = Some(PathBuf::from(parser.value(flag)?)),
            "--cli" => target.cli = Some(parser.value(flag)?),
            "--force" if allow_install_flags => {
                parser.advance();
                target.force = true;
            }
            "--dry-run" if allow_install_flags => {
                parser.advance();
                target.dry_run = true;
            }
            _ if is_flag(&token) => {
                return Err(ApplicationError::Usage(format!("unknown option: {token}")))
            }
            _ => {
                if !target.pack.is_empty() {
                    return Err(ApplicationError::Usage(format!(
                        "unexpected argument: {token}"
                    )));
                }
                parser.advance();
                target.pack = token;
            }
        }
    }
    if target.pack.is_empty() {
        return Err(ApplicationError::Usage(
            "missing pack name: rapm install <pack> / rapm uninstall <pack>".to_string(),
        ));
    }
    Ok(Some(target))
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
            "--dest" => {
                let raw = parser.value(flag)?;
                let Some((cli, dest)) = raw.split_once('=') else {
                    return Err(ApplicationError::Usage(format!(
                        "--dest expects <cli>=<dest>, got: {raw}"
                    )));
                };
                let Some(item) = options.items.last_mut() else {
                    return Err(ApplicationError::Usage(format!(
                        "{flag} applies to an item: pass --skill/--agent/--command first"
                    )));
                };
                item.destinations.push((cli.to_string(), dest.to_string()));
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
    fn create_parses_generic_dest_flag() {
        let command = parse(&args(&[
            "pack",
            "create",
            "--name",
            "my-pack",
            "--cli",
            "claude",
            "--skill",
            "./skills/tdd",
            "--dest",
            "claude=.claude/skills/tdd/SKILL.md",
        ]))
        .unwrap();
        let Some(CliCommand::Create(options)) = command else {
            panic!("expected create");
        };
        assert_eq!(options.clis, vec!["claude".to_string()]);
        assert_eq!(
            options.items[0].destinations,
            vec![(
                "claude".to_string(),
                ".claude/skills/tdd/SKILL.md".to_string()
            )]
        );
    }

    #[test]
    fn dest_flag_requires_cli_equals_dest() {
        let err = parse(&args(&[
            "pack", "create", "--skill", "./tdd", "--dest", "claude",
        ]))
        .unwrap_err();
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
    fn list_parses_cli_installed_and_dest() {
        let command = parse(&args(&[
            "list",
            "--cli",
            "vibe",
            "--installed",
            "--dest=/tmp/repo",
        ]))
        .unwrap();
        assert_eq!(
            command,
            Some(CliCommand::List(ListOptions {
                cli: Some("vibe".to_string()),
                installed: true,
                dest: Some(PathBuf::from("/tmp/repo")),
            }))
        );
    }

    #[test]
    fn list_defaults_to_available_packs() {
        let command = parse(&args(&["list"])).unwrap();
        assert_eq!(
            command,
            Some(CliCommand::List(ListOptions {
                cli: None,
                installed: false,
                dest: None,
            }))
        );
    }

    #[test]
    fn list_rejects_unknown_option() {
        assert!(matches!(
            parse(&args(&["list", "--wat"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
    }

    #[test]
    fn show_takes_one_pack_name() {
        assert_eq!(
            parse(&args(&["show", "tdd"])).unwrap(),
            Some(CliCommand::Show {
                pack: "tdd".to_string()
            })
        );
        assert!(matches!(
            parse(&args(&["show"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
        assert!(matches!(
            parse(&args(&["show", "tdd", "extra"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
        assert!(matches!(
            parse(&args(&["show", "--cli", "vibe"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
    }

    #[test]
    fn install_parses_pack_dest_cli_and_force() {
        let command = parse(&args(&[
            "install",
            "tdd",
            "--dest",
            "/tmp/repo",
            "--cli=vibe",
            "--force",
        ]))
        .unwrap();
        assert_eq!(
            command,
            Some(CliCommand::Install(InstallOptions {
                pack: "tdd".to_string(),
                dest: Some(PathBuf::from("/tmp/repo")),
                cli: Some("vibe".to_string()),
                force: true,
                dry_run: false,
            }))
        );
    }

    #[test]
    fn install_parses_dry_run() {
        let command = parse(&args(&[
            "install",
            "tdd",
            "--dest",
            "/tmp/repo",
            "--dry-run",
        ]))
        .unwrap();
        assert_eq!(
            command,
            Some(CliCommand::Install(InstallOptions {
                pack: "tdd".to_string(),
                dest: Some(PathBuf::from("/tmp/repo")),
                cli: None,
                force: false,
                dry_run: true,
            }))
        );
    }

    #[test]
    fn dry_run_exit_code_is_3_only_for_existing_destinations_without_force() {
        let entry = |status| crate::application::pack::dto::PlanEntryDto {
            source: "skills/tdd/SKILL.md".to_string(),
            destination: ".agents/skills/tdd".to_string(),
            status,
        };
        let plan = |statuses: Vec<PlanEntryStatus>| InstallPlanDto {
            pack_name: "tdd".to_string(),
            cli: "vibe".to_string(),
            target_root: "/repo".to_string(),
            entries: statuses.into_iter().map(entry).collect(),
        };
        assert_eq!(
            dry_run_exit_code(&plan(vec![PlanEntryStatus::Ok]), false),
            0
        );
        assert_eq!(
            dry_run_exit_code(&plan(vec![PlanEntryStatus::Collision]), false),
            3
        );
        assert_eq!(
            dry_run_exit_code(&plan(vec![PlanEntryStatus::Untracked]), false),
            3
        );
        assert_eq!(
            dry_run_exit_code(&plan(vec![PlanEntryStatus::Collision]), true),
            0
        );
    }

    #[test]
    fn install_defaults_dest_cli_and_force() {
        let command = parse(&args(&["install", "tdd"])).unwrap();
        assert_eq!(
            command,
            Some(CliCommand::Install(InstallOptions {
                pack: "tdd".to_string(),
                dest: None,
                cli: None,
                force: false,
                dry_run: false,
            }))
        );
    }

    #[test]
    fn install_requires_a_pack_name_and_rejects_extras() {
        assert!(matches!(
            parse(&args(&["install"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
        assert!(matches!(
            parse(&args(&["install", "tdd", "other"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
        assert!(matches!(
            parse(&args(&["install", "--wat", "tdd"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
    }

    #[test]
    fn uninstall_parses_pack_dest_cli_and_rejects_force() {
        let command = parse(&args(&[
            "uninstall",
            "tdd",
            "--dest",
            "/tmp/repo",
            "--cli",
            "gemini",
        ]))
        .unwrap();
        assert_eq!(
            command,
            Some(CliCommand::Uninstall(UninstallOptions {
                pack: "tdd".to_string(),
                dest: Some(PathBuf::from("/tmp/repo")),
                cli: Some("gemini".to_string()),
            }))
        );
        assert!(matches!(
            parse(&args(&["uninstall", "tdd", "--force"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
        assert!(matches!(
            parse(&args(&["uninstall", "tdd", "--dry-run"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
    }

    #[test]
    fn headless_subcommands_support_help() {
        for invocation in [
            args(&["list", "--help"]),
            args(&["show", "--help"]),
            args(&["install", "-h"]),
            args(&["uninstall", "--help"]),
        ] {
            assert_eq!(
                parse(&invocation).unwrap(),
                Some(CliCommand::Help),
                "invocation: {invocation:?}"
            );
        }
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
