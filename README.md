# bsn-fmt

A formatter for [Bevy Scene Notation (BSN)](https://github.com/bevyengine/bevy) — formats `bsn!` and `bsn_list!` macros in `.rs` files, and standalone `.bsn` files.

## Install

```bash
cargo install --git https://github.com/asamaayako/bsn-fmt
```

Or build from source:

```bash
git clone https://github.com/asamaayako/bsn-fmt
cd bsn-fmt
cargo install --path .
```

## Usage

```bash
# Format files in current directory (recursively)
bsn-fmt

# Format specific files or directories
bsn-fmt src/main.rs src/scenes/

# Check mode — report unformatted files without modifying
bsn-fmt --check

# Read from stdin, write to stdout
echo 'bsn! { Sprite { size: 1, handle: "hello" } }' | bsn-fmt --stdin

# Custom indent width (default: 4)
bsn-fmt --indent 2
```

## What it does

Before:
```rust
let entity = bsn! { Node { width: Val::Px(100.0), height: Val::Px(50.0) } BackgroundColor(Color::srgb(1.0, 0.0, 0.0)) Children [ (Text::new("Hello")), ] };
```

After:
```rust
let entity = bsn! {
    Node {
        width: Val::Px(100.0),
        height: Val::Px(50.0),
    }
    BackgroundColor(Color::srgb(1.0, 0.0, 0.0))
    Children [
        Text::new("Hello"),
    ]
};
```

Supported BSN syntax:

- Named fields — `Node { width: Val::Px(100.0) }`
- Tuple fields — `Health(10)`, `Foo(100, 200)`
- Enum variants — `Team::Red { x: 10 }`, `Team::Green(10)`, `Team::Blue`
- Constructors — `Transform::from_xyz(1.0, 2.0, 3.0)`
- Constants — `Transform::IDENTITY`
- Template patches — `@Transform { x: 1.0 }`
- Scene inheritance — `:"scene://base.bsn"`, `:button("Hello")`
- Entity names — `#my_entity`
- Expressions — `{some_expr}`
- Children — `Children [ (Sprite), (Node) ]`
- Generics — `Gen::<usize> { value: 10 }`
- `bsn_list!` macro

## License

MIT
