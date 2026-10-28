//! Feathers widgets for curves.

use bevy::{
    asset::embedded_asset,
    feathers::{
        constants::{fonts, size},
        controls::{
            FeathersMenu, FeathersMenuButton, FeathersMenuDivider, FeathersMenuItem,
            FeathersMenuPopup, FeathersMenuToolButton, FeathersNumberInput, HardLimit,
            NumberInputPrecision, NumberInputStep, NumberInputUnits, UnitsRegistry,
        },
        display::icon,
        theme::{
            ThemeBackgroundColor, ThemeBorderColor, ThemeTextColor, ThemeToken, ThemedText, UiTheme,
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
    gesture::{CurveGesture, CurveGestures, CurveHover, CurveTarget},
    model::{
        CurveConstraints, CurveEditOp, CurveSegment, CurveValue, HandleMode, HandleSide, PinEnds,
        ScalarCurve,
    },
    plot::CurveStroke,
    view::{CurvePath, CurvePlot, CurveView, flatten_segment, shown_handles},
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
/// selected point's x and y. Insert a [`CurveValue`] and its [`CurveConstraints`] on the same
/// entity; it receives each [`CurveEdit`], and the curve is written back as the host judges them.
/// Its fields say how the axes' values are written, in the labels and the inputs; by default as
/// plain numbers.
#[derive(SceneComponent, Clone, Debug, Default)]
#[scene(FeathersCurveEditorProps)]
pub struct FeathersCurveEditor {
    pub x_units: Option<NumberInputUnits>,
    pub y_units: Option<NumberInputUnits>,
    /// Labels for the x axis's start, middle and end in place of their values.
    pub x_labels: Option<[&'static str; 3]>,
}

/// Props used to construct a [`FeathersCurveEditor`] scene.
pub struct FeathersCurveEditorProps {
    /// Captions for the selected point's x and y inputs, by default `x` and `y`. None leaves the
    /// inputs out, for a curve whose points the author never types, such as one whose only points
    /// are its pinned ends.
    pub point_inputs: Option<(&'static str, &'static str)>,
    /// Whether the selected point's handles have inputs, by default true: a row each, In and Out,
    /// for the handle's offset from the point, shown while that handle is.
    pub handle_inputs: bool,
}

impl Default for FeathersCurveEditorProps {
    fn default() -> Self {
        Self {
            point_inputs: Some(("x", "y")),
            handle_inputs: true,
        }
    }
}

impl FeathersCurveEditor {
    fn scene(props: FeathersCurveEditorProps) -> impl Scene {
        bsn! {
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
                        // Before the markers, which the editor adds as the curve's points and
                        // handles, so they draw over these.
                        SpanHighlights
                        @fill()
                        Pickable::IGNORE
                        --
                        HandleStems
                        @fill()
                        Pickable::IGNORE
                        --
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
            {props.point_inputs.map(|(x_caption, y_caption)| bsn_list! {
                PointRow { placeholder: false }
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(4),
                    display: Display::None,
                }
                Children [
                    @FeathersNumberInput { @interior_label_text: {Some(x_caption)} }
                    #x
                    CoordinateInput { y: false }
                    Node { flex_grow: 1.0, flex_basis: px(0) }
                    --
                    @FeathersNumberInput { @interior_label_text: {Some(y_caption)} }
                    #y
                    CoordinateInput { y: true }
                    Node { flex_grow: 1.0, flex_basis: px(0) }
                ]
                --
                #no_point
                PointRow { placeholder: true }
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: px(4),
                }
                Children [
                    @placeholder_field(x_caption) -- @placeholder_field(y_caption)
                ]
            })}
            --
            {props.handle_inputs.then(|| bsn_list! {
                @handle_input_row(HandleSide::Left) -- @handle_input_row(HandleSide::Right)
            })}
        ]
        }
    }
}

/// The inputs for the selected point's handle on `side`, hidden while it shows none: `Δx In` and
/// `Δy In`, named `in_dx` and `in_dy`, or likewise Out.
fn handle_input_row(side: HandleSide) -> impl Scene {
    let (x_caption, y_caption, x_name, y_name) = match side {
        HandleSide::Left => ("Δx In", "Δy In", "in_dx", "in_dy"),
        HandleSide::Right => ("Δx Out", "Δy Out", "out_dx", "out_dy"),
    };
    let handle = Some(side);
    bsn! {
        HandleInputRow
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(4),
            display: Display::None,
        }
        Children [
            @FeathersNumberInput { @interior_label_text: {Some(x_caption)} }
            Name({x_name})
            CoordinateInput { handle: {handle}, y: false }
            Node { flex_grow: 1.0, flex_basis: px(0) }
            --
            @FeathersNumberInput { @interior_label_text: {Some(y_caption)} }
            Name({y_name})
            CoordinateInput { handle: {handle}, y: true }
            Node { flex_grow: 1.0, flex_basis: px(0) }
        ]
    }
}

/// A field drawn as a disabled number input captioned `caption`, showing "—" for no value.
fn placeholder_field(caption: &'static str) -> impl Scene {
    bsn! {
        Node {
            height: size::ROW_HEIGHT,
            flex_grow: 1.0,
            flex_basis: px(0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            // As far in as a number input's interior caption and value, past its chevrons.
            padding: UiRect::horizontal(px(20)),
            border_radius: BorderRadius::all(px(4)),
        }
        ThemeBackgroundColor(tokens::SLIDER_BG_DISABLED)
        Children [
            @placeholder_text(caption) -- @placeholder_text("—")
        ]
    }
}

fn placeholder_text(text: &'static str) -> impl Scene {
    bsn! {
        Text(text)
        TextFont {
            font: FontSourceTemplate::Families(fonts::SANS),
            font_size: size::COMPACT_FONT,
        }
        ThemeTextColor(tokens::TEXT_INPUT_TEXT_DISABLED)
        Pickable::IGNORE
    }
}

/// A curve editor's header tools, at its row's right end, for the host to place in the editor's
/// header line and aim with a [`CurveMenuTarget`]: dropdowns for the selected points' segment
/// kind and handle mode, named `kind` and `mode`, each there only while the constraints offer a
/// choice; then a ⋯ menu. The menu lists the host's `items`, if any, above a divider, then Copy
/// curve and Paste curve, named `copy` and `paste`. An item holding a [`CurveMenuReplace`]
/// replaces the curve when picked; the host observes its other items itself.
pub fn curve_menu(items: Option<Box<dyn SceneList>>) -> impl Scene {
    bsn! {
        Node {
            margin: UiRect::left(Val::Auto),
            column_gap: px(4),
            align_items: AlignItems::Center,
        }
        Children [
            @choice_menu(ChoiceGroup::Segment) #kind
            --
            @choice_menu(ChoiceGroup::Mode) #mode
            --
            @overflow_menu(items)
        ]
    }
}

/// The ⋯ menu of a [`curve_menu`].
fn overflow_menu(items: Option<Box<dyn SceneList>>) -> impl Scene {
    bsn! {
        @FeathersMenu
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
#[derive(Component, Clone, Debug, FromTemplate)]
pub struct CurveMenuTarget(pub Entity);

/// Which of a point's settings a [`curve_menu`]'s dropdown sets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ChoiceGroup {
    /// The kind of the segment after the point.
    #[default]
    Segment,
    /// The point's handle mode.
    Mode,
}

impl ChoiceGroup {
    /// Each choice's name, caption and icon, in the order of [`SEGMENTS`] or [`MODES`].
    fn choices(self) -> [(&'static str, &'static str, &'static str); 3] {
        match self {
            Self::Segment => [
                (
                    "constant",
                    "Constant",
                    "embedded://arbor_day/curve/icons/step.png",
                ),
                (
                    "linear",
                    "Linear",
                    "embedded://arbor_day/curve/icons/line.png",
                ),
                (
                    "cubic",
                    "Cubic",
                    "embedded://arbor_day/curve/icons/bezier.png",
                ),
            ],
            Self::Mode => [
                (
                    "auto",
                    "Auto",
                    "embedded://arbor_day/curve/icons/smooth.png",
                ),
                (
                    "aligned",
                    "Aligned",
                    "embedded://arbor_day/curve/icons/straight.png",
                ),
                ("free", "Free", "embedded://arbor_day/curve/icons/sharp.png"),
            ],
        }
    }

    /// Which choices the constraints allow.
    fn allowed(self, constraints: &CurveConstraints) -> [bool; 3] {
        match self {
            Self::Segment => SEGMENTS.map(|kind| constraints.segments.contains(&kind)),
            Self::Mode => MODES.map(|mode| constraints.handles.contains(&mode)),
        }
    }

    /// The choice point `index` of `curve` has.
    fn current(self, curve: &ScalarCurve, index: usize) -> Option<usize> {
        let point = curve.points.get(index)?;
        match self {
            Self::Segment => SEGMENTS.iter().position(|&kind| kind == point.segment),
            Self::Mode => MODES.iter().position(|&mode| mode == point.handles),
        }
    }

    /// The selected points this group acts on: for a segment kind, those with a segment after.
    fn points(self, selection: &CurveSelection, curve: Entity, count: usize) -> Vec<usize> {
        match self {
            Self::Segment => outgoing(selection, curve, count),
            Self::Mode => selection.indices(curve),
        }
    }

    /// The edit that sets choice `choice` on `points`.
    fn op(self, points: Vec<usize>, choice: usize) -> CurveEditOp {
        match self {
            Self::Segment => CurveEditOp::SetSegment(points, SEGMENTS[choice]),
            Self::Mode => CurveEditOp::SetHandleMode(points, MODES[choice]),
        }
    }
}

const SEGMENTS: [CurveSegment; 3] = [
    CurveSegment::Constant,
    CurveSegment::Linear,
    CurveSegment::Cubic,
];
const MODES: [HandleMode; 3] = [HandleMode::Auto, HandleMode::Aligned, HandleMode::Free];

/// On a [`curve_menu`]'s dropdown: the setting it chooses.
#[derive(Component, Clone, Copy, Debug, Default)]
struct ChoiceMenu(ChoiceGroup);

/// In a dropdown's button: the icon of one choice, or with none, the text shown while the
/// selected points differ or none is selected.
#[derive(Component, Clone, Copy, Debug, Default)]
struct ChoiceCaption(Option<usize>);

/// A dropdown's item for choice `0` of its group.
#[derive(Component, Clone, Copy, Debug, Default)]
struct ChoiceItem(usize);

/// A dropdown choosing among `group`'s choices, its button showing the selected points' choice, as
/// the color input's shows its swatch, or "—" while they differ; each item an icon and a name.
/// Hidden until [`show_choices`] finds the constraints offer a choice.
fn choice_menu(group: ChoiceGroup) -> impl Scene {
    let [a, b, c] = group.choices();
    bsn! {
        @FeathersMenu
        ChoiceMenu({group})
        Node { display: Display::None }
        Children [
            @FeathersMenuToolButton {
                @caption: bsn_list! {
                    @choice_icon(a.2) ChoiceCaption({Some(0)})
                    --
                    @choice_icon(b.2) ChoiceCaption({Some(1)})
                    --
                    @choice_icon(c.2) ChoiceCaption({Some(2)})
                    --
                    Text("—") ThemedText ChoiceCaption
                },
            }
            --
            @FeathersMenuPopup
            Children [
                @choice_item(0, a) -- @choice_item(1, b) -- @choice_item(2, c)
            ]
        ]
    }
}

fn choice_icon(path: &'static str) -> impl Scene {
    bsn! { @icon(path) Node { height: px(16), display: Display::None } }
}

/// The item for choice `index`, named, captioned and drawn by `choice`.
fn choice_item(
    index: usize,
    (name, caption, path): (&'static str, &'static str, &'static str),
) -> impl Scene {
    bsn! {
        @FeathersMenuItem {
            // Directly in the item, a row already, so its font reaches the text.
            @caption: bsn_list! {
                @icon(path) Node { height: px(16), margin: UiRect::right(px(8)) }
                --
                Text(caption) ThemedText
            }
        }
        ChoiceItem({index})
        Name({name})
    }
}

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

/// A number input for the selected point's x, or its `y`; with `handle`, for that handle's offset
/// from the point.
#[derive(Component, Clone, Copy, Debug, Default)]
struct CoordinateInput {
    handle: Option<HandleSide>,
    y: bool,
}

/// A row of [`CoordinateInput`]s for a handle, shown while the selected point shows that handle.
#[derive(Component, Clone, Copy, Debug, Default)]
struct HandleInputRow;

/// The row of [`CoordinateInput`]s for the selected point, shown while exactly one is selected;
/// or with `placeholder`, the row standing in for it otherwise, of the same height.
#[derive(Component, Clone, Copy, Debug, Default)]
struct PointRow {
    placeholder: bool,
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

/// The node holding the stems of the handles shown on the plot it is a child of, under the
/// markers.
#[derive(Component, Clone, Copy, Debug, Default)]
struct HandleStems;

/// The node holding the selected points' outgoing segments, drawn bright over the curve of the
/// plot it is a child of.
#[derive(Component, Clone, Copy, Debug, Default)]
struct SpanHighlights;

/// A path for point `index` in a [`PathLayer`]: a stem, or a segment drawn bright.
#[derive(Component, Clone, Copy, Debug)]
struct LayerPath {
    index: usize,
}

/// A node marking the handle on `side` of point `index` of the plot it is a child of.
#[derive(Component, Clone, Copy, Debug)]
struct HandleMarker {
    index: usize,
    side: HandleSide,
    hovered: bool,
}

impl HandleMarker {
    /// Radius, border width, fill and border: a ring in the selection's color, filled with the
    /// plot's background so the stem ends at its edge.
    fn style(hovered: bool) -> (f32, f32, ThemeToken, ThemeToken) {
        if hovered {
            (4.0, 1.5, tokens::WINDOW_BG, tokens::BUTTON_TEXT)
        } else {
            (4.0, 1.5, tokens::WINDOW_BG, SELECTION)
        }
    }
}

/// A marker's node: a circle of `radius` at `centre`, from its plot's top left.
fn marker_node(centre: Vec2, radius: f32, border: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(centre.x - radius),
        top: px(centre.y - radius),
        width: px(radius * 2.0),
        height: px(radius * 2.0),
        border: UiRect::all(px(border)),
        border_radius: BorderRadius::MAX,
        ..default()
    }
}

pub(super) fn plugin(app: &mut App) {
    embedded_asset!(app, "icons/step.png");
    embedded_asset!(app, "icons/line.png");
    embedded_asset!(app, "icons/bezier.png");
    embedded_asset!(app, "icons/smooth.png");
    embedded_asset!(app, "icons/straight.png");
    embedded_asset!(app, "icons/sharp.png");
    app.init_resource::<CurveClipboard>()
        .add_observer(edit_from_input)
        .add_observer(pick_menu_item)
        .add_observer(pick_choice)
        .add_systems(Update, (enable_paste, show_choices))
        .add_systems(
            PostUpdate,
            (
                place_markers,
                place_handles,
                place_spans,
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

/// Shows each dropdown while its editor's constraints offer a choice, and of its items only those
/// they allow. Its button shows the selected points' choice, or "—" while they differ or none is
/// selected, and is disabled while no selected point has the setting.
fn show_choices(
    menus: Query<(Entity, &ChoiceMenu)>,
    parents: Query<&ChildOf>,
    targets: Query<&CurveMenuTarget>,
    editors: Query<(&CurveValue, &CurveConstraints, &CurveSelection)>,
    children: Query<&Children>,
    mut nodes: Query<&mut Node>,
    captions: Query<&ChoiceCaption>,
    items: Query<&ChoiceItem>,
    buttons: Query<Has<InteractionDisabled>, With<FeathersMenuButton>>,
    mut commands: Commands,
) {
    for (menu, choice) in &menus {
        let Some(editor) = (parents.iter_ancestors(menu))
            .find_map(|ancestor| targets.get(ancestor).ok())
            .map(|target| target.0)
        else {
            continue;
        };
        let Ok((curve, constraints, selection)) = editors.get(editor) else {
            continue;
        };
        let group = choice.0;
        let allowed = group.allowed(constraints);
        let offered = allowed.iter().filter(|&&allowed| allowed).count() > 1;
        show(&mut nodes, menu, offered);
        if !offered {
            continue;
        }
        let points = group.points(selection, editor, curve.0.points.len());
        let mut current = points.iter().map(|&index| group.current(&curve.0, index));
        let shown = match current.next() {
            Some(first) if current.all(|other| other == first) => first,
            _ => None,
        };
        for descendant in children.iter_descendants(menu) {
            if let Ok(caption) = captions.get(descendant) {
                show(&mut nodes, descendant, caption.0 == shown);
            }
            if let Ok(item) = items.get(descendant) {
                show(&mut nodes, descendant, allowed[item.0]);
            }
            if let Ok(disabled) = buttons.get(descendant)
                && points.is_empty() != disabled
            {
                if disabled {
                    commands.entity(descendant).remove::<InteractionDisabled>();
                } else {
                    commands.entity(descendant).insert(InteractionDisabled);
                }
            }
        }
    }
}

/// Displays `entity`'s node if `shown`, else leaves it out of the layout.
fn show(nodes: &mut Query<&mut Node>, entity: Entity, shown: bool) {
    if let Ok(mut node) = nodes.get_mut(entity) {
        set_shown(&mut node, shown);
    }
}

/// Displays `node` if `shown`, else leaves it out of the layout.
fn set_shown(node: &mut Mut<Node>, shown: bool) {
    let display = if shown { Display::Flex } else { Display::None };
    if node.display != display {
        node.display = display;
    }
}

/// A dropdown's item, picked, sets its choice on the selected points, as one edit.
fn pick_choice(
    activate: On<Activate>,
    items: Query<&ChoiceItem>,
    menus: Query<&ChoiceMenu>,
    parents: Query<&ChildOf>,
    targets: Query<&CurveMenuTarget>,
    editors: Query<(&CurveValue, &CurveSelection)>,
    mut commands: Commands,
) {
    let Ok(item) = items.get(activate.entity) else {
        return;
    };
    let mut ancestors = parents.iter_ancestors(activate.entity);
    let Some(group) = (ancestors.by_ref())
        .find_map(|ancestor| menus.get(ancestor).ok())
        .map(|menu| menu.0)
    else {
        return;
    };
    let Some(editor) = ancestors
        .find_map(|ancestor| targets.get(ancestor).ok())
        .map(|target| target.0)
    else {
        return;
    };
    let Ok((curve, selection)) = editors.get(editor) else {
        return;
    };
    let points = group.points(selection, editor, curve.0.points.len());
    if points.is_empty() {
        return;
    }
    commands.trigger(CurveEdit {
        editor,
        curve: editor,
        op: group.op(points, item.0),
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
            } else if hover.0 == Some(CurveTarget::Point(point_ref)) {
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
                marker_node(centre, radius, border),
                ThemeBackgroundColor(fill),
                ThemeBorderColor(edge),
                Pickable::IGNORE,
                ChildOf(plot),
            ));
        }
    }
}

/// Keeps a marker per shown handle on each editor's plot, named `in<index>` or `out<index>`, and a
/// stem per selected point with any, from its In handle through the point to its Out handle.
fn place_handles(
    plots: Query<(Entity, &CurvePlot, &ComputedNode, &CurveHover, &Children), With<CurveGestures>>,
    parents: Query<&ChildOf>,
    curves: Query<(&CurveValue, &CurveConstraints, &CurveSelection)>,
    layers: Query<Option<&Children>, With<HandleStems>>,
    mut markers: Query<(&mut HandleMarker, &mut Node)>,
    mut stems: Query<(&LayerPath, &mut CurvePath)>,
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
        let handles = shown_handles(&curve.0, &selection.indices(curve_entity));

        let mut placed = vec![false; handles.len()];
        for child in children {
            let Ok((mut marker, mut placement)) = markers.get_mut(*child) else {
                continue;
            };
            let Some(at) = (handles.iter())
                .position(|&(index, side, _)| index == marker.index && side == marker.side)
            else {
                commands.entity(*child).despawn();
                continue;
            };
            placed[at] = true;
            let point = PointRef {
                curve: curve_entity,
                index: marker.index,
            };
            let hovered = hover.0 == Some(CurveTarget::Handle(point, marker.side));
            let (radius, border, fill, edge) = HandleMarker::style(hovered);
            placement.set_if_neq(marker_node(view.to_pixels(handles[at].2), radius, border));
            if marker.hovered != hovered {
                marker.hovered = hovered;
                commands
                    .entity(*child)
                    .insert((ThemeBackgroundColor(fill), ThemeBorderColor(edge)));
            }
        }
        for (&(index, side, at), _) in handles.iter().zip(&placed).filter(|(_, placed)| !**placed) {
            let (radius, border, fill, edge) = HandleMarker::style(false);
            let name = match side {
                HandleSide::Left => format!("in{index}"),
                HandleSide::Right => format!("out{index}"),
            };
            commands.spawn((
                HandleMarker {
                    index,
                    side,
                    hovered: false,
                },
                Name::new(name),
                marker_node(view.to_pixels(at), radius, border),
                ThemeBackgroundColor(fill),
                ThemeBorderColor(edge),
                Pickable::IGNORE,
                ChildOf(plot),
            ));
        }

        // Each stem in pixels, by point.
        let mut paths: Vec<(usize, Vec<Vec2>)> = Vec::new();
        for &(index, side, at) in &handles {
            let point = &curve.0.points[index];
            let centre = view.to_pixels(Vec2::new(point.x, point.y));
            let handle = view.to_pixels(at);
            match paths.last_mut() {
                Some((last, path)) if *last == index => path.push(handle),
                _ if side == HandleSide::Left => paths.push((index, vec![handle, centre])),
                _ => paths.push((index, vec![centre, handle])),
            }
        }
        if let Some((layer, layer_children)) =
            (children.iter()).find_map(|child| Some((child, layers.get(child).ok()?)))
        {
            let stroke = CurveStroke {
                color: SELECTION,
                width: 1.0,
            };
            let layer = PathLayer {
                entity: layer,
                children: layer_children,
                prefix: "stem",
                stroke,
            };
            layer.sync(paths, &mut stems, &mut commands);
        }
    }
}

/// While the plot offers a choice of segment kind, draws each selected point's outgoing segment
/// again over the curve, bright and wider, named `span<index>`, so it is plain which segments a
/// kind would change.
fn place_spans(
    plots: Query<(Entity, &CurvePlot, &ComputedNode, &Children), With<CurveGestures>>,
    parents: Query<&ChildOf>,
    curves: Query<(&CurveValue, &CurveConstraints, &CurveSelection)>,
    layers: Query<Option<&Children>, With<SpanHighlights>>,
    mut spans: Query<(&LayerPath, &mut CurvePath)>,
    mut commands: Commands,
) {
    for (plot, settings, node, children) in &plots {
        let owner = super::view::curve_owner(plot, &parents, |owner| curves.contains(owner));
        let Some((curve_entity, (curve, constraints, selection))) =
            owner.and_then(|owner| Some((owner, curves.get(owner).ok()?)))
        else {
            continue;
        };
        let Some((layer, layer_children)) =
            (children.iter()).find_map(|child| Some((child, layers.get(child).ok()?)))
        else {
            continue;
        };
        let mut paths = Vec::new();
        if constraints.segments.len() > 1 {
            let size = node.size() * node.inverse_scale_factor();
            let view = CurveView::new(constraints, size, settings.inset);
            let prepared = curve.0.prepare();
            let points = &curve.0.points;
            for index in outgoing(selection, curve_entity, points.len()) {
                let path = match prepared.segment_from(index) {
                    Some(segment) => flatten_segment(segment, &view),
                    // A step: its two points at one x.
                    None => [index, index + 1]
                        .map(|at| view.to_pixels(Vec2::new(points[at].x, points[at].y)))
                        .to_vec(),
                };
                paths.push((index, path));
            }
        }
        let stroke = CurveStroke {
            color: tokens::BUTTON_TEXT,
            width: 2.5,
        };
        let layer = PathLayer {
            entity: layer,
            children: layer_children,
            prefix: "span",
            stroke,
        };
        layer.sync(paths, &mut spans, &mut commands);
    }
}

/// The selected points of `curve` with a segment after them, of `count` points.
fn outgoing(selection: &CurveSelection, curve: Entity, count: usize) -> Vec<usize> {
    (selection.indices(curve).into_iter())
        .filter(|&index| index + 1 < count)
        .collect()
}

/// A node of stroked paths, one per point, over a plot.
struct PathLayer<'a> {
    entity: Entity,
    children: Option<&'a Children>,
    /// Each path is named this and its point's index.
    prefix: &'static str,
    stroke: CurveStroke,
}

impl PathLayer<'_> {
    /// Keeps a child per entry of `paths`, by point, each filling the layer.
    fn sync(
        &self,
        paths: Vec<(usize, Vec<Vec2>)>,
        nodes: &mut Query<(&LayerPath, &mut CurvePath)>,
        commands: &mut Commands,
    ) {
        let mut drawn = vec![false; paths.len()];
        for child in self.children.into_iter().flatten() {
            let Ok((node, mut path)) = nodes.get_mut(*child) else {
                continue;
            };
            match paths.iter().position(|(index, _)| *index == node.index) {
                Some(at) => {
                    drawn[at] = true;
                    path.set_if_neq(CurvePath(paths[at].1.clone()));
                }
                None => commands.entity(*child).despawn(),
            }
        }
        for ((index, path), _) in paths.into_iter().zip(drawn).filter(|(_, drawn)| !drawn) {
            commands.spawn((
                LayerPath { index },
                Name::new(format!("{}{index}", self.prefix)),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                CurvePath(path),
                self.stroke.clone(),
                Pickable::IGNORE,
                ChildOf(self.entity),
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
        (Entity, &CurveConstraints, &FeathersCurveEditor),
        Or<(Changed<CurveConstraints>, Changed<FeathersCurveEditor>)>,
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

/// Shows the selected point's x and y in the inputs, enabled only for a coordinate the constraints
/// let it move in; with none or several selected, a placeholder row stands in for them. A handle's
/// inputs show its offset, their row only while the point shows that handle. Each input rounds to
/// its axis's precision, written in the input's units.
fn show_selected_point(
    editors: Query<
        (
            Entity,
            &CurveValue,
            Ref<CurveConstraints>,
            &CurveSelection,
            Ref<FeathersCurveEditor>,
        ),
        Or<(
            Changed<CurveValue>,
            Changed<CurveConstraints>,
            Changed<CurveSelection>,
            Changed<FeathersCurveEditor>,
        )>,
    >,
    children: Query<&Children>,
    inputs: Query<(
        &CoordinateInput,
        &NumericValue,
        Has<InteractionDisabled>,
        &ChildOf,
    )>,
    mut rows: Query<&mut Node, With<HandleInputRow>>,
    mut point_rows: Query<(&mut Node, &PointRow), Without<HandleInputRow>>,
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
            if let Ok((mut node, row)) = point_rows.get_mut(descendant) {
                set_shown(&mut node, point.is_some() != row.placeholder);
                continue;
            }
            let Ok((input, shown, disabled, row)) = inputs.get(descendant) else {
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
                let units = registry.resolve(units.as_ref());
                if axis.precision > 0.0 {
                    let shown = units.canonical_to_display(axis.precision as f64);
                    entity.insert(NumberInputPrecision(-shown.log10().round() as i32));
                }
                if axis.step > 0.0 {
                    entity.insert(NumberInputStep(
                        units.canonical_to_display(axis.step as f64),
                    ));
                }
                // A handle's offset has bounds of its own, which `apply` keeps.
                if axis.clamped && input.handle.is_none() {
                    entity.insert(HardLimit(NumericRange::F32(axis.min..=axis.max)));
                }
            }
            // What the input shows, and whether it may be edited; with no single point, its row is
            // hidden.
            let (movable, at) = match input.handle {
                Some(side) => {
                    let offset = point.and_then(|(index, _)| curve.0.manual_handle(index, side));
                    if let Ok(mut row) = rows.get_mut(row.parent()) {
                        set_shown(&mut row, offset.is_some());
                    }
                    (offset.is_some(), offset.unwrap_or_default())
                }
                None => {
                    let movable = point.is_some_and(|(index, _)| {
                        let end = index == 0 || index == last;
                        match constraints.pin_ends {
                            PinEnds::None => true,
                            PinEnds::X => input.y || !end,
                            PinEnds::XY => !end,
                        }
                    });
                    (movable, point.map_or(Vec2::ZERO, |(_, at)| at))
                }
            };
            if movable == disabled {
                if movable {
                    entity.remove::<InteractionDisabled>();
                    entity.queue(restyle_number_input);
                } else {
                    entity.insert(InteractionDisabled);
                }
            }
            let value = NumericValue::F32(if input.y { at.y } else { at.x });
            if *shown != value || configure {
                entity.insert(value);
            }
        }
    }
}

/// Restyles a number input just enabled. Feathers restyles it as the disabled marker is removed,
/// while the marker is still there, so it stays drawn disabled (X67); a hover change restyles it
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

/// A value typed, scrubbed or stepped into a point input proposes the selected point there, or
/// into a handle's input, the handle at that offset, rounded to its axis's precision: feathers'
/// chevrons step without rounding.
fn edit_from_input(
    change: On<ValueChange<f32>>,
    inputs: Query<&CoordinateInput>,
    parents: Query<&ChildOf>,
    editors: Query<(&CurveValue, &CurveConstraints, &CurveSelection), With<FeathersCurveEditor>>,
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
    let Ok((curve, constraints, selection)) = editors.get(editor) else {
        return;
    };
    let Some(point) = selection.single() else {
        return;
    };
    let at = match input.handle {
        Some(side) => curve.0.manual_handle(point.index, side),
        None => (curve.0.points.get(point.index)).map(|at| Vec2::new(at.x, at.y)),
    };
    let Some(at) = at else {
        return;
    };
    let at = if input.y {
        Vec2::new(at.x, constraints.y.round(change.value))
    } else {
        Vec2::new(constraints.x.round(change.value), at.y)
    };
    let op = match input.handle {
        Some(side) => CurveEditOp::MoveHandle(point.index, side, at),
        None => CurveEditOp::SetPoint(point.index, at),
    };
    commands.trigger(CurveEdit {
        editor,
        curve: point.curve,
        op,
        phase: if change.is_final {
            CurveEditPhase::Finish
        } else {
            CurveEditPhase::Update
        },
    });
}
