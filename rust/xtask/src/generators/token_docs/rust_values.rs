//! Renders the theme values from the token groups (ROADMAP R118, Q455,
//! R133): one module per token file in `trailkit-ui/src/theme/generated/`
//! holds that file's values, and `theme/generated.rs` assembles them into
//! `Theme::DARK` and `Theme::LIGHT`. Only the values are generated; the types
//! stay hand-written, so a token the types do not know (or a missing one)
//! fails to compile.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

use anyhow::{Result, bail};

use super::source::{ColorValue, Group, Token, TokenValue};

/// Groups that are a struct of their own in each theme: `Theme` field and
/// Rust type. Nested groups of `chip` are `ChipCategoryColors`.
const THEME_GROUPS: [(&str, &str); 9] = [
    ("surface", "SurfaceColors"),
    ("text", "TextColors"),
    ("accent", "AccentColors"),
    ("status", "StatusColors"),
    ("border", "BorderTokens"),
    ("shadow", "ShadowTokens"),
    ("chip", "ChipColors"),
    ("coin", "CoinColors"),
    ("event", "EventColors"),
];

/// The type of a group nested in a theme group.
const NESTED_TYPE: &str = "ChipCategoryColors";

/// Groups shared by both themes as one constant: `Theme` field, constant
/// and Rust type.
const SHARED_GROUPS: [(&str, &str, &str); 7] = [
    ("spacing", "SPACING", "Spacing"),
    ("list", "LIST_LAYOUT", "ListLayout"),
    ("pane", "PANE_LAYOUT", "PaneLayout"),
    ("button", "BUTTON_LAYOUT", "ButtonLayout"),
    ("menu", "MENU_LAYOUT", "MenuLayout"),
    ("radius", "RADII", "Radii"),
    ("typography", "TYPOGRAPHY", "Typography"),
];

/// The assembling module, relative to the `theme` module directory.
pub(super) const ASSEMBLY_PATH: &str = "generated.rs";

/// The directory of the per-file modules, relative to the `theme` module
/// directory.
pub(super) const MODULE_DIRECTORY: &str = "generated";

/// Which built-in theme a color is rendered for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Variant {
    /// `Theme::DARK`.
    Dark,
    /// `Theme::LIGHT`.
    Light,
}

impl Variant {
    /// Both themes, in the order they are rendered.
    const BOTH: [Self; 2] = [Self::Dark, Self::Light];

    /// The prefix of its constants, e.g. `DARK`.
    const fn prefix(self) -> &'static str {
        match self {
            Self::Dark => "DARK",
            Self::Light => "LIGHT",
        }
    }

    /// Its name in doc comments, e.g. `dark`.
    const fn name(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    /// Its index into per-theme arrays.
    const fn index(self) -> usize {
        match self {
            Self::Dark => 0,
            Self::Light => 1,
        }
    }
}

/// One generated Rust file, before `rustfmt`.
#[derive(Debug)]
pub(super) struct RustFile {
    /// Its path relative to the `theme` module directory, e.g.
    /// `generated/colors.rs`.
    pub(super) path: String,
    /// Its source.
    pub(super) source: String,
}

/// The items of one per-file module while they are rendered.
#[derive(Debug, Default)]
struct Module {
    /// Theme types its constants use.
    types: BTreeSet<&'static str>,
    /// Its constants.
    items: String,
}

/// The module of `file`: its name without `.toml`.
fn module_name(file: &str) -> &str {
    file.strip_suffix(".toml").unwrap_or(file)
}

/// The Rust sources of `generated.rs` and of one module per token file.
///
/// # Errors
/// Returns an error if a group is unknown to the generator, has no tokens,
/// or spreads over several token files where the generator cannot split it
/// (a shared group, or a theme group with nested groups).
pub(super) fn render(groups: &[Group]) -> Result<Vec<RustFile>> {
    for group in groups {
        let top = group.path[0].as_str();
        let known = THEME_GROUPS.iter().any(|(field, _)| *field == top)
            || SHARED_GROUPS.iter().any(|(field, _, _)| *field == top);
        if !known {
            bail!(
                "token group `{}` has no Rust type in the generator",
                group.path.join(".")
            );
        }
    }
    let mut modules: BTreeMap<&'static str, Module> = BTreeMap::new();
    let mut theme_fields = [String::new(), String::new()];
    let mut assembly_types: BTreeSet<&str> = BTreeSet::from(["Theme"]);
    for (field, rust_type) in THEME_GROUPS {
        match files_under(groups, field).as_slice() {
            [] => bail!("token group `{field}` has no tokens"),
            [file] => render_whole_group(
                modules.entry(file).or_default(),
                &mut theme_fields,
                groups,
                file,
                field,
                rust_type,
            ),
            _ => {
                assembly_types.insert(rust_type);
                render_split_group(&mut modules, &mut theme_fields, groups, field, rust_type)?;
            }
        }
    }
    for (field, constant, rust_type) in SHARED_GROUPS {
        let [file] = files_under(groups, field)[..] else {
            bail!("token group `{field}` must come from exactly one token file");
        };
        let module = modules.entry(file).or_default();
        module.types.insert(rust_type);
        let _ = write!(
            module.items,
            "\n/// {} tokens, the same in both built-in themes.\npub(super) const {constant}: {rust_type} = ",
            title(groups, field)
        );
        render_struct(
            &mut module.items,
            groups,
            &[field.to_owned()],
            rust_type,
            Variant::Dark,
        );
        module.items.push_str(";\n");
        for fields in &mut theme_fields {
            let _ = write!(fields, "{field}: {}::{constant},", module_name(file));
        }
    }
    let mut files = vec![RustFile {
        path: ASSEMBLY_PATH.to_owned(),
        source: render_assembly(&modules, &assembly_types, &theme_fields),
    }];
    files.extend(modules.iter().map(|(file, module)| RustFile {
        path: format!("{MODULE_DIRECTORY}/{}.rs", module_name(file)),
        source: render_module(file, module),
    }));
    Ok(files)
}

/// Renders a theme group whose tokens all come from `file` as one constant
/// per theme in that file's module.
fn render_whole_group(
    module: &mut Module,
    theme_fields: &mut [String; 2],
    groups: &[Group],
    file: &str,
    field: &str,
    rust_type: &'static str,
) {
    module.types.insert(rust_type);
    if groups
        .iter()
        .any(|group| group.path.len() > 1 && group.path[0] == field)
    {
        module.types.insert(NESTED_TYPE);
    }
    for variant in Variant::BOTH {
        let constant = format!("{}_{}", variant.prefix(), field.to_uppercase());
        let _ = write!(
            module.items,
            "\n/// {} of the {} theme.\npub(super) const {constant}: {rust_type} = ",
            title(groups, field),
            variant.name()
        );
        render_struct(
            &mut module.items,
            groups,
            &[field.to_owned()],
            rust_type,
            variant,
        );
        module.items.push_str(";\n");
        let _ = write!(
            theme_fields[variant.index()],
            "{field}: {}::{constant},",
            module_name(file)
        );
    }
}

/// The token files of the group `field` and its nested groups, sorted.
fn files_under(groups: &[Group], field: &str) -> Vec<&'static str> {
    let files: BTreeSet<&'static str> = groups
        .iter()
        .filter(|group| group.path[0] == field)
        .flat_map(|group| group.tokens.iter().map(|token| token.file))
        .collect();
    files.into_iter().collect()
}

/// The title of the group `field`, or the field itself.
fn title(groups: &[Group], field: &str) -> String {
    groups
        .iter()
        .find(|group| group.path == [field])
        .and_then(|group| group.title.clone())
        .unwrap_or_else(|| field.to_owned())
}

/// Renders a theme group whose tokens come from several token files (Q488):
/// every token becomes a constant in the module of its own file, and the
/// theme literal assembles the struct from them.
///
/// # Errors
/// Returns an error if the group has nested groups.
fn render_split_group(
    modules: &mut BTreeMap<&'static str, Module>,
    theme_fields: &mut [String; 2],
    groups: &[Group],
    field: &str,
    rust_type: &str,
) -> Result<()> {
    if groups
        .iter()
        .any(|group| group.path.len() > 1 && group.path[0] == field)
    {
        bail!("token group `{field}` spreads over several token files and has nested groups");
    }
    let Some(group) = groups.iter().find(|group| group.path == [field]) else {
        bail!("token group `{field}` has no tokens");
    };
    for fields in theme_fields.iter_mut() {
        let _ = write!(fields, "{field}: {rust_type} {{");
    }
    for token in &group.tokens {
        let module = modules.entry(token.file).or_default();
        let module_name = module_name(token.file);
        let name = format!("{field}_{}", token.field).to_uppercase();
        match token.value {
            TokenValue::Number { .. } => {
                render_token_constant(&mut module.items, token, &name, "f32", None);
                for fields in theme_fields.iter_mut() {
                    let _ = write!(fields, "{}: {module_name}::{name},", token.field);
                }
            }
            TokenValue::Colors { .. } => {
                for variant in Variant::BOTH {
                    let constant = format!("{}_{name}", variant.prefix());
                    render_token_constant(
                        &mut module.items,
                        token,
                        &constant,
                        "Color",
                        Some(variant),
                    );
                    let _ = write!(
                        theme_fields[variant.index()],
                        "{}: {module_name}::{constant},",
                        token.field
                    );
                }
            }
        }
    }
    for fields in theme_fields.iter_mut() {
        fields.push_str("},");
    }
    Ok(())
}

/// Renders one token as a documented constant; `variant` is `None` for a
/// number, which is the same in both themes.
fn render_token_constant(
    items: &mut String,
    token: &Token,
    constant: &str,
    rust_type: &str,
    variant: Option<Variant>,
) {
    let theme = variant.map_or_else(
        || "both built-in themes".to_owned(),
        |variant| format!("the {} theme", variant.name()),
    );
    let _ = write!(
        items,
        "\n/// `{}` in {theme}: {}.\npub(super) const {constant}: {rust_type} = {};\n",
        token.name,
        token.purpose.trim_end_matches('.'),
        render_value(token.value, variant.unwrap_or(Variant::Dark))
    );
}

/// The source of `generated.rs`: the module declarations and both themes.
fn render_assembly(
    modules: &BTreeMap<&'static str, Module>,
    types: &BTreeSet<&str>,
    theme_fields: &[String; 2],
) -> String {
    let mut source = String::from(
        "//! Generated by `cargo xtask tokens` from `design/tokens/`; do not edit.\n\
         //! Assembles the built-in themes from the values of each token file,\n\
         //! which live in the submodule of the same name (ROADMAP R118, R133);\n\
         //! the types are in the sibling modules of `theme`.\n\n",
    );
    for file in modules.keys() {
        let _ = writeln!(source, "mod {};", module_name(file));
    }
    let types: Vec<&str> = types.iter().copied().collect();
    let _ = write!(
        source,
        "\nuse super::{{{}}};\n\nimpl Theme {{\n",
        types.join(", ")
    );
    source.push_str(
        "/// The built-in dark theme (default): ink-tinted surfaces, violet accent.\n\
         pub const DARK: Self = Self {",
    );
    source.push_str(&theme_fields[Variant::Dark.index()]);
    source.push_str(
        "};\n\n/// The built-in light theme: parchment-warm surfaces, violet accent.\n\
         pub const LIGHT: Self = Self {",
    );
    source.push_str(&theme_fields[Variant::Light.index()]);
    source.push_str("};\n}\n");
    source
}

/// The source of the module holding the values of `file`.
fn render_module(file: &str, module: &Module) -> String {
    let mut source = format!(
        "//! Generated by `cargo xtask tokens` from `design/tokens/{file}`; do not edit.\n\
         //! The values of that token file; `generated.rs` assembles them into\n\
         //! the built-in themes (ROADMAP R118, R133).\n\n"
    );
    if module.items.contains("Color::") {
        source.push_str("use crate::color::Color;\n");
    }
    if !module.types.is_empty() {
        let types: Vec<&str> = module.types.iter().copied().collect();
        let _ = writeln!(source, "use crate::theme::{{{}}};", types.join(", "));
    }
    source.push_str(&module.items);
    source
}

/// Renders the struct literal of the group at `path` with its tokens and
/// nested groups.
fn render_struct(
    source: &mut String,
    groups: &[Group],
    path: &[String],
    rust_type: &str,
    variant: Variant,
) {
    let _ = write!(source, "{rust_type} {{");
    if let Some(group) = groups.iter().find(|group| group.path == path) {
        for token in &group.tokens {
            let _ = write!(
                source,
                "{}: {},",
                token.field,
                render_value(token.value, variant)
            );
        }
    }
    for child in groups
        .iter()
        .filter(|group| group.path.len() == path.len() + 1 && group.path.starts_with(path))
    {
        let _ = write!(source, "{}: ", child.path[path.len()]);
        render_struct(source, groups, &child.path, NESTED_TYPE, variant);
        source.push(',');
    }
    source.push('}');
}

/// A token's value as a Rust expression for `variant`.
fn render_value(value: TokenValue, variant: Variant) -> String {
    match value {
        TokenValue::Number { value, .. } => format!("{value:?}"),
        TokenValue::Colors { dark, light } => {
            let color = if variant == Variant::Dark {
                dark
            } else {
                light
            };
            match color {
                ColorValue::Opaque(rgb) => format!("Color::from_rgb_hex(0x{rgb:06X})"),
                ColorValue::WithAlphaByte(rgba) => format!(
                    "Color::from_rgba_hex(0x{:04X}_{:04X})",
                    rgba >> 16,
                    rgba & 0xFFFF
                ),
                ColorValue::WithAlpha(rgb, alpha) => {
                    format!("Color::from_rgb_hex(0x{rgb:06X}).with_alpha({alpha:?})")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_docs::source::Unit;

    /// A token of `file` with `field` and `value`.
    fn token(file: &'static str, field: &str, value: TokenValue) -> Token {
        Token {
            file,
            field: field.to_owned(),
            name: field.to_owned(),
            purpose: "p".to_owned(),
            value,
        }
    }

    #[test]
    fn colors_render_like_the_hand_written_theme() {
        let colors = |dark| TokenValue::Colors { dark, light: dark };
        assert_eq!(
            render_value(colors(ColorValue::Opaque(0x0F1318)), Variant::Dark),
            "Color::from_rgb_hex(0x0F1318)"
        );
        assert_eq!(
            render_value(
                colors(ColorValue::WithAlphaByte(0x0507_0AB8)),
                Variant::Dark
            ),
            "Color::from_rgba_hex(0x0507_0AB8)"
        );
        assert_eq!(
            render_value(colors(ColorValue::WithAlpha(0xFFFFFF, 0.06)), Variant::Dark),
            "Color::from_rgb_hex(0xFFFFFF).with_alpha(0.06)"
        );
    }

    #[test]
    fn an_unknown_group_is_an_error() {
        let group = Group {
            path: vec!["sparkle".to_owned()],
            section: "shared".to_owned(),
            title: None,
            tokens: Vec::new(),
        };
        assert!(render(&[group]).is_err());
    }

    #[test]
    fn a_split_group_takes_each_token_from_its_own_module() {
        let mut modules = BTreeMap::new();
        let mut theme_fields = [String::new(), String::new()];
        let group = Group {
            path: vec!["border".to_owned()],
            section: "shared".to_owned(),
            title: None,
            tokens: vec![
                token(
                    "colors.toml",
                    "color",
                    TokenValue::Colors {
                        dark: ColorValue::Opaque(0x000000),
                        light: ColorValue::Opaque(0xFFFFFF),
                    },
                ),
                token(
                    "border.toml",
                    "width",
                    TokenValue::Number {
                        value: 1.0,
                        unit: Unit::Pixels,
                    },
                ),
            ],
        };
        render_split_group(
            &mut modules,
            &mut theme_fields,
            &[group],
            "border",
            "BorderTokens",
        )
        .unwrap();
        assert_eq!(
            theme_fields[1],
            "border: BorderTokens {color: colors::LIGHT_BORDER_COLOR,width: border::BORDER_WIDTH,},"
        );
        assert!(modules["colors.toml"].items.contains(
            "pub(super) const DARK_BORDER_COLOR: Color = Color::from_rgb_hex(0x000000);"
        ));
        assert!(
            modules["border.toml"]
                .items
                .contains("pub(super) const BORDER_WIDTH: f32 = 1.0;")
        );
    }
}
