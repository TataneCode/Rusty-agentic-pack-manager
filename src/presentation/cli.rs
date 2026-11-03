use std::path::PathBuf;

use crate::application::error::ApplicationError;
use crate::application::pack::service::PackService;
use crate::application::pack_admin::dto::AddPackDto;
use crate::application::pack_admin::service::PackAdminService;

const USAGE: &str = "\
rapm — Rusty Agentic Pack Manager

Usage:
  rapm                          Launch the TUI
  rapm install <pack> [options] Install a pack into a target directory
  rapm uninstall <pack> [options] Uninstall a pack from a target directory
  rapm add <dir> [options]      Add a directory as a pack of the library
  rapm remove <name>            Remove a pack from the library
  rapm help                     Show this help

`install` options:
  --dest <dir>             Target directory (default: current directory)
  --force                  Overwrite existing destinations

`uninstall` options:
  --dest <dir>             Target directory (default: current directory)

`add` options:
  --name <name>            Pack name (default: the directory's name)
  --force                  Replace an existing pack of the same name

The target CLI of install/uninstall is the `default_cli` of the config
file ($XDG_CONFIG_HOME/rapm/config.toml), else the pack's first declared
CLI. The pack library location is the `packs_dir` of the same file.

`add` scans the `agents/`, `skills/`, `rules/` and `commands/`
subdirectories of <dir> (each optional, at least one required), copies
them into the library and generates the pack's `pack.toml`.

Exit codes: 0 success, 1 runtime error, 2 usage error.
";

/// Whether the invocation is a CLI call (any argument) rather than a bare
/// TUI launch.
pub fn is_cli_invocation(args: &[String]) -> bool {
    !args.is_empty()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    Help,
    Install(InstallOptions),
    Uninstall(UninstallOptions),
    Add(AddOptions),
    Remove { pack: String },
}

/// Options of `rapm install <pack>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstallOptions {
    pub pack: String,
    pub dest: Option<PathBuf>,
    pub force: bool,
}

/// Options of `rapm uninstall <pack>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UninstallOptions {
    pub pack: String,
    pub dest: Option<PathBuf>,
}

/// Options of `rapm add <dir>`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AddOptions {
    pub dir: PathBuf,
    pub name: Option<String>,
    pub force: bool,
}

/// Run the CLI: parse `args`, execute, and return the process exit code.
/// `pack` and `admin` services and the library wiring are built at the DI
/// point (`main.rs`).
pub fn run(
    args: &[String],
    admin: &PackAdminService,
    pack: &PackService,
) -> Result<i32, ApplicationError> {
    match parse(args)? {
        None => Ok(0),
        Some(CliCommand::Help) => {
            print!("{USAGE}");
            Ok(0)
        }
        Some(CliCommand::Install(options)) => {
            let dest = resolve_dest(&options.dest)?;
            let cli = pack.default_cli(&options.pack)?;
            let result = pack.install(&options.pack, &cli, &dest, options.force)?;
            println!(
                "installed '{}' for {} into {} ({} path(s))",
                result.pack_name, cli, result.target_root, result.copied_count
            );
            Ok(0)
        }
        Some(CliCommand::Uninstall(options)) => {
            let dest = resolve_dest(&options.dest)?;
            let cli = pack.default_cli(&options.pack)?;
            let result = pack.uninstall(&options.pack, &cli, &dest)?;
            println!(
                "uninstalled '{}' for {} from {} ({} path(s) removed)",
                result.pack_name, result.cli, dest, result.removed_count
            );
            Ok(0)
        }
        Some(CliCommand::Add(options)) => {
            let dto = AddPackDto {
                dir: options.dir.clone(),
                name: options.name.clone(),
                overwrite: options.force,
            };
            let result = admin.add_from_dir(&dto)?;
            println!(
                "added pack '{}' from {} into {} ({} item(s))",
                result.pack_name,
                options.dir.display(),
                result.pack_dir.display(),
                result.item_count
            );
            Ok(0)
        }
        Some(CliCommand::Remove { pack: name }) => {
            let result = admin.remove(&name)?;
            println!(
                "removed pack '{}' from {}",
                result.pack_name,
                result.pack_dir.display()
            );
            Ok(0)
        }
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

/// True when `path` looks like a CLI flag (starts with `-`).
fn is_flag(token: &str) -> bool {
    token.starts_with('-') && token.len() > 1
}

/// Parse the CLI arguments (without the binary name). `None` = no
/// arguments: launch the TUI.
pub fn parse(args: &[String]) -> Result<Option<CliCommand>, ApplicationError> {
    let Some(first) = args.first() else {
        return Ok(None);
    };
    match first.as_str() {
        "-h" | "--help" | "help" => Ok(Some(CliCommand::Help)),
        "install" => parse_install(&args[1..]).map(Some),
        "uninstall" => parse_uninstall(&args[1..]).map(Some),
        "add" => parse_add(&args[1..]).map(Some),
        "remove" => parse_remove(&args[1..]).map(Some),
        other => Err(ApplicationError::Usage(format!("unknown command: {other}"))),
    }
}

fn parse_install(tokens: &[String]) -> Result<CliCommand, ApplicationError> {
    match parse_pack_target(tokens, true)? {
        None => Ok(CliCommand::Help),
        Some(target) => Ok(CliCommand::Install(InstallOptions {
            pack: target.pack,
            dest: target.dest,
            force: target.force,
        })),
    }
}

fn parse_uninstall(tokens: &[String]) -> Result<CliCommand, ApplicationError> {
    match parse_pack_target(tokens, false)? {
        None => Ok(CliCommand::Help),
        Some(target) => Ok(CliCommand::Uninstall(UninstallOptions {
            pack: target.pack,
            dest: target.dest,
        })),
    }
}

/// Positional pack name plus the shared `--dest` option of `install` and
/// `uninstall` (plus `--force` for install only).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PackTargetOptions {
    pack: String,
    dest: Option<PathBuf>,
    force: bool,
}

/// `None` = help requested.
fn parse_pack_target(
    tokens: &[String],
    allow_force: bool,
) -> Result<Option<PackTargetOptions>, ApplicationError> {
    let mut target = PackTargetOptions::default();
    let mut parser = TokenParser { tokens, pos: 0 };
    while let Some(token) = parser.peek().map(str::to_string) {
        let flag = token.split('=').next().unwrap_or("");
        match flag {
            "-h" | "--help" => return Ok(None),
            "--dest" => target.dest = Some(PathBuf::from(parser.value(flag)?)),
            "--force" if allow_force => {
                parser.advance();
                target.force = true;
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

fn parse_add(tokens: &[String]) -> Result<CliCommand, ApplicationError> {
    let mut options = AddOptions::default();
    let mut parser = TokenParser { tokens, pos: 0 };
    while let Some(token) = parser.peek().map(str::to_string) {
        let flag = token.split('=').next().unwrap_or("");
        match flag {
            "-h" | "--help" => return Ok(CliCommand::Help),
            "--name" => options.name = Some(parser.value(flag)?),
            "--force" => {
                parser.advance();
                options.force = true;
            }
            _ if is_flag(&token) => {
                return Err(ApplicationError::Usage(format!("unknown option: {token}")))
            }
            _ => {
                if !options.dir.as_os_str().is_empty() {
                    return Err(ApplicationError::Usage(format!(
                        "unexpected argument: {token}"
                    )));
                }
                parser.advance();
                options.dir = PathBuf::from(token);
            }
        }
    }
    if options.dir.as_os_str().is_empty() {
        return Err(ApplicationError::Usage(
            "missing directory: rapm add <dir>".to_string(),
        ));
    }
    Ok(CliCommand::Add(options))
}

fn parse_remove(tokens: &[String]) -> Result<CliCommand, ApplicationError> {
    let mut parser = TokenParser { tokens, pos: 0 };
    match parser.peek() {
        None => Err(ApplicationError::Usage(
            "missing pack name: rapm remove <name>".to_string(),
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
            Ok(CliCommand::Remove { pack })
        }
    }
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
            args(&["install", "--help"]),
            args(&["uninstall", "-h"]),
            args(&["add", "--help"]),
            args(&["remove", "--help"]),
        ] {
            assert_eq!(
                parse(&invocation).unwrap(),
                Some(CliCommand::Help),
                "invocation: {invocation:?}"
            );
        }
    }

    #[test]
    fn removed_commands_are_usage_errors() {
        for invocation in [
            args(&["list"]),
            args(&["show", "tdd"]),
            args(&["pack", "create", "--name", "x"]),
            args(&["frobnicate"]),
        ] {
            assert!(
                matches!(parse(&invocation).unwrap_err(), ApplicationError::Usage(_)),
                "invocation: {invocation:?}"
            );
        }
    }

    #[test]
    fn install_parses_pack_dest_and_force() {
        let command = parse(&args(&["install", "tdd", "--dest", "/tmp/repo", "--force"])).unwrap();
        assert_eq!(
            command,
            Some(CliCommand::Install(InstallOptions {
                pack: "tdd".to_string(),
                dest: Some(PathBuf::from("/tmp/repo")),
                force: true,
            }))
        );
    }

    #[test]
    fn install_defaults_dest_and_force() {
        assert_eq!(
            parse(&args(&["install", "tdd"])).unwrap(),
            Some(CliCommand::Install(InstallOptions {
                pack: "tdd".to_string(),
                dest: None,
                force: false,
            }))
        );
    }

    #[test]
    fn install_rejects_removed_flags_and_extras() {
        for extra in [
            args(&["install", "tdd", "--cli", "vibe"]),
            args(&["install", "tdd", "--dry-run"]),
            args(&["install", "tdd", "other"]),
            args(&["install"]),
        ] {
            assert!(
                matches!(parse(&extra).unwrap_err(), ApplicationError::Usage(_)),
                "invocation: {extra:?}"
            );
        }
    }

    #[test]
    fn uninstall_parses_pack_and_dest_and_rejects_force() {
        assert_eq!(
            parse(&args(&["uninstall", "tdd", "--dest=/tmp/repo"])).unwrap(),
            Some(CliCommand::Uninstall(UninstallOptions {
                pack: "tdd".to_string(),
                dest: Some(PathBuf::from("/tmp/repo")),
            }))
        );
        assert!(matches!(
            parse(&args(&["uninstall", "tdd", "--force"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
        assert!(matches!(
            parse(&args(&["uninstall"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
    }

    #[test]
    fn add_parses_dir_name_and_force() {
        assert_eq!(
            parse(&args(&["add", "./my-pack", "--name", "custom", "--force",])).unwrap(),
            Some(CliCommand::Add(AddOptions {
                dir: PathBuf::from("./my-pack"),
                name: Some("custom".to_string()),
                force: true,
            }))
        );
    }

    #[test]
    fn add_requires_a_directory_and_rejects_extras() {
        assert!(matches!(
            parse(&args(&["add"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
        assert!(matches!(
            parse(&args(&["add", "./a", "./b"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
        assert!(matches!(
            parse(&args(&["add", "./a", "--skill", "./x"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
        // `--name` without a value is a usage error.
        assert!(matches!(
            parse(&args(&["add", "./a", "--name"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
    }

    #[test]
    fn add_defaults_name_and_force() {
        assert_eq!(
            parse(&args(&["add", "./my-pack"])).unwrap(),
            Some(CliCommand::Add(AddOptions {
                dir: PathBuf::from("./my-pack"),
                name: None,
                force: false,
            }))
        );
    }

    #[test]
    fn remove_takes_one_pack_name() {
        assert_eq!(
            parse(&args(&["remove", "my-pack"])).unwrap(),
            Some(CliCommand::Remove {
                pack: "my-pack".to_string()
            })
        );
        assert!(matches!(
            parse(&args(&["remove"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
        assert!(matches!(
            parse(&args(&["remove", "a", "b"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
        assert!(matches!(
            parse(&args(&["remove", "--force", "a"])).unwrap_err(),
            ApplicationError::Usage(_)
        ));
    }
}
