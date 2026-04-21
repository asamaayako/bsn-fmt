use bsn_fmt::formatter::FormatConfig;
use bsn_fmt::{format_bsn_file, format_rs_source};
use clap::Parser;
use std::path::PathBuf;
use std::process;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "bsn-fmt", about = "Formatter for Bevy Scene Notation (BSN) macros")]
struct Cli {
    /// Files or directories to format. Defaults to current directory.
    files: Vec<PathBuf>,

    /// Check mode: report unformatted files without modifying them
    #[arg(long)]
    check: bool,

    /// Read from stdin (outputs to stdout)
    #[arg(long)]
    stdin: bool,

    /// Indentation width in spaces
    #[arg(long, default_value = "4")]
    indent: usize,
}

fn main() {
    let cli = Cli::parse();
    let config = FormatConfig { indent: cli.indent };

    if cli.stdin {
        let input = std::io::read_to_string(std::io::stdin()).expect("Failed to read stdin");
        let output = format_rs_source(&input, &config);
        print!("{output}");
        return;
    }

    let paths = if cli.files.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        cli.files.clone()
    };

    let mut unformatted_count = 0;
    let mut formatted_count = 0;

    for path in &paths {
        if path.is_file() {
            let result = process_file(path, &config, cli.check);
            match result {
                FileResult::Formatted => formatted_count += 1,
                FileResult::Unchanged => {}
                FileResult::Unformatted => unformatted_count += 1,
                FileResult::Error(e) => eprintln!("Error processing {}: {e}", path.display()),
            }
        } else if path.is_dir() {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let p = e.path();
                    matches!(p.extension().and_then(|s| s.to_str()), Some("rs" | "bsn"))
                })
            {
                let result = process_file(entry.path(), &config, cli.check);
                match result {
                    FileResult::Formatted => formatted_count += 1,
                    FileResult::Unchanged => {}
                    FileResult::Unformatted => unformatted_count += 1,
                    FileResult::Error(e) => {
                        eprintln!("Error processing {}: {e}", entry.path().display());
                    }
                }
            }
        }
    }

    if cli.check {
        if unformatted_count > 0 {
            eprintln!("{unformatted_count} file(s) need formatting");
            process::exit(1);
        }
    } else if formatted_count > 0 {
        eprintln!("Formatted {formatted_count} file(s)");
    }
}

enum FileResult {
    Formatted,
    Unchanged,
    Unformatted,
    Error(String),
}

fn process_file(path: &std::path::Path, config: &FormatConfig, check: bool) -> FileResult {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return FileResult::Error(e.to_string()),
    };

    let is_bsn = path.extension().and_then(|s| s.to_str()) == Some("bsn");
    let formatted = if is_bsn {
        format_bsn_file(&source, config)
    } else {
        format_rs_source(&source, config)
    };

    if formatted == source {
        return FileResult::Unchanged;
    }

    if check {
        eprintln!("Would format: {}", path.display());
        return FileResult::Unformatted;
    }

    match std::fs::write(path, &formatted) {
        Ok(()) => {
            eprintln!("Formatted: {}", path.display());
            FileResult::Formatted
        }
        Err(e) => FileResult::Error(e.to_string()),
    }
}
