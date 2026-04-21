pub mod bsn_ast;
pub mod bsn_parser; // registers Parse impls
pub mod formatter;
pub mod scanner;

use crate::bsn_ast::{BsnListRoot, BsnRoot};
use crate::formatter::{format_bsn_list_root, format_bsn_root, FormatConfig};
use crate::scanner::scan_macros;

/// Format a .bsn file (entire content is BSN syntax)
pub fn format_bsn_file(source: &str, config: &FormatConfig) -> String {
    match syn::parse_str::<BsnRoot>(source) {
        Ok(root) => format_bsn_root(&root, config),
        Err(e) => {
            eprintln!("Parse error in .bsn file: {e}");
            source.to_string()
        }
    }
}

/// Format bsn!/bsn_list! macros inside a .rs file, preserving everything else.
pub fn format_rs_source(source: &str, config: &FormatConfig) -> String {
    let spans = scan_macros(source);
    if spans.is_empty() {
        return source.to_string();
    }

    let mut output = String::with_capacity(source.len());
    let mut last_end = 0;

    for span in &spans {
        let inner = span.inner_range();
        // Copy everything before this macro's inner content
        output.push_str(&source[last_end..inner.start]);

        let inner_source = &source[inner.clone()];
        let formatted = if span.is_bsn {
            format_bsn_macro_inner(inner_source, config)
        } else {
            format_bsn_list_macro_inner(inner_source, config)
        };

        // Detect the base indentation of the macro line
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

/// Detect the indentation of the line containing the macro opening delimiter.
fn detect_macro_indent(source: &str, open_pos: usize) -> String {
    let before = &source[..open_pos];
    let line_start = before.rfind('\n').map_or(0, |p| p + 1);
    let line = &source[line_start..open_pos];
    let indent_len = line.len() - line.trim_start().len();
    line[..indent_len].to_string()
}

/// Re-indent formatted BSN output relative to the macro's base indentation.
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
