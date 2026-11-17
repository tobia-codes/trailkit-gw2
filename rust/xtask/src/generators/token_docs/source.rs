//! Reads the design token files in `design/tokens/` (ROADMAP R118, Q458).
//!
//! Every file has the sections `[shared]` (addons and web), `[addon]` and
//! later `[web]`. Below a section, a table is a token group: its key is the
//! Rust field of `Theme` (`surface`, `menu`, …), `title` names it in the
//! reference, and every other key is either a token (an inline table with a
//! `name`) or a nested group (`chip.map_point`). A group may be continued in a
//! later file (`border` has its colors in `colors.toml`, its width in
//! `border.toml`); its tokens are appended in file order.

use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use toml_edit::{DocumentMut, InlineTable, Item, Table, Value};

/// The token files in the order their groups appear in the reference.
pub(super) const TOKEN_FILES: [&str; 7] = [
    "colors.toml",
    "border.toml",
    "radius.toml",
    "spacing.toml",
    "components.toml",
    "shadow.toml",
    "typography.toml",
];

/// A color as written in a token file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum ColorValue {
    /// `"#RRGGBB"`: opaque.
    Opaque(u32),
    /// `"#RRGGBBAA"`: with an alpha byte.
    WithAlphaByte(u32),
    /// `{ hex = "#RRGGBB", alpha = 0.5 }`: with an alpha factor.
    WithAlpha(u32, f64),
}

/// The unit of a number token, for the reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Unit {
    /// Pixels (`4 px`).
    Pixels,
    /// A factor of the current font size (`4 × font size`).
    FontSize,
    /// A factor of the Nexus body size (`0.85 × body size`).
    BodySize,
}

/// What a token holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum TokenValue {
    /// A color for each built-in theme.
    Colors {
        /// The Dark theme's color.
        dark: ColorValue,
        /// The Light theme's color.
        light: ColorValue,
    },
    /// One number for both themes.
    Number {
        /// The value.
        value: f64,
        /// Its unit.
        unit: Unit,
    },
}

/// One token.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Token {
    /// The token file it came from, e.g. `colors.toml`; its values go to
    /// the generated module of the same name (ROADMAP R133).
    pub(super) file: &'static str,
    /// The Rust field it fills, e.g. `danger_fill`.
    pub(super) field: String,
    /// Its documented name, e.g. `danger.fill`.
    pub(super) name: String,
    /// What it is for.
    pub(super) purpose: String,
    /// Its value.
    pub(super) value: TokenValue,
}

/// A token group: a Rust struct of `Theme` or one nested in it.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Group {
    /// The path of Rust fields from `Theme`, e.g. `["chip", "place"]`.
    pub(super) path: Vec<String>,
    /// The section it came from (`shared`, `addon`).
    pub(super) section: String,
    /// Its title in the reference; nested groups have none.
    pub(super) title: Option<String>,
    /// Its tokens in file order.
    pub(super) tokens: Vec<Token>,
}

/// Reads all token files of `directory` into groups, in reference order.
///
/// # Errors
/// Returns an error if a file is missing or malformed, a token lacks a
/// field, or a value cannot be read.
pub(super) fn read_groups(directory: &Path) -> Result<Vec<Group>> {
    let mut groups: Vec<Group> = Vec::new();
    for file in TOKEN_FILES {
        let path = directory.join(file);
        let text =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let document: DocumentMut = text
            .parse()
            .with_context(|| format!("parsing {}", path.display()))?;
        for (section, item) in document.iter() {
            let Some(table) = item.as_table() else {
                bail!("{file}: `{section}` must be a section such as [shared]");
            };
            for (key, item) in table {
                let Some(group_table) = item.as_table() else {
                    bail!("{file}: [{section}.{key}] must be a table");
                };
                collect_group(
                    &mut groups,
                    file,
                    section,
                    vec![key.to_owned()],
                    group_table,
                )
                .with_context(|| format!("{file}: [{section}.{key}]"))?;
            }
        }
    }
    Ok(groups)
}

/// Adds the tokens of `table` at `path`, read from `file`, to `groups`
/// (continuing a group of an earlier file), then its nested groups.
fn collect_group(
    groups: &mut Vec<Group>,
    file: &'static str,
    section: &str,
    path: Vec<String>,
    table: &Table,
) -> Result<()> {
    let title = table.get("title").and_then(Item::as_str).map(str::to_owned);
    let mut tokens = Vec::new();
    let mut nested = Vec::new();
    for (key, item) in table {
        if key == "title" {
            continue;
        }
        if let Some(inline) = item.as_inline_table() {
            tokens.push(read_token(file, key, inline).with_context(|| format!("token `{key}`"))?);
        } else if let Some(child) = item.as_table() {
            nested.push((key.to_owned(), child));
        } else {
            bail!("`{key}` is neither a token nor a nested group");
        }
    }
    match groups.iter_mut().find(|group| group.path == path) {
        Some(group) => {
            group.tokens.extend(tokens);
            if group.title.is_none() {
                group.title = title;
            }
        }
        None => groups.push(Group {
            path: path.clone(),
            section: section.to_owned(),
            title,
            tokens,
        }),
    }
    for (key, child) in nested {
        let mut child_path = path.clone();
        child_path.push(key);
        collect_group(groups, file, section, child_path, child)?;
    }
    Ok(())
}

/// Reads one token from its inline table.
fn read_token(file: &'static str, field: &str, table: &InlineTable) -> Result<Token> {
    let text = |key: &str| -> Result<String> {
        table
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .with_context(|| format!("missing text `{key}`"))
    };
    let value = if let Some(number) = table.get("value") {
        let value = number
            .as_float()
            .or_else(|| number.as_integer().map(|integer| integer as f64))
            .context("`value` must be a number")?;
        let unit = match table.get("unit").and_then(Value::as_str) {
            None => Unit::Pixels,
            Some("font_size") => Unit::FontSize,
            Some("body_size") => Unit::BodySize,
            Some(other) => bail!("unknown unit {other:?}"),
        };
        TokenValue::Number { value, unit }
    } else {
        TokenValue::Colors {
            dark: read_color(table.get("dark").context("missing `dark`")?)?,
            light: read_color(table.get("light").context("missing `light`")?)?,
        }
    };
    Ok(Token {
        file,
        field: field.to_owned(),
        name: text("name")?,
        purpose: text("purpose")?,
        value,
    })
}

/// Reads a color: `"#RRGGBB"`, `"#RRGGBBAA"` or `{ hex, alpha }`.
fn read_color(value: &Value) -> Result<ColorValue> {
    if let Some(text) = value.as_str() {
        return match text.strip_prefix('#').map(str::len) {
            Some(6) => Ok(ColorValue::Opaque(parse_hex(text)?)),
            Some(8) => Ok(ColorValue::WithAlphaByte(parse_hex(text)?)),
            _ => bail!("color {text:?} must be #RRGGBB or #RRGGBBAA"),
        };
    }
    let table = value
        .as_inline_table()
        .context("a color is a string or { hex, alpha }")?;
    let hex = table
        .get("hex")
        .and_then(Value::as_str)
        .context("missing `hex`")?;
    let alpha = table
        .get("alpha")
        .and_then(Value::as_float)
        .context("missing `alpha`")?;
    if hex.len() != 7 {
        bail!("color {hex:?} must be #RRGGBB");
    }
    Ok(ColorValue::WithAlpha(parse_hex(hex)?, alpha))
}

/// The number of a `#…` hex color.
fn parse_hex(text: &str) -> Result<u32> {
    u32::from_str_radix(&text[1..], 16).with_context(|| format!("bad hex color {text:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_every_color_form() {
        let opaque: Value = "\"#0F1318\"".parse().unwrap();
        assert_eq!(read_color(&opaque).unwrap(), ColorValue::Opaque(0x0F1318));
        let byte: Value = "\"#05070AB8\"".parse().unwrap();
        assert_eq!(
            read_color(&byte).unwrap(),
            ColorValue::WithAlphaByte(0x0507_0AB8)
        );
        let factor: Value = "{ hex = \"#FFFFFF\", alpha = 0.06 }".parse().unwrap();
        assert_eq!(
            read_color(&factor).unwrap(),
            ColorValue::WithAlpha(0xFFFFFF, 0.06)
        );
    }

    #[test]
    fn a_group_continues_in_a_later_file() {
        let mut groups = Vec::new();
        for (file, text) in [
            (
                "colors.toml",
                "[shared.border]\ntitle = \"Borders\"\ncolor = { name = \"border\", dark = \"#000000\", light = \"#FFFFFF\", purpose = \"p\" }\n",
            ),
            (
                "border.toml",
                "[shared.border]\nwidth = { name = \"border.width\", value = 1.0, purpose = \"p\" }\n",
            ),
        ] {
            let document: DocumentMut = text.parse().unwrap();
            let table = document["shared"]["border"].as_table().unwrap();
            collect_group(
                &mut groups,
                file,
                "shared",
                vec!["border".to_owned()],
                table,
            )
            .unwrap();
        }
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].title.as_deref(), Some("Borders"));
        assert_eq!(groups[0].tokens.len(), 2);
        assert_eq!(groups[0].tokens[1].file, "border.toml");
    }
}
