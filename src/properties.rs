//! The property editor: panel rows for the fields of a reflected struct, one row per field, each
//! named after its field. Booleans get a checkbox, numbers a number input, fuzzy numbers a center,
//! a spread and a shape toggle, colors a swatch that opens a picker, enums of unit variants a
//! select, nested structs a subpane of rows of their own, an array with a [`ShownCount`] a subpane
//! per shown item, and every other field a label giving its name and type. A row's caption sits
//! above its inputs, so they get the panel's full width. Field attributes (below) set a row's
//! caption and a number's limits, precision, step and units.

use std::{collections::HashMap, ops::RangeInclusive};

use bevy::{
    feathers::{
        containers::{subpane, subpane_body, subpane_header},
        controls::{
            ButtonVariant, ColorInputValue, FeathersCheckbox, FeathersColorInput, FeathersListRow,
            FeathersNumberInput, FeathersSelect, FeathersToolButton, HardLimit,
            NumberInputPrecision, NumberInputStep, NumberInputUnits, OptionIndex, SoftLimit,
            UnitsFormat, UnitsRegistry,
        },
        display::{caption as option_caption, icon, label, label_small},
        theme::ThemedText,
    },
    prelude::*,
    reflect::{
        FromReflect, GetTypeRegistration, ReflectMut, ReflectRef, TypeInfo, Typed,
        array::Array,
        attributes::CustomAttributes,
        enums::{DynamicEnum, DynamicVariant, VariantInfo},
    },
    ui::Selected,
    ui_widgets::{NumericFormat, NumericRange, NumericValue, SetSelected, ValueChange},
};
use bevy_immediate::{
    CapSet, Imm, ImmCapAccessRequests, ImmCapability, ImmEntity, impl_capability_set,
    ui::{
        ImplCapsUiFeathers,
        activated::{self, ImmUiActivated},
        anchored, base, button_variant,
        checked::{self, ImmUiChecked},
        clicked, color_plane, color_swatch,
        disabled::{self, ImmUiInteractionsDisabled},
        interaction, layout_order, look,
        number_input::{self, ImmUiNumberInput, ImmUiNumericValue},
        selected, slider_base_color, slider_value,
        text::{self, ImmUiText},
        text_input,
        track_value_change_plugin::{NewValueChange, TrackValueChangePlugin},
    },
};

use crate::{
    fuzzy::{Distribution, Fuzzy},
    random::Seed,
    spread_strip::{StripRange, spread_strip},
};

/// The panel's capabilities: `bevy_immediate`'s for feathers, and reading back an enum's select.
pub struct CapsPanel;

impl_capability_set!(
    CapsPanel,
    ImplCapsPanel > ImplCapsUiFeathers,
    (
        base::CapabilityUiBase,
        layout_order::CapabilityUiLayoutOrder,
        look::CapabilityUiLook,
        disabled::CapabilityUiDisabled,
        interaction::CapabilityUiInteraction,
        text::CapabilityUiText,
        text_input::CapabilityUiTextInput,
        selected::CapabilityUiSelectable,
        checked::CapabilityUiChecked,
        clicked::CapabilityUiClicked,
        activated::CapabilityUiActivated,
        anchored::CapabilityUiAnchored,
        slider_value::CapabilityUiSliderValue,
        number_input::CapabilityUiNumberInput,
        slider_base_color::CapabilityUiSliderBaseColor,
        button_variant::CapabilityUiFeathersButtonVariant,
        color_swatch::CapabilityUiColorSwatch,
        color_plane::CapabilityUiColorPlane,
        CapabilityEnumSelect,
        CapabilityColorInput,
    )
);

/// Lets the panel bind a feathers color input: read the color it shows, and take the color the
/// user last picked.
pub struct CapabilityColorInput;

impl ImmCapability for CapabilityColorInput {
    fn build<Caps: CapSet>(app: &mut App, cap_req: &mut ImmCapAccessRequests<Caps>) {
        cap_req.request_component_read::<ColorInputValue>(app.world_mut());
        cap_req.request_component_write::<NewValueChange<Color>>(app.world_mut());
        if !app.is_plugin_added::<TrackValueChangePlugin<Color>>() {
            app.add_plugins(TrackValueChangePlugin::<Color>::default());
        }
    }
}

/// Lets the panel read and clear an enum select's [`PickedVariant`].
pub struct CapabilityEnumSelect;

impl ImmCapability for CapabilityEnumSelect {
    fn build<Caps: CapSet>(app: &mut App, cap_req: &mut ImmCapAccessRequests<Caps>) {
        cap_req.request_component_write::<PickedVariant>(app.world_mut());
    }
}

/// On an enum's select: the variant the user last picked, until the panel applies it, and the
/// variant the select shows.
#[derive(Component)]
struct PickedVariant {
    picked: Option<usize>,
    shown: usize,
}

/// The inclusive range a number may take. Dragging spans the same range, so the input looks and
/// drags like a slider, unless the field is also marked [`Spinbox`].
#[derive(Debug, Clone, Reflect)]
pub struct ValueRange<T>(pub RangeInclusive<T>);

/// Decimal places a number is rounded to while dragging.
#[derive(Debug, Clone, Reflect)]
pub struct Precision(pub i32);

/// How far one step of the input's arrows moves a number.
#[derive(Debug, Clone, Reflect)]
pub struct Step<T>(pub T);

/// The row's label, in place of the field's name.
#[derive(Debug, Clone, Reflect)]
pub struct Caption(pub &'static str);

/// A number in a [`ValueRange`] shown without the slider fill, as a spinbox with step arrows.
#[derive(Debug, Clone, Reflect)]
pub struct Spinbox;

/// On an array of structs: the sibling `i32` field holding how many of its items are shown, from
/// the first. Each shown item is a subpane, captioned with the array's caption and its number,
/// beside the struct's other rows. The last one's header has − and + buttons that change the
/// count, within the count field's [`ValueRange`] and the array's length, and never below 1. The
/// count field gets no row of its own.
#[derive(Debug, Clone, Reflect)]
pub struct ShownCount(pub &'static str);

/// Registers the editor's units with feathers.
pub struct PropertiesPlugin;

impl Plugin for PropertiesPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .resource_mut::<UnitsRegistry>()
            .insert(&RatioPercent);
    }
}

/// Units for a fraction shown as a percentage, `ratio_percent`: 0.4 shows as 40%. Typed text is
/// read as a percentage, with or without the sign.
pub struct RatioPercent;

impl UnitsFormat for RatioPercent {
    fn id(&self) -> &'static str {
        "ratio_percent"
    }

    fn format(&self, value: NumericValue, _editing: bool) -> String {
        // From the value's shortest decimal, so 0.3 shows as 30%: 0.3 as an f32 times 100 is
        // nearer 30.000002 than 30.
        let decimal = value.to_string().parse::<f64>().unwrap_or_default() * 100.0;
        match value {
            NumericValue::F32(_) => format!("{}%", decimal as f32),
            _ => format!("{decimal}%"),
        }
    }

    fn parse(&self, value: String, fmt: NumericFormat) -> Result<NumericValue, String> {
        let digits = value.trim();
        let digits = digits.strip_suffix('%').unwrap_or(digits).trim_end();
        NumericValue::parse_from(digits, fmt).map(|percent| percent.scale_by(0.01))
    }
}

/// A number type the editor shows in a number input.
trait NumberField: ImmUiNumericValue + FromReflect + Typed + GetTypeRegistration + TypePath {
    fn range(range: RangeInclusive<Self>) -> NumericRange;
    fn to_step(step: Self) -> f64;
}

impl NumberField for f32 {
    fn range(range: RangeInclusive<Self>) -> NumericRange {
        NumericRange::F32(range)
    }
    fn to_step(step: Self) -> f64 {
        step as f64
    }
}

impl NumberField for f64 {
    fn range(range: RangeInclusive<Self>) -> NumericRange {
        NumericRange::F64(range)
    }
    fn to_step(step: Self) -> f64 {
        step
    }
}

impl NumberField for i32 {
    fn range(range: RangeInclusive<Self>) -> NumericRange {
        NumericRange::I32(range)
    }
    fn to_step(step: Self) -> f64 {
        step as f64
    }
}

impl NumberField for i64 {
    fn range(range: RangeInclusive<Self>) -> NumericRange {
        NumericRange::I64(range)
    }
    fn to_step(step: Self) -> f64 {
        step as f64
    }
}

/// Text shown in parentheses after an item's caption in its subpane header ("Limbs 1 (8)"), keyed
/// by the item's name (`limbs[0]`): figures about the item that the panel's owner knows and the
/// item does not.
pub type HeaderNotes = HashMap<String, String>;

/// Adds a row for each field of `value`, which must be a struct, editing the fields in place.
pub fn edit_fields(ui: &mut Imm<CapsPanel>, value: &mut dyn PartialReflect, notes: &HeaderNotes) {
    // Attributes belong to the type, so read them from its info, not the value.
    let Some(TypeInfo::Struct(info)) = value.get_represented_type_info() else {
        return;
    };
    let ReflectMut::Struct(fields) = value.reflect_mut() else {
        return;
    };
    // The count fields named by `ShownCount`, with their values and ranges.
    let counts: Vec<(&'static str, i32, RangeInclusive<i32>)> = info
        .iter()
        .filter_map(|field| field.custom_attributes().get::<ShownCount>())
        .filter_map(|shown| {
            let value = fields.field(shown.0)?.try_downcast_ref::<i32>()?;
            let range = info
                .field(shown.0)?
                .custom_attributes()
                .get::<ValueRange<i32>>()
                .map_or(1..=i32::MAX, |range| range.0.clone());
            Some((shown.0, *value, range))
        })
        .collect();
    let mut new_counts = Vec::new();
    for index in 0..fields.field_len() {
        let (Some(field_info), Some(field)) = (info.field_at(index), fields.field_at_mut(index))
        else {
            continue;
        };
        let name = field_info.name();
        let attrs = field_info.custom_attributes();
        let caption = attrs.get::<Caption>().map_or(name, |caption| caption.0);
        if counts.iter().any(|(count, ..)| *count == name) {
            continue;
        }
        if let Some(shown) = attrs.get::<ShownCount>()
            && let Some((_, count, range)) = counts.iter().find(|(count, ..)| *count == shown.0)
            && let ReflectMut::Array(items) = field.reflect_mut()
        {
            let new_count = shown_items(ui, name, caption, items, *count, range.clone(), notes);
            if new_count != *count {
                new_counts.push((shown.0, new_count));
            }
            continue;
        }
        // Rows are keyed by field name, so they keep their entities from frame to frame.
        let row = ui.ch_id(name);

        if let Some(flag) = field.try_downcast_mut::<bool>() {
            row.on_spawn_apply_scene(move || {
                bsn! { @FeathersCheckbox { @caption: bsn! { Text(caption) ThemedText } } }
            })
            .on_spawn_insert(move || Name::new(name))
            .checked(flag);
        } else if let Some(fuzzy) = field.try_downcast_mut::<Fuzzy>() {
            fuzzy_row(row, name, caption, attrs, fuzzy);
        } else if let Some(seed) = field.try_downcast_mut::<Seed>() {
            seed_row(row, name, caption, seed);
        } else if let Some(number) = field.try_downcast_mut::<f32>() {
            number_row(row, name, caption, attrs, number);
        } else if let Some(number) = field.try_downcast_mut::<f64>() {
            number_row(row, name, caption, attrs, number);
        } else if let Some(number) = field.try_downcast_mut::<i32>() {
            number_row(row, name, caption, attrs, number);
        } else if let Some(number) = field.try_downcast_mut::<i64>() {
            number_row(row, name, caption, attrs, number);
        } else if let Some(color) = field.try_downcast_mut::<Color>() {
            color_row(row, name, caption, color);
        } else if let Some(variants) = unit_variants(field) {
            enum_row(row, name, caption, variants, field);
        } else if let Some(TypeInfo::Struct(_)) = field.get_represented_type_info() {
            group(
                row.on_spawn_insert(move || Name::new(name)),
                caption,
                |ui| {
                    edit_fields(ui, field, notes);
                },
            );
        } else {
            let text = format!("{caption}: {}", field.reflect_short_type_path());
            row.on_spawn_apply_scene(move || label(text))
                .on_spawn_insert(move || Name::new(name));
        }
    }
    for (count, value) in new_counts {
        if let Some(field) = fields.field_mut(count)
            && let Some(field) = field.try_downcast_mut::<i32>()
        {
            *field = value;
        }
    }
}

/// The first `count` of `items`, a subpane each named `name[index]` and headed with its note, if
/// any, the last with − and + buttons, named `remove` and `add`. Returns the count as the buttons
/// leave it.
fn shown_items(
    ui: &mut Imm<CapsPanel>,
    name: &'static str,
    caption: &'static str,
    items: &mut dyn Array,
    count: i32,
    range: RangeInclusive<i32>,
    notes: &HeaderNotes,
) -> i32 {
    // Never none: with no subpane there would be no + to bring one back.
    let min = (*range.start()).max(1);
    let max = (*range.end()).min(items.len() as i32).max(min);
    let count = count.clamp(min, max);
    let mut new_count = count;
    for index in 0..count as usize {
        let Some(item) = items.get_mut(index) else {
            continue;
        };
        let item_name = format!("{name}[{index}]");
        let header = match notes.get(&item_name) {
            Some(note) => format!("{caption} {} ({note})", index + 1),
            None => format!("{caption} {}", index + 1),
        };
        let pane = ui
            .ch_id((name, index))
            .on_spawn_insert(move || Name::new(item_name.clone()));
        let last = index + 1 == count as usize;
        let tools = |ui: &mut Imm<CapsPanel>| {
            if last {
                let removed = ui
                    .ch()
                    .on_spawn_apply_scene(|| tool_button("−", "remove"))
                    .interactions_disabled(count <= min)
                    .activated();
                let added = ui
                    .ch()
                    .on_spawn_apply_scene(|| tool_button("+", "add"))
                    .interactions_disabled(count >= max)
                    .activated();
                new_count = (count - removed as i32 + added as i32).clamp(min, max);
            }
        };
        group_with_tools(pane, header, tools, |ui| {
            edit_fields(ui, item, notes);
        });
    }
    new_count
}

/// A feathers tool button showing `caption`, named `name`.
fn tool_button(caption: &'static str, name: &'static str) -> impl Scene {
    bsn! {
        @FeathersToolButton { @caption: bsn! { Text(caption) ThemedText } }
        Name({name})
    }
}

/// A feathers subpane headed `caption`, its body built by `body`.
pub fn group<'w, 's>(
    pane: ImmEntity<'_, 'w, 's, CapsPanel>,
    caption: &'static str,
    body: impl FnOnce(&mut Imm<'w, 's, CapsPanel>),
) {
    group_with_tools(pane, caption.to_string(), |_| {}, body);
}

/// A feathers subpane headed `caption`, with buttons built by `tools` at the header's right end,
/// its body built by `body`. The caption follows `caption` as it changes.
fn group_with_tools<'w, 's>(
    pane: ImmEntity<'_, 'w, 's, CapsPanel>,
    caption: String,
    tools: impl FnOnce(&mut Imm<'w, 's, CapsPanel>),
    body: impl FnOnce(&mut Imm<'w, 's, CapsPanel>),
) {
    pane.on_spawn_apply_scene(subpane).add(|ui| {
        ui.ch().on_spawn_apply_scene(subpane_header).add(|ui| {
            ui.ch()
                .on_spawn_apply_scene(|| bsn! { Text("") ThemedText })
                .on_hash_change_text_fn(&caption, || caption.clone());
            ui.ch().on_spawn_insert(widget_line).add(tools);
        });
        ui.ch().on_spawn_apply_scene(subpane_body).add(body);
    });
}

/// How a number input limits and shows its number, read from a field's attributes.
#[derive(Clone)]
struct NumberLook {
    range: Option<NumericRange>,
    /// The fill bar dragged like a slider, rather than a spinbox.
    slider: bool,
    precision: Option<i32>,
    step: Option<f64>,
    units: Option<NumberInputUnits>,
}

impl NumberLook {
    fn from_attributes<T: NumberField>(attrs: &CustomAttributes) -> Self {
        let range = attrs
            .get::<ValueRange<T>>()
            .map(|range| T::range(range.0.clone()));
        Self {
            slider: range.is_some() && attrs.get::<Spinbox>().is_none(),
            range,
            precision: attrs.get::<Precision>().map(|precision| precision.0),
            step: attrs.get::<Step<T>>().map(|step| T::to_step(step.0)),
            units: attrs.get::<NumberInputUnits>().cloned(),
        }
    }
}

/// A row node named after its field, its caption stacked above its widgets.
fn row_node(name: &'static str) -> (Name, Node) {
    (
        Name::new(name),
        Node {
            flex_direction: FlexDirection::Column,
            // Half the gap between rows, so a caption reads as its own field's.
            row_gap: px(2),
            ..default()
        },
    )
}

/// The caption at the top of a row, in feathers' small label for field captions.
fn row_caption(ui: &mut Imm<CapsPanel>, caption: &'static str) {
    ui.ch().on_spawn_apply_scene(move || label_small(caption));
}

/// Widgets laid out left to right under a row's caption.
fn widget_line() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        // As wide as the gap between rows.
        column_gap: px(4),
        ..default()
    }
}

/// A caption and a number input, the input named `input` so plans can read its value.
fn number_row<T: NumberField>(
    row: ImmEntity<CapsPanel>,
    name: &'static str,
    caption: &'static str,
    attrs: &CustomAttributes,
    value: &mut T,
) {
    let look = NumberLook::from_attributes::<T>(attrs);
    row.on_spawn_insert(move || row_node(name)).add(|ui| {
        row_caption(ui, caption);
        number_input(ui.ch(), "input", look, percent(100), value);
    });
}

/// A number input named `name`, `width` wide, or taking a widget line's spare width if
/// `Val::Auto`. Returns the input, for adding children of its own.
fn number_input<'r, 'w, 's, T: NumberField>(
    input: ImmEntity<'r, 'w, 's, CapsPanel>,
    name: &'static str,
    look: NumberLook,
    width: Val,
    value: &mut T,
) -> ImmEntity<'r, 'w, 's, CapsPanel> {
    let grow: f32 = if width == Val::Auto { 1.0 } else { 0.0 };
    let initial = value.to_number_input_value();
    // The input writes its text from the components present when its text field spawns, so the
    // value, limits and units go in before the scene. Binding on the same frame would write the
    // text a second time while the first write is still queued, and both would show ("11" for
    // 1); so the binding starts the frame after.
    let input = input
        .on_spawn_apply_commands(move |commands| {
            commands.insert((Name::new(name), initial));
            if let Some(range) = look.range {
                if look.slider {
                    commands.insert(SoftLimit(range.clone()));
                }
                commands.insert(HardLimit(range));
            }
            if let Some(precision) = look.precision {
                commands.insert(NumberInputPrecision(precision));
            }
            if let Some(step) = look.step {
                commands.insert(NumberInputStep(step));
            }
            if let Some(units) = look.units {
                commands.insert(units);
            }
        })
        .on_spawn_apply_scene(
            move || bsn! { @FeathersNumberInput Node { flex_grow: {grow}, width: {width} } },
        );
    if input.will_be_spawned() {
        input
    } else {
        input.number_input(value)
    }
}

/// A seed's row: a number input named `input`, without the slider's fill, since nearby seeds are
/// no more alike than distant ones, and a button named `randomize` that picks a seed at random.
fn seed_row(row: ImmEntity<CapsPanel>, name: &'static str, caption: &'static str, seed: &mut Seed) {
    let look = NumberLook {
        range: Some(NumericRange::I32(0..=Seed::MAX)),
        slider: false,
        precision: None,
        step: None,
        units: None,
    };
    row.on_spawn_insert(move || row_node(name)).add(|ui| {
        row_caption(ui, caption);
        ui.ch().on_spawn_insert(widget_line).add(|ui| {
            number_input(ui.ch(), "input", look, Val::Auto, &mut seed.0);
            let mut button = ui.ch().on_spawn_apply_scene(|| {
                bsn! {
                    @FeathersToolButton { @variant: {ButtonVariant::Plain} }
                    #randomize
                }
            });
            if button.activated() {
                *seed = Seed::random();
            }
            button.add(|ui| {
                ui.ch().on_spawn_apply_scene(
                    || bsn! { @icon("icons/ifl.png") Node { height: px(18) } },
                );
            });
        });
    });
}

/// A fuzzy number's row: its center as a slider, its spread as a spinbox, and a button that
/// toggles the distribution's shape, showing the current one. The field's [`ValueRange`] bounds
/// the center; the spread goes up to half of it, so a spread can reach across the whole range.
/// With a range, the center input also shows the spread strip.
fn fuzzy_row(
    row: ImmEntity<CapsPanel>,
    name: &'static str,
    caption: &'static str,
    attrs: &CustomAttributes,
    fuzzy: &mut Fuzzy,
) {
    let center_look = NumberLook::from_attributes::<f32>(attrs);
    let strip_range = match &center_look.range {
        Some(NumericRange::F32(range)) => Some(StripRange {
            start: *range.start(),
            end: *range.end(),
        }),
        _ => None,
    };
    let half_range = strip_range.map_or(f32::MAX, |range| (range.end - range.start) * 0.5);
    let spread_look = NumberLook {
        range: Some(NumericRange::F32(0.0..=half_range)),
        slider: false,
        ..center_look.clone()
    };
    row.on_spawn_insert(move || row_node(name)).add(|ui| {
        row_caption(ui, caption);
        ui.ch().on_spawn_insert(widget_line).add(|ui| {
            let center = number_input(ui.ch(), "center", center_look, Val::Auto, &mut fuzzy.center);
            // Drawn from this frame's center and last frame's spread and shape, a frame behind
            // when they change.
            if let Some(range) = strip_range {
                let drawn = *fuzzy;
                center.add(|ui| spread_strip(ui, drawn, range));
            }
            number_input(ui.ch(), "spread", spread_look, px(56), &mut fuzzy.spread);
            let mut button = ui.ch().on_spawn_apply_scene(|| {
                bsn! {
                    @FeathersToolButton { @variant: {ButtonVariant::Plain} }
                    #shape
                }
            });
            if button.activated() {
                fuzzy.shape = match fuzzy.shape {
                    Distribution::Uniform => Distribution::Bell,
                    Distribution::Bell => Distribution::Uniform,
                };
            }
            let shape = fuzzy.shape;
            button.add(|ui| {
                // Keyed by shape, so a toggle replaces the icon.
                ui.ch_id(shape).on_spawn_apply_scene(move || {
                    let image = match shape {
                        Distribution::Uniform => "icons/uniform.png",
                        Distribution::Bell => "icons/bell.png",
                    };
                    bsn! { @icon(image) Node { height: px(18) } }
                });
            });
        });
    });
}

/// A caption with a feathers color input, a swatch that opens a picker, at the row's right end.
/// The input is named `input`.
fn color_row(
    row: ImmEntity<CapsPanel>,
    name: &'static str,
    caption: &'static str,
    color: &mut Color,
) {
    let line = move || Node {
        justify_content: JustifyContent::SpaceBetween,
        ..widget_line()
    };
    row.on_spawn_insert(move || (Name::new(name), line()))
        .add(|ui| {
            ui.ch().on_spawn_apply_scene(move || label(caption));
            let initial = *color;
            let mut input = ui
                .ch()
                .on_spawn_apply_scene(|| bsn! { #input @FeathersColorInput })
                .on_spawn_insert(move || {
                    (ColorInputValue(initial), NewValueChange::<Color>::default())
                });
            if !input.will_be_spawned() {
                if let Ok(Some(mut picked)) = input.cap_get_component_mut::<NewValueChange<Color>>()
                    && let Some(value) = NewValueChange::take(&mut picked)
                {
                    *color = value;
                }
                // Shows a color changed elsewhere, or just picked: the input does not update itself.
                let shown = input
                    .cap_get_component::<ColorInputValue>()
                    .ok()
                    .flatten()
                    .map(|value| value.0);
                if shown != Some(*color) {
                    input.entity_commands().insert(ColorInputValue(*color));
                }
            }
        });
}

/// The variant names of `value`'s type, if it is an enum whose variants are all units.
fn unit_variants(value: &dyn PartialReflect) -> Option<Vec<&'static str>> {
    let Some(TypeInfo::Enum(info)) = value.get_represented_type_info() else {
        return None;
    };
    info.iter()
        .map(|variant| match variant {
            VariantInfo::Unit(unit) => Some(unit.name()),
            _ => None,
        })
        .collect()
}

/// A caption and a select of the enum's variants, the select named `select` and each option
/// after its variant, so plans can pick one.
fn enum_row(
    row: ImmEntity<CapsPanel>,
    name: &'static str,
    caption: &'static str,
    variants: Vec<&'static str>,
    value: &mut dyn PartialReflect,
) {
    let ReflectRef::Enum(current) = value.reflect_ref() else {
        return;
    };
    let index = current.variant_index();
    row.on_spawn_insert(move || row_node(name)).add(|ui| {
        row_caption(ui, caption);
        let options = variants.clone();
        let mut select = ui
            .ch()
            .on_spawn_insert(move || PickedVariant {
                picked: None,
                shown: index,
            })
            .on_spawn_observe(pick_variant)
            .on_spawn_apply_scene(move || {
                let rows: Vec<Box<dyn SceneList>> = options
                    .iter()
                    .enumerate()
                    .map(|(i, &variant)| -> Box<dyn SceneList> {
                        if i == index {
                            bsn! { @FeathersListRow Selected OptionIndex(i) Name({variant}) Children [ @option_caption(variant) ] }.into()
                        } else {
                            bsn! { @FeathersListRow OptionIndex(i) Name({variant}) Children [ @option_caption(variant) ] }.into()
                        }
                    })
                    .collect();
                let rows: Box<dyn SceneList> = Box::new(rows);
                bsn! { @FeathersSelect { @options: {rows} } #select }
            });
        let Ok(Some(mut state)) = select.cap_get_component_mut::<PickedVariant>() else {
            return;
        };
        if let Some(picked) = state.picked.take() {
            if picked != index {
                value.apply(&DynamicEnum::new(variants[picked], DynamicVariant::Unit));
            }
        } else if state.shown != index {
            // Changed elsewhere: select the matching option.
            let entity = select.entity();
            select
                .commands()
                .queue(move |world: &mut World| show_variant(world, entity, index));
        }
    });
}

/// Records the variant picked in an enum's select, for the panel to apply on its next pass.
fn pick_variant(
    change: On<ValueChange<Entity>>,
    options: Query<&OptionIndex>,
    mut selects: Query<&mut PickedVariant>,
) {
    let (Ok(option), Ok(mut state)) = (
        options.get(change.value),
        selects.get_mut(change.event_target()),
    ) else {
        return;
    };
    state.picked = Some(option.0);
    state.shown = option.0;
}

/// Selects the option for variant `index` in the enum select `select`.
fn show_variant(world: &mut World, select: Entity, index: usize) {
    let mut options = world.query::<(Entity, &OptionIndex)>();
    let mut parents = world.query::<&ChildOf>();
    let row = options.iter(world).find_map(|(row, option)| {
        let mut ancestor = row;
        while let Ok(parent) = parents.get(world, ancestor) {
            ancestor = parent.parent();
            if ancestor == select {
                return (option.0 == index).then_some(row);
            }
        }
        None
    });
    if let Some(row) = row {
        world.trigger(SetSelected {
            entity: select,
            row,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentages_show_without_a_tail_and_read_back() {
        for (fraction, shown) in [(0.3, "30%"), (0.07, "7%"), (0.955, "95.5%"), (1.0, "100%")] {
            assert_eq!(
                RatioPercent.format(NumericValue::F32(fraction), false),
                shown
            );
            for typed in [shown, shown.trim_end_matches('%'), &format!("{shown} ")] {
                let read = RatioPercent.parse(typed.to_string(), NumericFormat::F32);
                assert_eq!(read, Ok(NumericValue::F32(fraction)), "{typed:?}");
            }
        }
    }
}
