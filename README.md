# bsn-fmt
##all code by ai
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
cargo bsn-fmt

# Format specific files or directories
cargo bsn-fmt src/main.rs src/scenes/

# Check mode — report unformatted files without modifying
cargo bsn-fmt --check

# Read from stdin, write to stdout
echo 'bsn! { Sprite { size: 1 } }' | cargo bsn-fmt --stdin

# Custom indent width (default: 4)
cargo bsn-fmt --indent 2
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

## Compatibility

BSN 语法解析器适配自 Bevy 主分支 (`main`) 的 `bevy_scene_macros/src/bsn/` 模块，对应 BSN 在 Bevy 0.19 开发周期中的语法（[PR #20158](https://github.com/bevyengine/bevy/pull/20158) 及后续合并的系列 PR）。

BSN 目前仍在快速迭代中，尚未包含在任何 Bevy 稳定版本中（最新稳定版为 0.18.1）。当 BSN 语法在上游发生变化时，本工具的解析器可能需要同步更新。

| bsn-fmt 版本 | 对应 Bevy 分支 | 备注 |
|---|---|---|
| 0.1.x | `main` (pre-0.19) | 初始版本，覆盖 BSN 核心语法 |

## License

MIT
