//! Architecture test (E01): the strictly-inward dependency flow is
//! enforced on the real sources — the build fails if a layer boundary is
//! crossed again.
//!
//! Rules:
//! 1. No file under `src/presentation/` (co-located `#[cfg(test)]`
//!    modules included) imports `crate::domain` or `crate::infrastructure`.
//! 2. No file under `src/presentation/tui/` performs direct I/O (`std::fs`);
//!    `src/presentation/library.rs` likewise, except inside its `#[cfg(test)]`
//!    module (TempDir fixtures).
//! 3. No file under `src/application/` imports `crate::infrastructure`.

use std::path::{Path, PathBuf};

fn src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in std::fs::read_dir(&current).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// `use` statements referencing the given crate path, co-located test
/// modules included.
fn use_lines(source: &str, needle: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let trimmed = line.trim_start();
            trimmed.starts_with("use ") && trimmed.contains(needle)
        })
        .map(|(index, _)| index + 1)
        .collect()
}

/// Source with any top-level `#[cfg(test)] mod ... { ... }` block removed
/// (brace-matched, so code after the test module is still scanned). Used
/// for the I/O rule on `library.rs`, whose test fixtures legitimately use
/// TempDir.
fn strip_test_module(source: &str) -> String {
    let Some(attr) = source.find("#[cfg(test)]") else {
        return source.to_string();
    };
    let rest = &source[attr..];
    let Some(mod_kw) = rest.find("mod ") else {
        return source.to_string();
    };
    let after_mod = &rest[mod_kw..];
    let Some(open) = after_mod.find('{') else {
        return source.to_string();
    };
    let mut depth = 0usize;
    let mut end = None;
    for (index, ch) in after_mod[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(open + index + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    match end {
        Some(end) => format!("{}{}", &source[..attr], &after_mod[end..]),
        None => source.to_string(),
    }
}

fn direct_io_lines(source: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains("std::fs"))
        .map(|(index, _)| index + 1)
        .collect()
}

#[test]
fn presentation_never_imports_domain_or_infrastructure() {
    let presentation = src_root().join("presentation");
    let mut violations = Vec::new();
    for file in rust_files(&presentation) {
        let source = std::fs::read_to_string(&file).unwrap();
        for needle in ["crate::domain", "crate::infrastructure"] {
            for line in use_lines(&source, needle) {
                violations.push(format!("{}:{line} imports `{needle}`", file.display()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "presentation layer must not import domain/infrastructure:\n{}",
        violations.join("\n")
    );
}

#[test]
fn presentation_tui_does_no_direct_io() {
    let tui = src_root().join("presentation").join("tui");
    let mut violations = Vec::new();
    for file in rust_files(&tui) {
        let source = std::fs::read_to_string(&file).unwrap();
        for line in direct_io_lines(&source) {
            violations.push(format!("{}:{line} uses std::fs", file.display()));
        }
    }
    assert!(
        violations.is_empty(),
        "presentation TUI must not do direct I/O:\n{}",
        violations.join("\n")
    );
}

#[test]
fn presentation_library_does_no_direct_io_outside_its_tests() {
    let file = src_root().join("presentation").join("library.rs");
    let source = std::fs::read_to_string(&file).unwrap();
    let violations = direct_io_lines(&strip_test_module(&source));
    assert!(
        violations.is_empty(),
        "src/presentation/library.rs must not use std::fs outside #[cfg(test)]: lines {violations:?}"
    );
}

#[test]
fn application_never_imports_infrastructure() {
    let application = src_root().join("application");
    let mut violations = Vec::new();
    for file in rust_files(&application) {
        let source = std::fs::read_to_string(&file).unwrap();
        for line in use_lines(&source, "crate::infrastructure") {
            violations.push(format!(
                "{}:{line} imports `crate::infrastructure`",
                file.display()
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "application layer must not import infrastructure:\n{}",
        violations.join("\n")
    );
}

#[cfg(test)]
mod unit {
    use super::{strip_test_module, use_lines};

    #[test]
    fn use_lines_finds_imports_in_test_modules_too() {
        let source = "use crate::domain::pack::value_objects::CliId;\n#[cfg(test)]\nmod tests {\n    use crate::domain::Pack;\n}\n";
        assert_eq!(use_lines(source, "crate::domain"), vec![1, 4]);
    }

    #[test]
    fn use_lines_ignores_comments_and_non_use_lines() {
        let source = "// use crate::domain::fake;\nlet x = \"use crate::domain\";\n";
        assert!(use_lines(source, "crate::domain").is_empty());
    }

    #[test]
    fn strip_test_module_removes_the_test_block_but_keeps_code_after_it() {
        let source =
            "fn a() {}\n#[cfg(test)]\nmod tests {\n    std::fs::remove_file(\"x\");\n}\nfn after() {}\n";
        assert_eq!(strip_test_module(source), "fn a() {}\n\nfn after() {}\n");
        assert_eq!(strip_test_module("fn a() {}"), "fn a() {}");
    }
}
