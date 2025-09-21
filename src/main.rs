mod elf;
mod requirements;
mod symbols;
mod version;

use clap::{Parser, ValueEnum};
use colored::Colorize;
use requirements::VersionRequirements;
use std::path::PathBuf;
use symbols::SymbolVersion;

#[derive(Clone, Debug, ValueEnum)]
enum ColorChoice {
    #[value(alias = "yes", alias = "true", alias = "on")]
    Always,
    #[value(alias = "no", alias = "false", alias = "off")]
    Never,
    Auto,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum DemangleChoice {
    Cpp,
    Rust,
    #[value(alias = "no", alias = "off")]
    None,
}

#[derive(Parser)]
struct CliArgs {
    #[arg(
        name = "max_version",
        short = 'm',
        required = true,
        help = "Maximum permitted version(s) (e.g. GLIBC_2.17)"
    )]
    max_version: Vec<String>,

    #[arg(required = true, help = "ELF files to analyze")]
    files: Vec<PathBuf>,

    #[arg(
        long = "color",
        short = 'c',
        value_enum,
        default_value = "auto",
        help = "When to use colors"
    )]
    color: ColorChoice,

    #[arg(
        long = "demangle",
        short = 'd',
        value_enum,
        default_value = "none",
        help = "Print demangled symbol names"
    )]
    demangle: DemangleChoice,
}

const EXIT_PASSED: i32 = 0;
const EXIT_ERROR_CHECKING_FILES: i32 = 1;
const EXIT_BAD_ARGS: i32 = 2;
const EXIT_FILES_FAILED_CHECK: i32 = 3;

enum FileCheckResult {
    Pass,
    Fail(Vec<SymbolVersion>),
    Error(anyhow::Error),
}

struct FileResult {
    file: PathBuf,
    result: FileCheckResult,
}

impl FileResult {
    fn new(file: PathBuf, check_result: anyhow::Result<Vec<SymbolVersion>>) -> Self {
        let result = match check_result {
            Ok(symbols) if symbols.is_empty() => FileCheckResult::Pass,
            Ok(symbols) => FileCheckResult::Fail(symbols),
            Err(e) => FileCheckResult::Error(e),
        };
        Self { file, result }
    }
}

struct CheckResult {
    file_results: Vec<FileResult>,
}

impl CheckResult {
    fn has_errors(&self) -> bool {
        self.file_results
            .iter()
            .any(|r| matches!(r.result, FileCheckResult::Error(_)))
    }

    fn has_failures(&self) -> bool {
        self.file_results
            .iter()
            .any(|r| matches!(r.result, FileCheckResult::Fail(_)))
    }
}

fn check_files(files: &[PathBuf], requirements: &VersionRequirements) -> CheckResult {
    let file_results = files
        .iter()
        .map(|f| {
            let file_result =
                elf::get_dyn_undef_symbols(f).map(|symbols| requirements.check_symbols(&symbols));
            FileResult::new(f.clone(), file_result)
        })
        .collect();
    CheckResult { file_results }
}

fn configure_colors(color_choice: &ColorChoice) {
    match color_choice {
        ColorChoice::Always => {
            colored::control::set_override(true);
        }
        ColorChoice::Never => {
            colored::control::set_override(false);
        }
        ColorChoice::Auto => {
            colored::control::unset_override();
        }
    }
}

fn demangle_symbol_name(symbol: &SymbolVersion, demangle: DemangleChoice) -> String {
    match demangle {
        DemangleChoice::Cpp => symbol
            .try_demangle_cpp_name()
            .unwrap_or_else(|| symbol.name.to_string()),
        DemangleChoice::Rust => symbol
            .try_demangle_rust_name()
            .unwrap_or_else(|| symbol.name.to_string()),
        DemangleChoice::None => symbol.name.to_string(),
    }
}

fn print_results(check_result: &CheckResult, demangle: DemangleChoice) {
    for file_result in &check_result.file_results {
        match &file_result.result {
            FileCheckResult::Pass => {
                println!("{}: {}", file_result.file.display(), "PASS".green().bold())
            }
            FileCheckResult::Fail(failed_symbols) => {
                let mut failed_symbols = failed_symbols.clone();
                failed_symbols.sort();

                println!("{}: {}", file_result.file.display(), "FAIL".red().bold());
                for symbol in failed_symbols {
                    let name = demangle_symbol_name(&symbol, demangle);
                    match &symbol.file {
                        None => println!(
                            "    {}{}{}",
                            name,
                            "@".dimmed(),
                            symbol.version.to_string().red()
                        ),
                        Some(file) => {
                            println!(
                                "    {}{}{} ({})",
                                name,
                                "@".dimmed(),
                                symbol.version.to_string().red(),
                                file.dimmed()
                            )
                        }
                    }
                }
            }
            FileCheckResult::Error(e) => {
                eprintln!("{}: {}", file_result.file.display(), "ERROR".red().bold());
                let error_chain: String = e
                    .chain()
                    .map(|cause| cause.to_string())
                    .collect::<Vec<_>>()
                    .join(": ");
                eprintln!("    {}", error_chain.red());
            }
        }
    }
}

fn get_exit_code(check_result: CheckResult) -> i32 {
    match (check_result.has_errors(), check_result.has_failures()) {
        (true, _) => EXIT_ERROR_CHECKING_FILES,
        (false, true) => EXIT_FILES_FAILED_CHECK,
        (false, false) => EXIT_PASSED,
    }
}

fn main() {
    let args = CliArgs::parse();

    configure_colors(&args.color);

    let requirements = match VersionRequirements::parse(&args.max_version) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(EXIT_BAD_ARGS);
        }
    };

    let check_result = check_files(&args.files, &requirements);

    print_results(&check_result, args.demangle);

    let exit_code = get_exit_code(check_result);
    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn file_result_new_with_empty_symbols_is_pass() {
        let result = FileResult::new(PathBuf::from("test.so"), Ok(vec![]));
        assert!(matches!(result.result, FileCheckResult::Pass));
    }

    #[test]
    fn file_result_new_with_symbols_is_fail() {
        let symbols = vec![SymbolVersion::parse("malloc", "GLIBC_2.14", None).unwrap()];
        let result = FileResult::new(PathBuf::from("test.so"), Ok(symbols.clone()));
        match result.result {
            FileCheckResult::Fail(failed_symbols) => {
                assert_eq!(failed_symbols.len(), 1);
                assert_eq!(failed_symbols[0].name, "malloc");
            }
            _ => panic!("Expected Fail result"),
        }
    }

    #[test]
    fn file_result_new_with_error_is_error() {
        let error = anyhow!("Test error");
        let result = FileResult::new(PathBuf::from("test.so"), Err(error));
        assert!(matches!(result.result, FileCheckResult::Error(_)));
    }

    #[test]
    fn check_result_has_errors_true_when_error_present() {
        let file_results = vec![
            FileResult {
                file: PathBuf::from("good.so"),
                result: FileCheckResult::Pass,
            },
            FileResult {
                file: PathBuf::from("bad.so"),
                result: FileCheckResult::Error(anyhow!("Test error")),
            },
        ];
        let check_result = CheckResult { file_results };
        assert!(check_result.has_errors());
    }

    #[test]
    fn check_result_has_errors_false_when_no_errors() {
        let file_results = vec![
            FileResult {
                file: PathBuf::from("good.so"),
                result: FileCheckResult::Pass,
            },
            FileResult {
                file: PathBuf::from("fail.so"),
                result: FileCheckResult::Fail(vec![
                    SymbolVersion::parse("malloc", "GLIBC_2.14", None).unwrap(),
                ]),
            },
        ];
        let check_result = CheckResult { file_results };
        assert!(!check_result.has_errors());
    }

    #[test]
    fn check_result_has_failures_true_when_failure_present() {
        let file_results = vec![
            FileResult {
                file: PathBuf::from("good.so"),
                result: FileCheckResult::Pass,
            },
            FileResult {
                file: PathBuf::from("fail.so"),
                result: FileCheckResult::Fail(vec![
                    SymbolVersion::parse("malloc", "GLIBC_2.14", None).unwrap(),
                ]),
            },
        ];
        let check_result = CheckResult { file_results };
        assert!(check_result.has_failures());
    }

    #[test]
    fn check_result_has_failures_false_when_no_failures() {
        let file_results = vec![
            FileResult {
                file: PathBuf::from("good.so"),
                result: FileCheckResult::Pass,
            },
            FileResult {
                file: PathBuf::from("error.so"),
                result: FileCheckResult::Error(anyhow!("Test error")),
            },
        ];
        let check_result = CheckResult { file_results };
        assert!(!check_result.has_failures());
    }

    #[test]
    fn get_exit_code_all_pass_returns_success() {
        let file_results = vec![
            FileResult {
                file: PathBuf::from("test1.so"),
                result: FileCheckResult::Pass,
            },
            FileResult {
                file: PathBuf::from("test2.so"),
                result: FileCheckResult::Pass,
            },
        ];
        let check_result = CheckResult { file_results };
        assert_eq!(get_exit_code(check_result), EXIT_PASSED);
    }

    #[test]
    fn get_exit_code_with_failures_returns_failure_code() {
        let file_results = vec![
            FileResult {
                file: PathBuf::from("good.so"),
                result: FileCheckResult::Pass,
            },
            FileResult {
                file: PathBuf::from("bad.so"),
                result: FileCheckResult::Fail(vec![
                    SymbolVersion::parse("malloc", "GLIBC_2.14", None).unwrap(),
                ]),
            },
        ];
        let check_result = CheckResult { file_results };
        assert_eq!(get_exit_code(check_result), EXIT_FILES_FAILED_CHECK);
    }

    #[test]
    fn get_exit_code_with_errors_returns_error_code() {
        let file_results = vec![
            FileResult {
                file: PathBuf::from("good.so"),
                result: FileCheckResult::Pass,
            },
            FileResult {
                file: PathBuf::from("error.so"),
                result: FileCheckResult::Error(anyhow!("Test error")),
            },
        ];
        let check_result = CheckResult { file_results };
        assert_eq!(get_exit_code(check_result), EXIT_ERROR_CHECKING_FILES);
    }

    #[test]
    fn get_exit_code_errors_take_precedence_over_failures() {
        let file_results = vec![
            FileResult {
                file: PathBuf::from("fail.so"),
                result: FileCheckResult::Fail(vec![
                    SymbolVersion::parse("malloc", "GLIBC_2.14", None).unwrap(),
                ]),
            },
            FileResult {
                file: PathBuf::from("error.so"),
                result: FileCheckResult::Error(anyhow!("Test error")),
            },
        ];
        let check_result = CheckResult { file_results };
        assert_eq!(get_exit_code(check_result), EXIT_ERROR_CHECKING_FILES);
    }
}
