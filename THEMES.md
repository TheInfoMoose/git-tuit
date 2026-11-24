# Themes

Classic theme on a light terminal:
![](assets/light-theme.png)

With no `theme.ron`, gitui uses the amber theme on terminals that set `COLORTERM=truecolor` or `COLORTERM=24bit`, and the classic gitui colors otherwise. On a light-background truecolor terminal, put `(base: Some("classic"))` in `theme.ron` to use the classic colors.

## Configuration

To change colors, add a `theme.ron` file with the fields you want to override. Unset fields are filled from the classic gitui colors unless the file sets `base: Some("amber")`. Base names ignore case, and `base: "amber"` also works without `Some`. A file that sets `base` is never rewritten, and it may write every color without `Some`. Old full-format theme files are automatically converted to the patch format on first load, with omitted fields taking their classic values. If `theme.ron` exists but cannot be read or parsed, gitui falls back to the classic theme and leaves the file unchanged.

## Default Theme

The default theme uses an amber-on-black RGB palette:

| Field | Default |
| --- | --- |
| `selected_tab` | `#FFD866` |
| `command_fg` | `#FFB000` |
| `selection_bg` | `#2A1A00` |
| `selection_fg` | `#FFD866` |
| `use_selection_fg` | `true` |
| `cmdbar_bg` | `#0C0800` |
| `disabled_fg` | `#996300` |
| `diff_line_add` | `#7FD962` |
| `diff_line_delete` | `#FF5500` |
| `diff_file_added` | `#7FD962` |
| `diff_file_removed` | `#FF5500` |
| `diff_file_moved` | `#FFD866` |
| `diff_file_modified` | `#FFB000` |
| `commit_hash` | `#FFD866` |
| `commit_time` | `#996300` |
| `commit_author` | `#FFB000` |
| `danger_fg` | `#FF5500` |
| `push_gauge_bg` | `#5C3A00` |
| `push_gauge_fg` | `#FFD866` |
| `tag_fg` | `#FFD866` |
| `branch_fg` | `#FFB000` |
| `line_break` | `"¶"` |
| `block_title_focused` | `#FFD866` |
| `border_focused_fg` | `#FFB000` |
| `scroll_track_fg` | `#5C3A00` |
| `scroll_thumb_fg` | `#FFD866` |
| `panel_bg` | `#0C0800` |
| `attention_fg` | `#FF5500` |
| `canvas_bg` | `#000000` |
| `syntax` | `"base16-eighties.dark"` |

The file uses the [Ron format](https://github.com/ron-rs/ron) and is located at one of the following paths, depending on your operating system:

* `$HOME/.config/gitui/theme.ron` (mac)
* `$XDG_CONFIG_HOME/gitui/theme.ron` (linux using XDG)
* `$HOME/.config/gitui/theme.ron` (linux)
* `%APPDATA%/gitui/theme.ron` (Windows)

Alternatively, you can create a theme in the same directory mentioned above and use it with the `-t` flag followed by the name of the file in the directory. E.g. If you are on linux calling `gitui -t arc.ron`, this will load the theme in `$XDG_CONFIG_HOME/gitui/arc.ron` or `$HOME/.config/gitui/arc.ron`.

Example theme override (fills unset fields from classic colors):

```ron
(
    base: Some("classic"),
    selection_bg: Some("Blue"),
    selection_fg: Some("#ffffff"),
    scroll_thumb_fg: Some("Cyan"),
)
```

To override only a few colors on top of the amber default theme, set `base: Some("amber")`:

```ron
(
    base: Some("amber"),
    selection_bg: Some("Black"),
)
```

Note that you need to wrap values in `Some` due to the way the overrides work (as of 0.23); `base` also takes a bare string, and a file that sets `base` may write every color without `Some`.

Notes:

* rgb colors might not be supported in every terminal.
* using a color like `yellow` might appear in whatever your terminal/theme defines for `yellow`
* valid colors can be found in ratatui's [Color](https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html) struct.
* all customizable theme elements can be found in [`style.rs` in the `impl Default for Theme` block](https://github.com/gitui-org/gitui/blob/master/src/ui/style.rs#L305)

## Preset Themes

You can find preset themes by Catppuccin [here](https://github.com/catppuccin/gitui.git).

## Syntax Highlighting

The syntax highlighting theme can be defined using the element `syntax`. Both [default themes of the syntect library](https://github.com/trishume/syntect/blob/7fe13c0fd53cdfa0f9fea1aa14c5ba37f81d8b71/src/dumps.rs#L215) and custom themes are supported.

Example syntax theme:
```ron
(
    syntax: Some("InspiredGitHub"),
)
```

Custom themes are located in the [configuration directory](#configuration), are using TextMate's theme format and must have a `.tmTheme` file extension. To load a custom theme, `syntax` must be set to the file name without the file extension. For example, to load [`Blackboard.tmTheme`](https://raw.githubusercontent.com/filmgirl/TextMate-Themes/refs/heads/master/Blackboard.tmTheme), place the file next to `theme.ron` and set:
```ron
(
    syntax: Some("Blackboard"),
)
```

[filmgirl/TextMate-Themes](https://github.com/filmgirl/TextMate-Themes) offers many [beautiful](https://inkdeep.github.io/TextMate-Themes) TextMate themes to choose from.

## Customizing line breaks

If you want to change how the line break is displayed in the diff, you can also specify `line_break` in your `theme.ron`:

```ron
(
    line_break: Some("¶"),
)
```

Note that if you want to turn it off, you should use a blank string:

```ron
(
    line_break: Some(""),
)
```
## Customizing selection

By default the `selection_fg` color is used to color the text of the selected line.
Diff line, filename, commit hashes, time and author are re-colored with `selection_fg` color.
This can be changed by specifying the `use_selection_fg` boolean in your `theme.ron`:

```
(
    use_selection_fg: Some(false),
)
```

By default, `use_selection_fg` is set to `true`.
