//! Keyboard shortcuts shown as text, such as a menu item's key equivalent ("⌘⇧S").
//!
//! Fira Sans, feathers' font, lacks ⌘, ⌥, ⌃ and ⇥, so shortcut text falls back to Key Symbols
//! (`assets/fonts/KeySymbols.ttf`), a subset of DejaVu Sans holding only the key symbols Fira Sans
//! lacks. Parley picks a font per character from the family list, so the rest of the text stays
//! Fira Sans. ⎋ is in neither; write "Esc".

use bevy::{
    app::PropagateOver,
    feathers::{constants::size, theme::ThemeTextColor, tokens},
    prelude::*,
    text::FontSourceTemplate,
};

/// The shortcut font's family list: Fira Sans first, Key Symbols for what it lacks.
const SHORTCUT_FAMILIES: &str = "Fira Sans, Key Symbols";

pub struct ShortcutPlugin;

impl Plugin for ShortcutPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_key_symbols);
    }
}

/// Keeps Key Symbols loaded, so its family resolves by name.
#[derive(Resource)]
struct KeySymbols(#[expect(dead_code, reason = "held to keep the font loaded")] Handle<Font>);

fn load_key_symbols(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(KeySymbols(assets.load("fonts/KeySymbols.ttf")));
}

/// A shortcut's text, dimmed, at the right end of its row: a menu item's key equivalent.
pub fn shortcut(keys: impl Into<String>) -> impl Scene {
    bsn! {
        Text(keys)
        TextFont {
            font: {FontSourceTemplate::Families(SHORTCUT_FAMILIES.into())},
            font_size: size::MEDIUM_FONT,
            weight: FontWeight::NORMAL,
        }
        // The menu item's inherited font passes over this text rather than replacing its own.
        PropagateOver<TextFont>
        ThemeTextColor(tokens::TEXT_DIM)
        // At the row's right end, and clear of the caption when the menu is only as wide as its
        // widest item.
        Node {
            margin: UiRect::left(Val::Auto),
            padding: UiRect::left(px(24)),
        }
    }
}
