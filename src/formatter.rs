use crate::bsn_ast::*;
use proc_macro2::TokenTree;
use quote::ToTokens;
use std::fmt::Write;

pub struct FormatConfig {
    pub indent: usize,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self { indent: 4 }
    }
}

/// Pretty-print a TokenStream without the extra spaces that `TokenStream::to_string()` adds.
/// Produces output like `Val::Px(100.0)` instead of `Val :: Px (100.0)`.
fn tokens_to_compact_string(ts: &proc_macro2::TokenStream) -> String {
    let mut out = String::new();
    let mut prev_needs_space = false;

    for tt in ts.clone() {
        match &tt {
            TokenTree::Punct(p) => {
                let ch = p.as_char();
                // No space before :: , ; . and similar
                if ch == ':' || ch == ',' || ch == ';' || ch == '.' {
                    // No space before
                    out.push(ch);
                    // Space after comma/semicolon, but not after : (part of ::)
                    prev_needs_space = ch == ',' || ch == ';';
                } else {
                    if prev_needs_space {
                        out.push(' ');
                    }
                    out.push(ch);
                    prev_needs_space = false;
                }
            }
            TokenTree::Group(g) => {
                let (open, close) = match g.delimiter() {
                    proc_macro2::Delimiter::Parenthesis => ("(", ")"),
                    proc_macro2::Delimiter::Brace => ("{ ", " }"),
                    proc_macro2::Delimiter::Bracket => ("[", "]"),
                    proc_macro2::Delimiter::None => ("", ""),
                };
                // No space before opening delimiter in most cases
                out.push_str(open);
                out.push_str(&tokens_to_compact_string(&g.stream()));
                out.push_str(close);
                prev_needs_space = true;
            }
            TokenTree::Ident(_) | TokenTree::Literal(_) => {
                if prev_needs_space {
                    out.push(' ');
                }
                write!(out, "{tt}").unwrap();
                prev_needs_space = true;
            }
        }
    }
    out
}

pub fn format_bsn_root(root: &BsnRoot, config: &FormatConfig) -> String {
    let mut out = String::new();
    format_entries(&root.0.entries, &mut out, 0, config);
    trim_trailing_whitespace(&mut out);
    out
}

pub fn format_bsn_list_root(root: &BsnListRoot, config: &FormatConfig) -> String {
    let mut out = String::new();
    format_scene_list_items(&root.0, &mut out, 0, config);
    trim_trailing_whitespace(&mut out);
    out
}

fn indent_str(level: usize, config: &FormatConfig) -> String {
    " ".repeat(level * config.indent)
}

fn format_entries(entries: &[BsnEntry], out: &mut String, level: usize, config: &FormatConfig) {
    for entry in entries {
        format_entry(entry, out, level, config);
        out.push('\n');
    }
}

fn format_entry(entry: &BsnEntry, out: &mut String, level: usize, config: &FormatConfig) {
    let pad = indent_str(level, config);
    match entry {
        BsnEntry::Name(ident) => {
            write!(out, "{pad}#{ident}").unwrap();
        }
        BsnEntry::NameExpression(ts) => {
            write!(out, "{pad}#{{{}}}", tokens_to_compact_string(ts)).unwrap();
        }
        BsnEntry::FromTemplatePatch(bsn_type) => {
            write!(out, "{pad}").unwrap();
            format_bsn_type(bsn_type, out, level, config);
        }
        BsnEntry::TemplatePatch(bsn_type) => {
            write!(out, "{pad}@").unwrap();
            format_bsn_type(bsn_type, out, level, config);
        }
        BsnEntry::FromTemplateConstructor(ctor) => {
            write!(out, "{pad}").unwrap();
            format_constructor(ctor, out);
        }
        BsnEntry::TemplateConstructor(ctor) => {
            write!(out, "{pad}@").unwrap();
            format_constructor(ctor, out);
        }
        BsnEntry::TemplateConst {
            type_path,
            const_ident,
        } => {
            write!(out, "{pad}{}::{const_ident}", path_to_string(type_path)).unwrap();
        }
        BsnEntry::SceneExpression(ts) => {
            let compact = tokens_to_compact_string(ts);
            // If the expression looks like a path/function call (starts with ident),
            // output without braces. Otherwise wrap in {}.
            if looks_like_path_expr(ts) {
                write!(out, "{pad}{compact}").unwrap();
            } else {
                write!(out, "{pad}{{{compact}}}").unwrap();
            }
        }
        BsnEntry::InheritedScene(scene) => {
            write!(out, "{pad}").unwrap();
            format_inherited_scene(scene, out);
        }
        BsnEntry::RelatedSceneList(rsl) => {
            write!(out, "{pad}").unwrap();
            format_related_scene_list(rsl, out, level, config);
        }
    }
}

fn format_bsn_type(bsn_type: &BsnType, out: &mut String, level: usize, config: &FormatConfig) {
    let path_str = path_to_string(&bsn_type.path);
    if let Some(ref variant) = bsn_type.enum_variant {
        write!(out, "{path_str}::{variant}").unwrap();
    } else {
        write!(out, "{path_str}").unwrap();
    }
    format_fields(&bsn_type.fields, out, level, config);
}

fn format_fields(fields: &BsnFields, out: &mut String, level: usize, config: &FormatConfig) {
    match fields {
        BsnFields::Named(named) if named.is_empty() => {}
        BsnFields::Named(named) => {
            // Single short field → inline
            if named.len() == 1 {
                let f = &named[0];
                let inline = format_named_field_inline(f);
                if inline.len() <= 60 {
                    write!(out, " {{ {inline} }}").unwrap();
                    return;
                }
            }
            // Multi-field → multiline
            out.push_str(" {\n");
            for f in named {
                let pad = indent_str(level + 1, config);
                write!(out, "{pad}").unwrap();
                format_named_field(f, out, level + 1, config);
                out.push_str(",\n");
            }
            let pad = indent_str(level, config);
            write!(out, "{pad}}}").unwrap();
        }
        BsnFields::Tuple(unnamed) if unnamed.is_empty() => {}
        BsnFields::Tuple(unnamed) => {
            out.push('(');
            for (i, f) in unnamed.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                format_value(&f.value, out, level, config);
            }
            out.push(')');
        }
    }
}

fn format_named_field_inline(field: &BsnNamedField) -> String {
    let mut s = field.name.to_string();
    if let Some(ref val) = field.value {
        s.push_str(": ");
        let mut val_str = String::new();
        format_value(val, &mut val_str, 0, &FormatConfig::default());
        s.push_str(&val_str);
    }
    s
}

fn format_named_field(
    field: &BsnNamedField,
    out: &mut String,
    level: usize,
    config: &FormatConfig,
) {
    write!(out, "{}", field.name).unwrap();
    if let Some(ref val) = field.value {
        out.push_str(": ");
        format_value(val, out, level, config);
    }
}

fn format_value(val: &BsnValue, out: &mut String, level: usize, config: &FormatConfig) {
    match val {
        BsnValue::Expr(ts) => out.push_str(&tokens_to_compact_string(ts)),
        BsnValue::Closure(ts) => out.push_str(&tokens_to_compact_string(ts)),
        BsnValue::Ident(ident) => write!(out, "{ident}").unwrap(),
        BsnValue::Lit(lit) => out.push_str(&tokens_to_compact_string(&lit.to_token_stream())),
        BsnValue::Type(bsn_type) => format_bsn_type(bsn_type, out, level, config),
        BsnValue::Tuple(tuple) => {
            out.push('(');
            for (i, v) in tuple.0.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                format_value(v, out, level, config);
            }
            out.push(')');
        }
        BsnValue::Name(ident) => write!(out, "#{ident}").unwrap(),
    }
}

fn format_constructor(ctor: &BsnConstructor, out: &mut String) {
    let path_str = path_to_string(&ctor.type_path);
    write!(out, "{path_str}::{}(", ctor.function).unwrap();
    if let Some(ref args) = ctor.args {
        let args_str: Vec<String> = args
            .iter()
            .map(|e| tokens_to_compact_string(&e.to_token_stream()))
            .collect();
        out.push_str(&args_str.join(", "));
    }
    out.push(')');
}

fn format_inherited_scene(scene: &BsnInheritedScene, out: &mut String) {
    match scene {
        BsnInheritedScene::Asset(lit) => {
            write!(out, ":\"{}\"", lit.value()).unwrap();
        }
        BsnInheritedScene::Fn { function, args } => {
            write!(out, ":{}", path_to_string(function)).unwrap();
            if let Some(args) = args {
                out.push('(');
                let args_str: Vec<String> = args
                    .iter()
                    .map(|e| tokens_to_compact_string(&e.to_token_stream()))
                    .collect();
                out.push_str(&args_str.join(", "));
                out.push(')');
            }
        }
    }
}

fn format_related_scene_list(
    rsl: &BsnRelatedSceneList,
    out: &mut String,
    level: usize,
    config: &FormatConfig,
) {
    let path_str = path_to_string(&rsl.relationship_path);
    writeln!(out, "{path_str} [").unwrap();
    format_scene_list_items(&rsl.scene_list.0, out, level + 1, config);
    let pad = indent_str(level, config);
    write!(out, "{pad}]").unwrap();
}

fn format_scene_list_items(
    items: &BsnSceneListItems,
    out: &mut String,
    level: usize,
    config: &FormatConfig,
) {
    for (i, item) in items.0.iter().enumerate() {
        format_scene_list_item(item, out, level, config);
        out.push(',');
        out.push('\n');
        if i < items.0.len() - 1 {
            // No extra blank line between items
        }
    }
}

fn format_scene_list_item(
    item: &BsnSceneListItem,
    out: &mut String,
    level: usize,
    config: &FormatConfig,
) {
    let pad = indent_str(level, config);
    match item {
        BsnSceneListItem::Scene(bsn) => {
            if bsn.entries.len() == 1 {
                // Single entry scene: inline without parens
                format_entry(&bsn.entries[0], out, level, config);
            } else {
                // Multi-entry scene: wrap in parens
                writeln!(out, "{pad}(").unwrap();
                format_entries(&bsn.entries, out, level + 1, config);
                write!(out, "{pad})").unwrap();
            }
        }
        BsnSceneListItem::Expression(stmts) => {
            let tokens: proc_macro2::TokenStream = stmts
                .iter()
                .flat_map(|s| s.to_token_stream())
                .collect();
            write!(out, "{pad}{{{}}}", tokens_to_compact_string(&tokens)).unwrap();
        }
    }
}

/// Check if a token stream looks like a path expression (e.g. `foo()`, `foo::bar()`, `my_var`).
/// These don't need braces in BSN output. Complex expressions like `a + b` do need braces.
fn looks_like_path_expr(ts: &proc_macro2::TokenStream) -> bool {
    let tokens: Vec<TokenTree> = ts.clone().into_iter().collect();
    if tokens.is_empty() {
        return false;
    }
    // Must start with an identifier
    if !matches!(&tokens[0], TokenTree::Ident(_)) {
        return false;
    }
    // Walk through: allow Ident, Punct(::), and a final Group(Paren)
    let mut i = 1;
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Punct(p) if p.as_char() == ':' => {
                i += 1; // skip second ':'
            }
            TokenTree::Ident(_) => {}
            TokenTree::Group(g)
                if g.delimiter() == proc_macro2::Delimiter::Parenthesis
                    && i == tokens.len() - 1 =>
            {
                return true;
            }
            _ => return false,
        }
        i += 1;
    }
    // Bare path without parens is also fine
    true
}

fn path_to_string(path: &syn::Path) -> String {
    tokens_to_compact_string(&path.to_token_stream())
}

fn trim_trailing_whitespace(s: &mut String) {
    while s.ends_with('\n') || s.ends_with(' ') {
        s.pop();
    }
    s.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;

    fn compact(input: &str) -> String {
        let ts: TokenStream = input.parse().unwrap();
        tokens_to_compact_string(&ts)
    }

    #[test]
    fn compact_path_with_colons() {
        assert_eq!(compact("Val :: Px"), "Val::Px");
    }

    #[test]
    fn compact_function_call() {
        assert_eq!(compact("Val :: Px (100.0)"), "Val::Px(100.0)");
    }

    #[test]
    fn compact_comma_spacing() {
        assert_eq!(compact("a , b , c"), "a, b, c");
    }

    #[test]
    fn compact_braces() {
        // Group tokens don't get a space before the opening delimiter
        assert_eq!(compact("Foo { x }"), "Foo{ x }");
    }

    #[test]
    fn compact_nested_path() {
        assert_eq!(compact("std :: collections :: HashMap"), "std::collections::HashMap");
    }

    #[test]
    fn looks_like_path_simple_ident() {
        let ts: TokenStream = "Transform".parse().unwrap();
        assert!(looks_like_path_expr(&ts));
    }

    #[test]
    fn looks_like_path_qualified() {
        let ts: TokenStream = "Val::Px(100.0)".parse().unwrap();
        assert!(looks_like_path_expr(&ts));
    }

    #[test]
    fn looks_like_path_rejects_arithmetic() {
        let ts: TokenStream = "1 + 2".parse().unwrap();
        assert!(!looks_like_path_expr(&ts));
    }

    #[test]
    fn looks_like_path_rejects_leading_literal() {
        let ts: TokenStream = "42".parse().unwrap();
        assert!(!looks_like_path_expr(&ts));
    }
}
