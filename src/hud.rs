//! The HUD over the 3D view: controls that change the preview rather than the species, and
//! figures about the plant. Its buttons have fixed colors, not the feathers theme: they sit over
//! the scene, not over a panel, so they would not follow a light or dark theme.

use bevy::{
    feathers::{constants::size, display::label},
    picking::{cursor::EntityCursor, events::PointerDrag, hover::Hovered},
    prelude::*,
    ui::Pressed,
    ui_widgets::{Activate, Button},
    window::SystemCursorIcon,
};

use crate::plant::{PlantEnvelope, PlantStats};

/// A HUD button's background, at rest, under the pointer and pressed: a gray that shows over the
/// sky and the ground alike, faint enough not to draw the eye from the plant.
const BUTTON_BG: Color = Color::srgba(0.5, 0.5, 0.5, 0.2);
const BUTTON_BG_HOVER: Color = Color::srgba(0.5, 0.5, 0.5, 0.3);
const BUTTON_BG_PRESSED: Color = Color::srgba(0.5, 0.5, 0.5, 0.4);

/// A toggle's icon while what it shows is shown, and while it is hidden.
const ICON_ON: Color = Color::WHITE;
const ICON_OFF: Color = Color::srgba(1.0, 1.0, 1.0, 0.4);

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (style_hud_buttons, show_triangles));
    }
}

/// The ground disc, which the HUD's ground toggle hides.
#[derive(Component, Default, Clone)]
pub struct Ground;

/// What a HUD toggle shows and hides.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum HudTarget {
    #[default]
    Envelope,
    Ground,
}

/// A HUD button that shows and hides its target, its icon dimmed while the target is hidden.
#[derive(Component, Default, Clone)]
pub struct HudToggle(pub HudTarget);

/// Marks a HUD button, for its colors.
#[derive(Component, Default, Clone)]
struct HudButton;

/// The text showing the plant's triangle count.
#[derive(Component, Default, Clone)]
struct TriangleCount;

/// The HUD, laid over the view as a grid with the panel's spacing: toggles in the upper right,
/// the triangle count in the lower left. Only its buttons take the pointer, so the view's drags
/// and scrolls pass between them.
pub fn hud() -> impl Scene {
    bsn! {
        #Hud
        Node {
            display: Display::Grid,
            width: percent(100),
            height: percent(100),
            padding: px(8),
            row_gap: px(8),
            column_gap: px(8),
            grid_template_columns: {vec![GridTrack::flex(1.0), GridTrack::auto(), GridTrack::auto()]},
            grid_template_rows: {vec![GridTrack::auto(), GridTrack::flex(1.0), GridTrack::auto()]},
        }
        Pickable::IGNORE
        // A drag that starts on a button would otherwise reach the view and turn the camera.
        on(|mut drag: On<PointerDrag>| drag.propagate(false))
        Children [
            @hud_toggle("icons/language.png")
            #envelope
            HudToggle(HudTarget::Envelope)
            Node { grid_row: GridPlacement::start(1), grid_column: GridPlacement::start(2) }
            --
            @hud_toggle("icons/landscape.png")
            #ground
            HudToggle(HudTarget::Ground)
            Node { grid_row: GridPlacement::start(1), grid_column: GridPlacement::start(3) }
            --
            @label("")
            #triangles
            TriangleCount
            Pickable::IGNORE
            Node {
                grid_row: GridPlacement::start(3),
                grid_column: GridPlacement::start(1),
                justify_self: JustifySelf::Start,
            }
        ]
    }
}

/// A square button the size of feathers' tool button, holding an icon.
fn hud_toggle(icon: &'static str) -> impl Scene {
    bsn! {
        Node {
            width: size::ROW_HEIGHT,
            height: size::ROW_HEIGHT,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(4)),
        }
        Button
        HudButton
        Hovered
        EntityCursor::System(SystemCursorIcon::Pointer)
        BackgroundColor(BUTTON_BG)
        on(toggle_target)
        Children [
            Node { height: px(18) }
            ImageNode { image: icon }
            Pickable::IGNORE
        ]
    }
}

fn toggle_target(
    activate: On<Activate>,
    toggles: Query<&HudToggle>,
    mut envelopes: Query<&mut Visibility, (With<PlantEnvelope>, Without<Ground>)>,
    mut grounds: Query<&mut Visibility, (With<Ground>, Without<PlantEnvelope>)>,
) {
    let Ok(toggle) = toggles.get(activate.entity) else {
        return;
    };
    let flip = |mut visibility: Mut<Visibility>| {
        *visibility = match *visibility {
            Visibility::Hidden => Visibility::Inherited,
            _ => Visibility::Hidden,
        };
    };
    match toggle.0 {
        HudTarget::Envelope => envelopes.iter_mut().for_each(flip),
        HudTarget::Ground => grounds.iter_mut().for_each(flip),
    }
}

/// Colors each HUD button by the pointer, and dims a toggle's icon while its target is hidden.
fn style_hud_buttons(
    mut buttons: Query<
        (
            &mut BackgroundColor,
            &Hovered,
            Has<Pressed>,
            Option<&HudToggle>,
            &Children,
        ),
        With<HudButton>,
    >,
    mut icons: Query<&mut ImageNode>,
    envelopes: Query<&Visibility, With<PlantEnvelope>>,
    grounds: Query<&Visibility, With<Ground>>,
) {
    for (mut background, hovered, pressed, toggle, children) in &mut buttons {
        let color = match (pressed, hovered.0) {
            (true, _) => BUTTON_BG_PRESSED,
            (false, true) => BUTTON_BG_HOVER,
            (false, false) => BUTTON_BG,
        };
        background.set_if_neq(BackgroundColor(color));

        let Some(toggle) = toggle else {
            continue;
        };
        let hidden = match toggle.0 {
            HudTarget::Envelope => envelopes.iter().any(|v| *v == Visibility::Hidden),
            HudTarget::Ground => grounds.iter().any(|v| *v == Visibility::Hidden),
        };
        let tint = if hidden { ICON_OFF } else { ICON_ON };
        for &child in children {
            if let Ok(mut icon) = icons.get_mut(child)
                && icon.color != tint
            {
                icon.color = tint;
            }
        }
    }
}

fn show_triangles(
    stats: Query<&PlantStats, Changed<PlantStats>>,
    mut texts: Query<&mut Text, With<TriangleCount>>,
) {
    let Ok(stats) = stats.single() else {
        return;
    };
    for mut text in &mut texts {
        text.0 = format!("{} triangles", stats.triangles);
    }
}
