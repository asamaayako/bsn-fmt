use bsn_fmt::formatter::FormatConfig;
use bsn_fmt::{format_bsn_file, format_rs_source};

fn cfg() -> FormatConfig {
    FormatConfig::default()
}

// ─────────────────────────────────────────────────────────
// bsn! macro in .rs source — real Bevy patterns
// ─────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────
// .bsn file formatting — exact output assertions
// ─────────────────────────────────────────────────────────

/// Bevy Sprite with named fields → multi-line
#[test]
fn bsn_file_sprite_named_fields() {
    let output = format_bsn_file(
        r#"Sprite { size: 1, handle: "hello" }"#,
        &cfg(),
    );
    assert_eq!(
        output,
        "Sprite {\n    size: 1,\n    handle: \"hello\",\n}\n"
    );
}

/// Enum variant with tuple field
#[test]
fn bsn_file_enum_tuple() {
    assert_eq!(
        format_bsn_file("Team::Green(10)", &cfg()),
        "Team::Green(10)\n"
    );
}

/// Enum variant with named fields (multi-line)
#[test]
fn bsn_file_enum_named_fields() {
    assert_eq!(
        format_bsn_file("Team::Red { x: 10 }", &cfg()),
        "Team::Red { x: 10 }\n"
    );
}

/// Enum unit variant
#[test]
fn bsn_file_enum_unit_variant() {
    assert_eq!(
        format_bsn_file("Team::Blue", &cfg()),
        "Team::Blue\n"
    );
}

/// Constructor call — Transform::from_translation(...)
#[test]
fn bsn_file_constructor() {
    assert_eq!(
        format_bsn_file(
            "Transform::from_translation(Vec3::new(1.0, 1.0, 1.0))",
            &cfg(),
        ),
        "Transform::from_translation(Vec3::new(1.0, 1.0, 1.0))\n"
    );
}

/// Template constructor with @
#[test]
fn bsn_file_template_constructor() {
    assert_eq!(
        format_bsn_file("@Transform::from_xyz(1.0, 2.0, 3.0)", &cfg()),
        "@Transform::from_xyz(1.0, 2.0, 3.0)\n"
    );
}

/// Constant — Transform::IDENTITY
#[test]
fn bsn_file_type_const() {
    assert_eq!(
        format_bsn_file("Transform::IDENTITY", &cfg()),
        "Transform::IDENTITY\n"
    );
}

/// Entity name + component
#[test]
fn bsn_file_name_and_component() {
    assert_eq!(
        format_bsn_file("#my_entity Transform", &cfg()),
        "#my_entity\nTransform\n"
    );
}

/// Template patch with @
#[test]
fn bsn_file_template_patch() {
    assert_eq!(
        format_bsn_file("@Transform { x: 1.0 }", &cfg()),
        "@Transform { x: 1.0 }\n"
    );
}

/// Scene inheritance — asset path
#[test]
fn bsn_file_scene_inherit_asset() {
    assert_eq!(
        format_bsn_file(r#":"scene://base.bsn""#, &cfg()),
        ":\"scene://base.bsn\"\n"
    );
}

/// Scene inheritance — function
#[test]
fn bsn_file_scene_inherit_fn() {
    assert_eq!(
        format_bsn_file(":x", &cfg()),
        ":x\n"
    );
}

/// Scene inheritance — function with args
#[test]
fn bsn_file_scene_inherit_fn_with_args() {
    assert_eq!(
        format_bsn_file(r#":button("Hello")"#, &cfg()),
        ":button(\"Hello\")\n"
    );
}

/// Expression that looks like a function call → no braces
#[test]
fn bsn_file_expression_fn_call() {
    assert_eq!(
        format_bsn_file("{transform_1337()}", &cfg()),
        "transform_1337()\n"
    );
}

/// Nested struct in field value — single field stays inline
#[test]
fn bsn_file_nested_struct_inline() {
    assert_eq!(
        format_bsn_file("Sprite { nested: Nested { foo: 10 } }", &cfg()),
        "Sprite { nested: Nested { foo: 10 } }\n"
    );
}

/// Multiple tuple fields
#[test]
fn bsn_file_multi_tuple_fields() {
    assert_eq!(
        format_bsn_file("Foo(100, 200)", &cfg()),
        "Foo(100, 200)\n"
    );
}

/// Generics — Gen::<usize>
#[test]
fn bsn_file_generics() {
    let output = format_bsn_file("Gen::<usize> { value: 10 }", &cfg());
    assert!(output.contains("Gen::<usize"));
    assert!(output.contains("value: 10"));
}

/// Children list — real Bevy pattern
#[test]
fn bsn_file_children_list() {
    assert_eq!(
        format_bsn_file("Children [ (Sprite), (Node) ]", &cfg()),
        "Children [\n    Sprite,\n    Node,\n]\n"
    );
}

/// Scene inheritance + component
#[test]
fn bsn_file_inherit_plus_component() {
    assert_eq!(
        format_bsn_file(":button Sprite { size: 1 }", &cfg()),
        ":button\nSprite { size: 1 }\n"
    );
}

/// Enum with multiple named fields → multi-line
#[test]
fn bsn_file_enum_multi_named_fields() {
    assert_eq!(
        format_bsn_file("Team::Red { x: 10, y: Nested { foo: 10 } }", &cfg()),
        "Team::Red {\n    x: 10,\n    y: Nested { foo: 10 },\n}\n"
    );
}

// ─────────────────────────────────────────────────────────
// Complex real-world Bevy scenes
// ─────────────────────────────────────────────────────────

/// Multi-entry scene: name + inherit + components + expression + children
#[test]
fn bsn_file_complex_scene() {
    let input = r#"#TopLevel :"scene://base.bsn" Sprite { size: 1 } Team::Green(10) {transform_1337()}"#;
    assert_eq!(
        format_bsn_file(input, &cfg()),
        "#TopLevel\n:\"scene://base.bsn\"\nSprite { size: 1 }\nTeam::Green(10)\ntransform_1337()\n"
    );
}

/// Node with children containing multi-entry scene items
#[test]
fn bsn_file_children_multi_entry() {
    let input = "Node\nChildren [\n(Sprite { size: 2 } Transform::IDENTITY),\n(Node),\n]";
    let output = format_bsn_file(input, &cfg());
    assert_eq!(
        output,
        "Node\nChildren [\n    (\n        Sprite { size: 2 }\n        Transform::IDENTITY\n    ),\n    Node,\n]\n"
    );
}

/// Bevy UI pattern: Node + BackgroundColor + Children with Text
#[test]
fn bsn_file_bevy_ui_pattern() {
    let input = r#"Node { width: Val::Px(100.0), height: Val::Px(50.0) } BackgroundColor(Color::srgb(1.0, 0.0, 0.0)) Children [ (Text::new("Hello")), ]"#;
    let output = format_bsn_file(input, &cfg());
    assert_eq!(
        output,
        concat!(
            "Node {\n",
            "    width: Val::Px(100.0),\n",
            "    height: Val::Px(50.0),\n",
            "}\n",
            "BackgroundColor(Color::srgb(1.0, 0.0, 0.0))\n",
            "Children [\n",
            "    Text::new(\"Hello\"),\n",
            "]\n",
        )
    );
}

// ─────────────────────────────────────────────────────────
// bsn_list! macro — real patterns
// ─────────────────────────────────────────────────────────

/// bsn_list! with single-entry and multi-entry items
#[test]
fn bsn_list_formatted() {
    let input = r#"bsn_list! { (Sprite { size: 1 }), (Node) }"#;
    assert_eq!(
        format_rs_source(input, &cfg()),
        "bsn_list! {\n    Sprite { size: 1 },\n    Node,\n}"
    );
}

/// bsn_list! with expression item
#[test]
fn bsn_list_with_expression() {
    let input = r#"bsn_list! { (Sprite { size: 1 }), {some_vec}, (Node) }"#;
    assert_eq!(
        format_rs_source(input, &cfg()),
        "bsn_list! {\n    Sprite { size: 1 },\n    {some_vec},\n    Node,\n}"
    );
}

// ─────────────────────────────────────────────────────────
// Full .rs file with multiple macros
// ─────────────────────────────────────────────────────────

/// Multiple bsn!/bsn_list! macros in a single .rs file
#[test]
fn rs_source_multiple_macros() {
    let input = r#"
fn setup() {
    let a = bsn! { Transform };
    let b = bsn! { Sprite { size: 1, handle: "hello" } };
    let c = bsn_list! { (Transform), (Visibility) };
}
"#;
    let output = format_rs_source(input, &cfg());
    assert_eq!(
        output,
        concat!(
            "\nfn setup() {\n",
            "    let a = bsn! {\n",
            "        Transform\n",
            "    };\n",
            "    let b = bsn! {\n",
            "        Sprite {\n",
            "            size: 1,\n",
            "            handle: \"hello\",\n",
            "        }\n",
            "    };\n",
            "    let c = bsn_list! {\n",
            "        Transform,\n",
            "        Visibility,\n",
            "    };\n",
            "}\n",
        )
    );
}

/// Single bsn! in a let binding
#[test]
fn rs_source_single_bsn_let() {
    assert_eq!(
        format_rs_source("let x = bsn! { Sprite { size: 1 } };", &cfg()),
        "let x = bsn! {\n    Sprite { size: 1 }\n};"
    );
}

// ─────────────────────────────────────────────────────────
// Edge cases
// ─────────────────────────────────────────────────────────

/// No bsn macros → source returned unchanged
#[test]
fn no_bsn_macros_unchanged() {
    let input = "fn main() { println!(\"hello\"); }";
    assert_eq!(format_rs_source(input, &cfg()), input);
}

/// Parse failure → original content preserved
#[test]
fn parse_failure_preserves_original() {
    let input = "bsn! { @@@ invalid @@@ }";
    let output = format_rs_source(input, &cfg());
    assert!(output.contains("bsn!"));
}

/// Formatting is idempotent
#[test]
fn idempotent() {
    let input = r#"bsn! { Sprite { size: 1, handle: "hello" } }"#;
    let first = format_rs_source(input, &cfg());
    let second = format_rs_source(&first, &cfg());
    assert_eq!(first, second);
}

/// Custom indent width
#[test]
fn custom_indent_2() {
    let config = FormatConfig { indent: 2 };
    let output = format_rs_source(
        r#"bsn! { Sprite { size: 1, handle: "hello" } }"#,
        &config,
    );
    // With indent=2, the fields should be indented by 2 spaces inside the type
    assert!(output.contains("  Sprite"));
    assert!(output.contains("    size: 1"));
}

/// .bsn file idempotent
#[test]
fn bsn_file_idempotent() {
    let input = "Node { width: Val::Px(100.0), height: Val::Px(50.0) }";
    let first = format_bsn_file(input, &cfg());
    let second = format_bsn_file(&first, &cfg());
    assert_eq!(first, second);
}
