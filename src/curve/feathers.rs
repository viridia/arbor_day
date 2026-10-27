//! Feathers widgets for curves.

use bevy::{
    feathers::{
        constants::{fonts, size},
        controls::{
            FeathersMenu, FeathersMenuDivider, FeathersMenuItem, FeathersMenuPopup,
            FeathersMenuToolButton, FeathersNumberInput, HardLimit, NumberInputPrecision,
            NumberInputUnits, UnitsRegistry,
        },
        theme::{
            ThemeBackgroundColor, ThemeBorderColor, ThemeTextColor, ThemeToken, ThemedText,
            UiTheme,
        },
        tokens,
    },
    input_focus::{Focusable, tab_navigation::TabIndex},
    picking::hover::Hovered,
    prelude::*,
    text::{FontSourceTemplate, LineHeight},
    ui::InteractionDisabled,
    ui_widgets::{
        Activate, NumericFormat, NumericRange, NumericValue, ValueChange,
        popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide},
    },
};

use super::{
    edit::{CurveEdit, CurveEditPhase, CurveSelection, PointRef, SeenCurve},
    gesture::{CurveGesture, CurveGestures, CurveHover},
    model::{CurveConstraints, CurveEditOp, CurveValue, PinEnds, ScalarCurve},
    plot::CurveStroke,
    view::{CurvePlot, CurveView},
};

/// The editor's plot, its labels included, in logical pixels.
const PLOT_HEIGHT: f32 = 172.0;
/// The width of the y labels' gutter, left of the plot.
const GUTTER: f32 = 34.0;
/// The space between the plot and the x labels.
const LABEL_GAP: f32 = 4.0;
/// The height of the x labels' strip, under the plot, the gap included.
const LABEL_STRIP: f32 = 16.0 + LABEL_GAP;
/// How far the plot keeps the curve from its edges.
const INSET: f32 = 8.0;
/// Half a label's line height, so a label centres on its grid line.
const HALF_LABEL: f32 = 7.0;

/// The grid at a quarter of each axis, the ends and middle drawn brighter.
const GRID_MINOR: ThemeToken = tokens::PANE_BODY_BG;
const GRID_MAJOR: ThemeToken = tokens::GROUP_BORDER;

/// The selection's color: a selected point's ring, and the rectangle's outline and, faintly, its
/// fill.
const SELECTION: ThemeToken = tokens::CHECKBOX_BG_CHECKED;
const SELECTION_FILL_ALPHA: f32 = 0.14;

/// A curve drawn small, with no gestures, as a collapsed row shows it. Insert a [`CurveValue`]
/// and its [`CurveConstraints`] on the same entity, and write the value when the curve changes.
#[derive(Component, Clone, Default)]
pub struct FeathersCurveThumbnail;

/// A [`FeathersCurveThumbnail`] at its row's right end, as tall as a number input and twice as
/// wide, the open plot's shape: the editor's plot, small, its background, stroke and grid inside
/// the ends, so the two read as one widget, the curve keeps its slopes, and a flat curve reads as
/// a graph.
pub fn curve_thumbnail() -> impl Scene {
    bsn! {
        FeathersCurveThumbnail
        Node {
            width: px(48),
            height: px(24),
            flex_shrink: 0.0,
            margin: UiRect::left(Val::Auto),
            border_radius: BorderRadius::all(px(4)),
        }
        ThemeBackgroundColor(tokens::WINDOW_BG)
        Children [
            Node {
                position_type: PositionType::Absolute,
                left: px(3),
                right: px(3),
                top: px(3),
                bottom: px(3),
            }
            Pickable::IGNORE
            Children [
                @grid_line(true, 0.25, false)
                -- @grid_line(true, 0.5, true)
                -- @grid_line(true, 0.75, false)
                -- @grid_line(false, 0.5, true)
            ]
            --
            CurvePlot { inset: 3.0 }
            CurveStroke { color: tokens::SLIDER_BAR, width: 1.5 }
            @fill()
            Pickable::IGNORE
        ]
    }
}

/// A curve editor: a plot whose points are selected, dragged, inserted by double-click and
/// deleted with Delete, labelled at the ends and middle of each axis, over number inputs for the
/// selected point's x and y, captioned `x_caption` and `y_caption`. Insert a [`CurveValue`], its
/// [`CurveConstraints`] and a [`CurveEditorFormat`] on the same entity; it receives each
/// [`CurveEdit`], and the curve is written back as the host judges them.
pub fn curve_editor(x_caption: &'static str, y_caption: &'static str) -> impl Scene {
    bsn! {
        FeathersCurveEditor
        CurveSelection
        SeenCurve
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
        }
        Children [
            Node { height: px(PLOT_HEIGHT) }
            Children [
                // The y labels, top to bottom, each centred on its grid line.
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    width: px(GUTTER - 4.0),
                    top: px(INSET - HALF_LABEL),
                    bottom: px(LABEL_STRIP + INSET - HALF_LABEL),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexEnd,
                }
                Children [
                    @axis_label(true, 1.0, Justify::Right)
                    --
                    @axis_label(true, 0.5, Justify::Right)
                    --
                    @axis_label(true, 0.0, Justify::Right)
                ]
                --
                Node {
                    position_type: PositionType::Absolute,
                    left: px(GUTTER),
                    right: px(0),
                    top: px(0),
                    bottom: px(LABEL_STRIP),
                    border_radius: BorderRadius::all(px(4)),
                }
                ThemeBackgroundColor(tokens::WINDOW_BG)
                Children [
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(INSET),
                        right: px(INSET),
                        top: px(INSET),
                        bottom: px(INSET),
                    }
                    Pickable::IGNORE
                    Children [
                        @grid_line(true, 0.0, true) -- @grid_line(true, 0.25, false)
                        -- @grid_line(true, 0.5, true) -- @grid_line(true, 0.75, false)
                        -- @grid_line(true, 1.0, true)
                        -- @grid_line(false, 0.0, true) -- @grid_line(false, 0.25, false)
                        -- @grid_line(false, 0.5, true) -- @grid_line(false, 0.75, false)
                        -- @grid_line(false, 1.0, true)
                    ]
                    --
                    #plot
                    CurvePlot { inset: INSET }
                    CurveStroke { color: tokens::SLIDER_BAR, width: 1.75 }
                    CurveGestures
                    Focusable
                    TabIndex(0)
                    @fill()
                    Children [
                        // Before the markers, which the editor adds as the curve's points, so
                        // they draw over it.
                        SelectionRectOutline
                        Node {
                            position_type: PositionType::Absolute,
                            display: Display::None,
                            border: UiRect::all(px(1)),
                        }
                        ThemeBorderColor(SELECTION)
                        BackgroundColor
                        Pickable::IGNORE
                    ]
                ]
                --
                // The x labels: the first starts at its line, the last ends at its, the middle
                // centres on its.
                Node {
                    position_type: PositionType::Absolute,
                    left: px(GUTTER + INSET),
                    right: px(INSET),
                    bottom: px(0),
                    height: px(LABEL_STRIP),
                    padding: UiRect::top(px(LABEL_GAP)),
                }
                Children [
                    @axis_label(false, 0.0, Justify::Left)
                    Node { flex_basis: px(0), flex_grow: 1.0 }
                    --
                    @axis_label(false, 0.5, Justify::Center)
                    Node { flex_basis: px(0), flex_grow: 1.0 }
                    --
                    @axis_label(false, 1.0, Justify::Right)
                    Node { flex_basis: px(0), flex_grow: 1.0 }
                ]
            ]
            --
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(4),
            }
            Children [
                @FeathersNumberInput { @interior_label_text: {Some(x_caption)} }
                #x
                PointInput { y: false }
                Node { flex_grow: 1.0 }
                --
                @FeathersNumberInput { @interior_label_text: {Some(y_caption)} }
                #y
                PointInput { y: true }
                Node { flex_grow: 1.0 }
            ]
        ]
    }
}

/// Marks a [`curve_editor`].
#[derive(Component, Clone, Default)]
pub struct FeathersCurveEditor;

/// On a [`curve_editor`]: how its axes' values are written, in the labels and the inputs.
#[derive(Component, Clone, Debug, Default)]
pub struct CurveEditorFormat {
    pub x_units: Option<NumberInputUnits>,
    pub y_units: Option<NumberInputUnits>,
    /// Labels for the x axis's start, middle and end in place of their values.
    pub x_labels: Option<[&'static str; 3]>,
}

/// A curve editor's ⋯ menu, at its row's right end, for the host to place in the editor's header
/// line and aim with a [`CurveMenuTarget`]. It lists the host's `items`, if any, above a divider,
/// then Copy curve and Paste curve, named `copy` and `paste`. An item holding a
/// [`CurveMenuReplace`] replaces the curve when picked; the host observes its other items itself.
pub fn curve_menu(items: Option<Box<dyn SceneList>>) -> impl Scene {
    bsn! {
        @FeathersMenu
        Node { margin: UiRect::left(Val::Auto) }
        Children [
            @FeathersMenuToolButton {
                @caption: bsn! { Text("…") ThemedText },
                @arrow: false,
            }
            --
            @FeathersMenuPopup
            // Aligned to the button's right edge, as the button is at the row's right end.
            Popover {
                positions: vec![
                    PopoverPlacement {
                        side: PopoverSide::Bottom,
                        align: PopoverAlign::End,
                        gap: 2.0,
                    },
                    PopoverPlacement {
                        side: PopoverSide::Top,
                        align: PopoverAlign::End,
                        gap: 2.0,
                    },
                ],
                window_margin: 10.0,
            }
            Children [
                {items.map(|items| bsn_list! { {items} -- @FeathersMenuDivider })}
                --
                @curve_menu_item("Copy curve") ClipboardItem::Copy #copy
                --
                @curve_menu_item("Paste curve") ClipboardItem::Paste #paste
            ]
        ]
    }
}

/// A menu item captioned `caption`, for a [`curve_menu`]'s items.
pub fn curve_menu_item(caption: &'static str) -> impl Scene {
    bsn! { @FeathersMenuItem { @caption: bsn! { Text(caption) ThemedText } } }
}

/// On a [`curve_menu`]: the editor it acts on.
#[derive(Component, Clone, Copy, Debug)]
pub struct CurveMenuTarget(pub Entity);

/// On an item of a [`curve_menu`]: picked, it proposes this curve in place of the editor's.
#[derive(Component, Clone, Debug, Default)]
pub struct CurveMenuReplace(pub ScalarCurve);

/// The curve last copied from any [`curve_menu`]. The app's own rather than the system clipboard,
/// so Paste curve knows when it has nothing to paste.
#[derive(Resource, Clone, Debug, Default)]
struct CurveClipboard(Option<ScalarCurve>);

/// A [`curve_menu`]'s own items.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ClipboardItem {
    #[default]
    Copy,
    Paste,
}

/// An absolutely placed node filling its parent.
fn fill() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
        }
    }
}

/// A grid line, `vertical` or across, at `share` of its parent, which spans the axes' bounds.
fn grid_line(vertical: bool, share: f32, major: bool) -> impl Scene {
    let at = percent(share * 100.0);
    let (left, top, width, height) = if vertical {
        (at, px(0), px(1), percent(100))
    } else {
        (px(0), at, percent(100), px(1))
    };
    let color = if major { GRID_MAJOR } else { GRID_MINOR };
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            left: {left},
            top: {top},
            width: {width},
            height: {height},
        }
        ThemeBackgroundColor({color})
    }
}

/// A grid label, its text written from the editor's constraints and format.
#[derive(Component, Clone, Copy, Debug, Default)]
struct AxisLabel {
    y: bool,
    /// Where along the axis, 0 at its minimum.
    share: f32,
}

fn axis_label(y: bool, share: f32, justify: Justify) -> impl Scene {
    bsn! {
        Text("")
        TextFont {
            font: FontSourceTemplate::Families(fonts::SANS),
            font_size: size::COMPACT_FONT,
        }
        TextLayout { justify: {justify} }
        ThemeTextColor(tokens::TEXT_DIM)
        LineHeight::Px({HALF_LABEL * 2.0})
        AxisLabel { y: {y}, share: {share} }
        Pickable::IGNORE
    }
}

/// A number input for the selected point's x, or its `y`.
#[derive(Component, Clone, Copy, Debug, Default)]
struct PointInput {
    y: bool,
}

/// The rectangle being dragged out on the plot it is a child of, hidden while there is none.
#[derive(Component, Clone, Copy, Debug, Default)]
struct SelectionRectOutline;

/// A node marking point `index` of the plot it is a child of.
#[derive(Component, Clone, Copy, Debug)]
struct PointMarker {
    index: usize,
    look: MarkerLook,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MarkerLook {
    Plain,
    Hovered,
    Selected,
}

impl MarkerLook {
    /// Radius, border width, fill and border.
    fn style(self) -> (f32, f32, ThemeToken, ThemeToken) {
        match self {
            Self::Plain => (4.0, 1.5, tokens::TEXT_MAIN, tokens::WINDOW_BG),
            Self::Hovered => (4.0, 1.5, tokens::BUTTON_TEXT, tokens::WINDOW_BG),
            Self::Selected => (5.0, 2.5, tokens::BUTTON_TEXT, SELECTION),
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CurveClipboard>()
        .add_observer(edit_from_input)
        .add_observer(pick_menu_item)
        .add_systems(Update, enable_paste)
        .add_systems(
            PostUpdate,
            (
                place_markers,
                place_selection_rect,
                label_axes,
                show_selected_point.after(super::edit::follow_point_count),
            )
                .after(bevy::ui::UiSystems::Layout),
        );
}

/// Copies the editor's curve, or proposes the clipboard's or an item's curve in its place.
fn pick_menu_item(
    activate: On<Activate>,
    items: Query<
        (Option<&ClipboardItem>, Option<&CurveMenuReplace>),
        Or<(With<ClipboardItem>, With<CurveMenuReplace>)>,
    >,
    parents: Query<&ChildOf>,
    menus: Query<&CurveMenuTarget>,
    curves: Query<&CurveValue>,
    mut clipboard: ResMut<CurveClipboard>,
    mut commands: Commands,
) {
    let Ok((own, replace)) = items.get(activate.entity) else {
        return;
    };
    let Some(editor) = (parents.iter_ancestors(activate.entity))
        .find_map(|ancestor| menus.get(ancestor).ok())
        .map(|target| target.0)
    else {
        return;
    };
    let curve = match (own, replace) {
        (Some(ClipboardItem::Copy), _) => {
            // One curve per editor for now: the curve is on the editor's own entity.
            if let Ok(value) = curves.get(editor) {
                clipboard.0 = Some(value.0.clone());
            }
            return;
        }
        (Some(ClipboardItem::Paste), _) => clipboard.0.clone(),
        (None, replace) => replace.map(|replace| replace.0.clone()),
    };
    let Some(curve) = curve else {
        return;
    };
    commands.trigger(CurveEdit {
        editor,
        curve: editor,
        op: CurveEditOp::Replace(curve),
        phase: CurveEditPhase::Finish,
    });
}

/// Disables every Paste curve while the clipboard is empty.
fn enable_paste(
    clipboard: Res<CurveClipboard>,
    items: Query<(Entity, &ClipboardItem, Has<InteractionDisabled>)>,
    mut commands: Commands,
) {
    for (item, own, disabled) in &items {
        let empty = clipboard.0.is_none();
        if *own == ClipboardItem::Paste && empty != disabled {
            if empty {
                commands.entity(item).insert(InteractionDisabled);
            } else {
                commands.entity(item).remove::<InteractionDisabled>();
            }
        }
    }
}

/// Keeps a marker per point on each editor's plot, placed and styled from the selection and hover.
fn place_markers(
    plots: Query<
        (
            Entity,
            &CurvePlot,
            &ComputedNode,
            &CurveHover,
            Option<&Children>,
        ),
        With<CurveGestures>,
    >,
    parents: Query<&ChildOf>,
    curves: Query<(&CurveValue, &CurveConstraints, &CurveSelection)>,
    mut markers: Query<(&mut PointMarker, &mut Node)>,
    mut commands: Commands,
) {
    for (plot, settings, node, hover, children) in &plots {
        let owner = super::view::curve_owner(plot, &parents, |owner| curves.contains(owner));
        let Some((curve_entity, (curve, constraints, selection))) =
            owner.and_then(|owner| Some((owner, curves.get(owner).ok()?)))
        else {
            continue;
        };
        let size = node.size() * node.inverse_scale_factor();
        let view = CurveView::new(constraints, size, settings.inset);
        let mut placed = 0;
        for child in children.into_iter().flatten() {
            let Ok((mut marker, mut marker_node)) = markers.get_mut(*child) else {
                continue;
            };
            let Some(point) = curve.0.points.get(marker.index) else {
                commands.entity(*child).despawn();
                continue;
            };
            placed = placed.max(marker.index + 1);
            let point_ref = PointRef {
                curve: curve_entity,
                index: marker.index,
            };
            let look = if selection.points.contains(&point_ref) {
                MarkerLook::Selected
            } else if hover.0 == Some(point_ref) {
                MarkerLook::Hovered
            } else {
                MarkerLook::Plain
            };
            let (radius, border, fill, edge) = look.style();
            let centre = view.to_pixels(Vec2::new(point.x, point.y));
            let (left, top) = (px(centre.x - radius), px(centre.y - radius));
            if marker_node.left != left || marker_node.top != top || marker.look != look {
                marker_node.left = left;
                marker_node.top = top;
                marker_node.width = px(radius * 2.0);
                marker_node.height = px(radius * 2.0);
                marker_node.border = UiRect::all(px(border));
            }
            if marker.look != look {
                marker.look = look;
                commands
                    .entity(*child)
                    .insert((ThemeBackgroundColor(fill), ThemeBorderColor(edge)));
            }
        }
        for index in placed..curve.0.points.len() {
            let look = MarkerLook::Plain;
            let (radius, border, fill, edge) = look.style();
            let centre =
                view.to_pixels(Vec2::new(curve.0.points[index].x, curve.0.points[index].y));
            commands.spawn((
                PointMarker { index, look },
                Name::new(format!("point{index}")),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(centre.x - radius),
                    top: px(centre.y - radius),
                    width: px(radius * 2.0),
                    height: px(radius * 2.0),
                    border: UiRect::all(px(border)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                ThemeBackgroundColor(fill),
                ThemeBorderColor(edge),
                Pickable::IGNORE,
                ChildOf(plot),
            ));
        }
    }
}

/// Draws the rectangle being dragged out on each plot, and hides it otherwise.
fn place_selection_rect(
    plots: Query<
        (Entity, &CurvePlot, &ComputedNode, &CurveGesture, &Children),
        Changed<CurveGesture>,
    >,
    parents: Query<&ChildOf>,
    curves: Query<&CurveConstraints>,
    mut outlines: Query<(&mut Node, &mut BackgroundColor), With<SelectionRectOutline>>,
    theme: Res<UiTheme>,
) {
    for (plot, settings, node, gesture, children) in &plots {
        let rect = match gesture {
            CurveGesture::SelectRect { from, to, .. } => Some((*from, *to)),
            _ => None,
        };
        let owner = super::view::curve_owner(plot, &parents, |owner| curves.contains(owner));
        let constraints = owner.and_then(|owner| curves.get(owner).ok());
        for child in children {
            let Ok((mut outline, mut fill)) = outlines.get_mut(*child) else {
                continue;
            };
            let (Some((from, to)), Some(constraints)) = (rect, constraints) else {
                outline.display = Display::None;
                continue;
            };
            let size = node.size() * node.inverse_scale_factor();
            let view = CurveView::new(constraints, size, settings.inset);
            let (a, b) = (view.to_pixels(from), view.to_pixels(to));
            let (min, max) = (a.min(b), a.max(b));
            outline.display = Display::Flex;
            outline.left = px(min.x);
            outline.top = px(min.y);
            outline.width = px(max.x - min.x);
            outline.height = px(max.y - min.y);
            // A theme token has no alpha of its own, so the fill is set from the theme here.
            fill.set_if_neq(BackgroundColor(
                theme.color(&SELECTION).with_alpha(SELECTION_FILL_ALPHA),
            ));
        }
    }
}

/// Writes each grid label from its editor's axes and format.
fn label_axes(
    editors: Query<
        (Entity, &CurveConstraints, &CurveEditorFormat),
        Or<(Changed<CurveConstraints>, Changed<CurveEditorFormat>)>,
    >,
    children: Query<&Children>,
    mut labels: Query<(&AxisLabel, &mut Text)>,
    registry: Res<UnitsRegistry>,
) {
    for (editor, constraints, format) in &editors {
        for descendant in children.iter_descendants(editor) {
            let Ok((label, mut text)) = labels.get_mut(descendant) else {
                continue;
            };
            let named = (format.x_labels)
                .filter(|_| !label.y)
                .map(|names| names[(label.share * 2.0).round() as usize].to_string());
            let written = named.unwrap_or_else(|| {
                let (axis, units) = if label.y {
                    (&constraints.y, &format.y_units)
                } else {
                    (&constraints.x, &format.x_units)
                };
                let value = axis.min + (axis.max - axis.min) * label.share;
                let units = registry.resolve(units.as_ref());
                units.format(
                    units.canonical_to_display(value as f64),
                    NumericFormat::F32,
                    false,
                )
            });
            if text.0 != written {
                text.0 = written;
            }
        }
    }
}

/// Shows the selected point's x and y in the inputs, which are enabled only while exactly one
/// point is selected, and only for a coordinate the constraints let it move in; with none or
/// several, they show 0. Each input rounds to its axis's precision, written in the input's units.
fn show_selected_point(
    editors: Query<
        (
            Entity,
            &CurveValue,
            Ref<CurveConstraints>,
            &CurveSelection,
            Ref<CurveEditorFormat>,
        ),
        Or<(
            Changed<CurveValue>,
            Changed<CurveConstraints>,
            Changed<CurveSelection>,
            Changed<CurveEditorFormat>,
        )>,
    >,
    children: Query<&Children>,
    inputs: Query<(&PointInput, &NumericValue, Has<InteractionDisabled>)>,
    registry: Res<UnitsRegistry>,
    mut commands: Commands,
) {
    for (editor, curve, constraints, selection, format) in &editors {
        let configure = format.is_changed() || constraints.is_changed();
        let point = (selection.single()).and_then(|point| {
            let at = curve.0.points.get(point.index)?;
            Some((point.index, Vec2::new(at.x, at.y)))
        });
        let last = curve.0.points.len().saturating_sub(1);
        for descendant in children.iter_descendants(editor) {
            let Ok((input, shown, disabled)) = inputs.get(descendant) else {
                continue;
            };
            let mut entity = commands.entity(descendant);
            if configure {
                let (axis, units) = if input.y {
                    (&constraints.y, &format.y_units)
                } else {
                    (&constraints.x, &format.x_units)
                };
                if let Some(units) = units {
                    entity.insert(units.clone());
                }
                if axis.precision > 0.0 {
                    let shown = (registry.resolve(units.as_ref()))
                        .canonical_to_display(axis.precision as f64);
                    entity.insert(NumberInputPrecision(-shown.log10().round() as i32));
                }
                if axis.clamped {
                    entity.insert(HardLimit(NumericRange::F32(axis.min..=axis.max)));
                }
            }
            let movable = point.is_some_and(|(index, _)| {
                let end = index == 0 || index == last;
                match constraints.pin_ends {
                    PinEnds::None => true,
                    PinEnds::X => input.y || !end,
                    PinEnds::XY => !end,
                }
            });
            if movable == disabled {
                if movable {
                    entity.remove::<InteractionDisabled>();
                    entity.queue(restyle_number_input);
                } else {
                    entity.insert(InteractionDisabled);
                }
            }
            // With no single point, 0 stands in for showing no value (X65).
            let value = NumericValue::F32(match point {
                Some((_, at)) if input.y => at.y,
                Some((_, at)) => at.x,
                None => 0.0,
            });
            if *shown != value || configure {
                entity.insert(value);
            }
        }
    }
}

/// Restyles a number input just enabled. Feathers restyles it as the disabled marker is removed,
/// while the marker is still there, so it stays drawn disabled (X64); a hover change restyles it
/// from the input's state, so its text field's hover is set again.
fn restyle_number_input(input: EntityWorldMut) {
    let children: Vec<Entity> = input
        .get::<Children>()
        .map(|children| children.to_vec())
        .unwrap_or_default();
    let world = input.into_world_mut();
    for child in children {
        if let Some(hovered) = world.get::<Hovered>(child).cloned() {
            world.entity_mut(child).insert(hovered);
        }
    }
}

/// A value typed or scrubbed into a point input proposes the selected point there.
fn edit_from_input(
    change: On<ValueChange<f32>>,
    inputs: Query<&PointInput>,
    parents: Query<&ChildOf>,
    editors: Query<(&CurveValue, &CurveSelection), With<FeathersCurveEditor>>,
    mut commands: Commands,
) {
    let Ok(input) = inputs.get(change.source) else {
        return;
    };
    let Some(editor) = parents
        .iter_ancestors(change.source)
        .find(|&ancestor| editors.contains(ancestor))
    else {
        return;
    };
    let Ok((curve, selection)) = editors.get(editor) else {
        return;
    };
    let Some(point) = selection.single() else {
        return;
    };
    let Some(at) = curve.0.points.get(point.index) else {
        return;
    };
    let at = if input.y {
        Vec2::new(at.x, change.value)
    } else {
        Vec2::new(change.value, at.y)
    };
    commands.trigger(CurveEdit {
        editor,
        curve: point.curve,
        op: CurveEditOp::SetPoint(point.index, at),
        phase: if change.is_final {
            CurveEditPhase::Finish
        } else {
            CurveEditPhase::Update
        },
    });
}
