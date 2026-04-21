mod bsn_ast;
mod bsn_parser;
mod formatter;
mod scanner;

use crate::bsn_ast::{BsnListRoot, BsnRoot};
use crate::formatter::{format_bsn_list_root, format_bsn_root, FormatConfig};
use crate::scanner::scan_macros;
use clap::Parser;
use std::path::PathBuf;
use std::process;
use walkdir::WalkDir;

// ─────────────────────────────────────────────────────────
// Core formatting functions (previously in lib.rs)
// ─────────────────────────────────────────────────────────

/// Format a .bsn file (entire content is BSN syntax)
fn format_bsn_file(source: &str, config: &FormatConfig) -> String {
    match syn::parse_str::<BsnRoot>(source) {
        Ok(root) => format_bsn_root(&root, config),
        Err(e) => {
            eprintln!("Parse error in .bsn file: {e}");
            source.to_string()
        }
    }
}

/// Format bsn!/bsn_list! macros inside a .rs file, preserving everything else.
fn format_rs_source(source: &str, config: &FormatConfig) -> String {
    let spans = scan_macros(source);
    if spans.is_empty() {
        return source.to_string();
    }

    let mut output = String::with_capacity(source.len());
    let mut last_end = 0;

    for span in &spans {
        let inner = span.inner_range();
        output.push_str(&source[last_end..inner.start]);

        let inner_source = &source[inner.clone()];
        let formatted = if span.is_bsn {
            format_bsn_macro_inner(inner_source, config)
        } else {
            format_bsn_list_macro_inner(inner_source, config)
        };

        let base_indent = detect_macro_indent(source, span.open);
        let reindented = reindent(&formatted, &base_indent, config);
        output.push('\n');
        output.push_str(&reindented);
        output.push('\n');
        output.push_str(&base_indent);

        last_end = inner.end;
    }

    output.push_str(&source[last_end..]);
    output
}

fn format_bsn_macro_inner(source: &str, config: &FormatConfig) -> String {
    match syn::parse_str::<BsnRoot>(source) {
        Ok(root) => format_bsn_root(&root, config),
        Err(e) => {
            eprintln!("Parse error in bsn! macro: {e}");
            source.to_string()
        }
    }
}

fn format_bsn_list_macro_inner(source: &str, config: &FormatConfig) -> String {
    match syn::parse_str::<BsnListRoot>(source) {
        Ok(root) => format_bsn_list_root(&root, config),
        Err(e) => {
            eprintln!("Parse error in bsn_list! macro: {e}");
            source.to_string()
        }
    }
}

fn detect_macro_indent(source: &str, open_pos: usize) -> String {
    let before = &source[..open_pos];
    let line_start = before.rfind('\n').map_or(0, |p| p + 1);
    let line = &source[line_start..open_pos];
    let indent_len = line.len() - line.trim_start().len();
    line[..indent_len].to_string()
}

fn reindent(formatted: &str, base_indent: &str, config: &FormatConfig) -> String {
    let extra = " ".repeat(config.indent);
    let mut result = String::new();
    for (i, line) in formatted.lines().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        if line.trim().is_empty() {
            continue;
        }
        result.push_str(base_indent);
        result.push_str(&extra);
        result.push_str(line);
    }
    result
}

// ─────────────────────────────────────────────────────────
// CLI
// ─────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "bsn-fmt", about = "Formatter for Bevy Scene Notation (BSN) macros")]
struct Cli {
    #[command(subcommand)]
    subcmd: Option<CargoSubcmd>,

    /// Files or directories to format. Defaults to current directory.
    #[arg(global = true)]
    files: Vec<PathBuf>,

    /// Check mode: report unformatted files without modifying them
    #[arg(long, global = true)]
    check: bool,

    /// Read from stdin (outputs to stdout)
    #[arg(long, global = true)]
    stdin: bool,

    /// Indentation width in spaces
    #[arg(long, default_value = "4", global = true)]
    indent: usize,
}

#[derive(clap::Subcommand)]
enum CargoSubcmd {
    /// Invoked via `cargo bsn-fmt`
    #[command(name = "bsn-fmt")]
    BsnFmt {
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
    },
}

fn main() {
    let cli = Cli::parse();

    let (files, check, stdin, indent) = match cli.subcmd {
        Some(CargoSubcmd::BsnFmt {
            files,
            check,
            stdin,
            indent,
        }) => (files, check, stdin, indent),
        None => (cli.files, cli.check, cli.stdin, cli.indent),
    };

    let config = FormatConfig { indent };

    if stdin {
        let input = std::io::read_to_string(std::io::stdin()).expect("Failed to read stdin");
        let output = format_rs_source(&input, &config);
        print!("{output}");
        return;
    }

    let paths = if files.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        files
    };

    let mut unformatted_count = 0;
    let mut formatted_count = 0;

    for path in &paths {
        if path.is_file() {
            let result = process_file(path, &config, check);
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
                let result = process_file(entry.path(), &config, check);
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

    if check {
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

// ─────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> FormatConfig {
        FormatConfig::default()
    }

    // bsn! macro in .rs source

    #[test]
    fn bsn_simple_type() {
        let input = "bsn! { Transform }";
        let output = format_rs_source(input, &cfg());
        assert!(output.contains("Transform"));
    }

    #[test]
    fn bsn_named_fields() {
        let input = r#"bsn! { Transform { x: 1.0, y: 2.0 } }"#;
        let output = format_rs_source(input, &cfg());
        assert!(output.contains("x: 1.0"));
        assert!(output.contains("y: 2.0"));
    }

    #[test]
    fn bsn_tuple_field() {
        let input = r#"bsn! { Val::Px(100.0) }"#;
        let output = format_rs_source(input, &cfg());
        assert!(output.contains("Val::Px(100.0)"));
    }

    #[test]
    fn bsn_name_hash() {
        let input = r#"bsn! { #my_entity Transform }"#;
        let output = format_rs_source(input, &cfg());
        assert!(output.contains("#my_entity"));
        assert!(output.contains("Transform"));
    }

    #[test]
    fn bsn_expression_braces() {
        let input = r#"bsn! { {some_expr} }"#;
        let output = format_rs_source(input, &cfg());
        assert!(output.contains("some_expr"));
    }

    #[test]
    fn bsn_constructor_call() {
        let input = r#"bsn! { Color::srgb(1.0, 0.0, 0.0) }"#;
        let output = format_rs_source(input, &cfg());
        assert!(output.contains("Color::srgb(1.0, 0.0, 0.0)"));
    }

    #[test]
    fn bsn_template_patch_at() {
        let input = r#"bsn! { @Transform { x: 1.0 } }"#;
        let output = format_rs_source(input, &cfg());
        assert!(output.contains("@Transform { x: 1.0 }"));
    }

    #[test]
    fn bsn_inherited_scene_asset() {
        let input = r#"bsn! { :"scene://base.bsn" }"#;
        let output = format_rs_source(input, &cfg());
        assert!(output.contains(r#":"scene://base.bsn""#));
    }

    // .bsn file formatting — exact output assertions

    #[test]
    fn bsn_file_sprite_named_fields() {
        let output = format_bsn_file(
            r#"Sprite { size: 1, handle: "hello" }"#,
            &cfg(),
        );
        assert_eq!(
            output.trim(),
            "Sprite {\n    size: 1,\n    handle: \"hello\",\n}"
        );
    }

    #[test]
    fn bsn_file_enum_tuple() {
        let output = format_bsn_file("Team::Green(10)", &cfg());
        assert_eq!(output.trim(), "Team::Green(10)");
    }

    #[test]
    fn bsn_file_enum_unit_variant() {
        let output = format_bsn_file("Team::Blue", &cfg());
        assert_eq!(output.trim(), "Team::Blue");
    }

    #[test]
    fn bsn_file_enum_named_fields() {
        let output = format_bsn_file("Team::Red { x: 10 }", &cfg());
        assert_eq!(output.trim(), "Team::Red { x: 10 }");
    }

    #[test]
    fn bsn_file_enum_multi_named_fields() {
        let output = format_bsn_file("Team::Red { x: 10, y: 20 }", &cfg());
        assert_eq!(output.trim(), "Team::Red {\n    x: 10,\n    y: 20,\n}");
    }

    #[test]
    fn bsn_file_constructor() {
        let output = format_bsn_file("Transform::from_translation(Vec3::new(1.0, 2.0, 3.0))", &cfg());
        assert_eq!(
            output.trim(),
            "Transform::from_translation(Vec3::new(1.0, 2.0, 3.0))"
        );
    }

    #[test]
    fn bsn_file_type_const() {
        let output = format_bsn_file("Transform::IDENTITY", &cfg());
        assert_eq!(output.trim(), "Transform::IDENTITY");
    }

    #[test]
    fn bsn_file_template_patch() {
        let output = format_bsn_file("@Transform { x: 1.0 }", &cfg());
        assert_eq!(output.trim(), "@Transform { x: 1.0 }");
    }

    #[test]
    fn bsn_file_template_constructor() {
        let output = format_bsn_file("@Transform::from_xyz(1.0, 2.0, 3.0)", &cfg());
        assert_eq!(output.trim(), "@Transform::from_xyz(1.0, 2.0, 3.0)");
    }

    #[test]
    fn bsn_file_scene_inherit_asset() {
        let output = format_bsn_file(r#":"scene://base.bsn""#, &cfg());
        assert_eq!(output.trim(), r#":"scene://base.bsn""#);
    }

    #[test]
    fn bsn_file_scene_inherit_fn() {
        let output = format_bsn_file(r#":button("Hello")"#, &cfg());
        assert_eq!(output.trim(), r#":button("Hello")"#);
    }

    #[test]
    fn bsn_file_scene_inherit_fn_with_args() {
        let output = format_bsn_file(r#":my_scene(10, "hello")"#, &cfg());
        assert_eq!(output.trim(), r#":my_scene(10, "hello")"#);
    }

    #[test]
    fn bsn_file_inherit_plus_component() {
        let output = format_bsn_file(
            r#":"scene://base.bsn" Transform::IDENTITY"#,
            &cfg(),
        );
        let trimmed = output.trim();
        assert!(trimmed.contains(r#":"scene://base.bsn""#));
        assert!(trimmed.contains("Transform::IDENTITY"));
    }

    #[test]
    fn bsn_file_name_and_component() {
        let output = format_bsn_file("#my_entity Transform", &cfg());
        let trimmed = output.trim();
        assert!(trimmed.contains("#my_entity"));
        assert!(trimmed.contains("Transform"));
    }

    #[test]
    fn bsn_file_children_list() {
        let output = format_bsn_file(
            "Node {} Children [ (Text::new(\"Hello\")), ]",
            &cfg(),
        );
        let trimmed = output.trim();
        assert!(trimmed.contains("Children ["));
        assert!(trimmed.contains("Text::new(\"Hello\")"));
    }

    #[test]
    fn bsn_file_children_multi_entry() {
        let output = format_bsn_file(
            "Node {} Children [ (Sprite), (Text::new(\"Hi\")), ]",
            &cfg(),
        );
        let trimmed = output.trim();
        assert!(trimmed.contains("Sprite"));
        assert!(trimmed.contains("Text::new(\"Hi\")"));
    }

    #[test]
    fn bsn_file_expression_fn_call() {
        let output = format_bsn_file("{some_fn()}", &cfg());
        assert!(output.contains("some_fn"));
    }

    #[test]
    fn bsn_file_generics() {
        let output = format_bsn_file("Gen::<usize> { value: 10 }", &cfg());
        let trimmed = output.trim();
        assert!(trimmed.contains("Gen::"));
        assert!(trimmed.contains("usize"));
        assert!(trimmed.contains("value: 10"));
    }

    #[test]
    fn bsn_file_multi_tuple_fields() {
        let output = format_bsn_file("Foo(100, 200)", &cfg());
        assert_eq!(output.trim(), "Foo(100, 200)");
    }

    #[test]
    fn bsn_file_nested_struct_inline() {
        let output = format_bsn_file(
            "Outer { inner: Inner { x: 1 } }",
            &cfg(),
        );
        let trimmed = output.trim();
        assert!(trimmed.contains("Outer"));
        assert!(trimmed.contains("inner:"));
    }

    #[test]
    fn bsn_file_complex_scene() {
        let output = format_bsn_file(
            r#"#root :"scene://base.bsn" Node { width: Val::Px(100.0) } @Transform { x: 1.0 } Children [ (Text::new("Hello")), ]"#,
            &cfg(),
        );
        let trimmed = output.trim();
        assert!(trimmed.contains("#root"));
        assert!(trimmed.contains(r#":"scene://base.bsn""#));
        assert!(trimmed.contains("Node {"));
        assert!(trimmed.contains("@Transform {"));
        assert!(trimmed.contains("Children ["));
    }

    #[test]
    fn bsn_file_bevy_ui_pattern() {
        let output = format_bsn_file(
            r#"Node { width: Val::Px(100.0), height: Val::Px(50.0) } BackgroundColor(Color::srgb(1.0, 0.0, 0.0)) Children [ (Text::new("Hello")), ]"#,
            &cfg(),
        );
        let trimmed = output.trim();
        assert!(trimmed.contains("Node {"));
        assert!(trimmed.contains("width: Val::Px(100.0)"));
        assert!(trimmed.contains("BackgroundColor(Color::srgb(1.0, 0.0, 0.0))"));
        assert!(trimmed.contains("Children ["));
        assert!(trimmed.contains("Text::new(\"Hello\")"));
    }

    // bsn_list! macro

    #[test]
    fn bsn_list_formatted() {
        let input = "bsn_list! [ (Sprite), (Transform) ]";
        let output = format_rs_source(input, &cfg());
        assert!(output.contains("Sprite"));
        assert!(output.contains("Transform"));
    }

    #[test]
    fn bsn_list_with_expression() {
        let input = "bsn_list! [ (Sprite), ({some_expr}), ]";
        let output = format_rs_source(input, &cfg());
        assert!(output.contains("Sprite"));
        assert!(output.contains("some_expr"));
    }

    // .rs source with macros

    #[test]
    fn rs_source_single_bsn_let() {
        let input = r#"fn setup() { let e = bsn! { Sprite { size: 1, handle: "hello" } }; }"#;
        let output = format_rs_source(input, &cfg());
        assert!(output.contains("fn setup()"));
        assert!(output.contains("Sprite {"));
        assert!(output.contains("size: 1"));
    }

    #[test]
    fn rs_source_multiple_macros() {
        let input = r#"let a = bsn! { Sprite }; let b = bsn! { Transform { x: 1.0 } };"#;
        let output = format_rs_source(input, &cfg());
        assert!(output.contains("Sprite"));
        assert!(output.contains("Transform"));
        assert!(output.contains("x: 1.0"));
    }

    #[test]
    fn no_bsn_macros_unchanged() {
        let input = "fn main() { println!(\"hello\"); }";
        let output = format_rs_source(input, &cfg());
        assert_eq!(output, input);
    }

    #[test]
    fn parse_failure_preserves_original() {
        let input = "bsn! { @@@ invalid @@@ }";
        let output = format_rs_source(input, &cfg());
        assert!(output.contains("bsn!"));
    }

    #[test]
    fn idempotent() {
        let input = r#"bsn! { Sprite { size: 1, handle: "hello" } }"#;
        let first = format_rs_source(input, &cfg());
        let second = format_rs_source(&first, &cfg());
        assert_eq!(first, second);
    }

    #[test]
    fn custom_indent_2() {
        let config = FormatConfig { indent: 2 };
        let output = format_rs_source(
            r#"bsn! { Sprite { size: 1, handle: "hello" } }"#,
            &config,
        );
        assert!(output.contains("  Sprite"));
        assert!(output.contains("    size: 1"));
    }

    #[test]
    fn bsn_file_idempotent() {
        let input = "Node { width: Val::Px(100.0), height: Val::Px(50.0) }";
        let first = format_bsn_file(input, &cfg());
        let second = format_bsn_file(&first, &cfg());
        assert_eq!(first, second);
    }
}
