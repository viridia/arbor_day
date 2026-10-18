//! The property editor: panel rows for the fields of a reflected struct, one row per field, each
//! named after its field. Booleans get a checkbox, numbers a number input, fuzzy numbers a center,
//! a spread and a shape toggle, colors a swatch that opens a picker, enums of unit variants a
//! select, nested structs a subpane of rows of their own, or in a subpane a heading over them, or
//! with [`Flatten`] their rows in place of their own, an array with a [`ShownCount`] a subpane per
//! shown item, and every other field a label giving its
//! name and type. Every row is one line:
//! a number input carries its caption inside, and other widgets sit right of theirs. Field
//! attributes (below) set a row's
//! caption, its control group, and a number's limits, precision, step and units.

use std::{collections::HashMap, ops::RangeInclusive};

use bevy::{
    app::PropagateOver,
    feathers::{
        constants::{fonts, icons, size},
        containers::{subpane, subpane_body, subpane_header},
        controls::{
            ButtonVariant, ColorInputValue, FeathersCheckbox, FeathersColorInput, FeathersListRow,
            FeathersMenu, FeathersMenuDivider, FeathersMenuItem, FeathersMenuPopup,
            FeathersMenuToolButton, FeathersNumberInput, FeathersSelect, FeathersToolButton,
            HardLimit, NumberInputPrecision, NumberInputStep, NumberInputUnits, OptionIndex,
            SoftLimit,
        },
        display::{caption as option_caption, icon, label},
        theme::{ThemeTextColor, ThemeToken, ThemedText},
        tokens,
    },
    prelude::*,
    reflect::{
        FromReflect, GetTypeRegistration, NamedField, ReflectMut, ReflectRef, TypeInfo, Typed,
        array::Array,
        attributes::CustomAttributes,
        enums::{DynamicEnum, DynamicVariant, VariantInfo},
        structs::Struct,
    },
    text::FontSourceTemplate,
    ui::Selected,
    ui_widgets::{
        NumericRange, SetSelected, ValueChange,
        popover::{Popover, PopoverAlign, PopoverPlacement, PopoverSide},
    },
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
    fuzzy::{Distribution, Fuzzy, Interval},
    random::Seed,
    shortcut::shortcut,
    spread_strip::{StripRange, spread_strip},
    toggle::{OperatorState, set_toggle_state, toggle_state},
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

/// Decimal places a number is rounded to while dragging, in the units shown: 1 for an angle is a
/// tenth of a degree.
#[derive(Debug, Clone, Reflect)]
pub struct Precision(pub i32);

/// How far one step of the input's arrows moves a number.
#[derive(Debug, Clone, Reflect)]
pub struct Step<T>(pub T);

/// The row's label, in place of the field's name.
#[derive(Debug, Clone, Reflect)]
pub struct Caption(pub &'static str);

/// The control group a field's row sits in: the rows of a struct's fields naming the same group
/// are gathered under its name, where the first of them would be. In a subpane the name is a
/// heading over the rows; at the panel's top level the group is a subpane of its own. The
/// grouping is the panel's alone: the fields stay where they are in the struct.
#[derive(Debug, Clone, Reflect)]
pub struct ControlGroup(pub &'static str);

/// On a struct field: its fields' rows in place of its own, as if they were the outer struct's,
/// with no subpane, heading or node of their own.
#[derive(Debug, Clone, Reflect)]
pub struct Flatten;

/// On a field shown as a subpane at the panel's top level, or the first field of a top-level
/// [`ControlGroup`]: the color of a stripe down the subpane's left side, so parts of the value are
/// told apart at a glance. An array's items take one color each, in order. A subpane with none
/// keeps the stripe's room, so subpanes stay aligned.
#[derive(Debug, Clone, Reflect)]
pub struct Stripe(pub Vec<Color>);

/// The stripe color of item `index` of the field with attributes `attrs`, clear if it has none.
fn stripe(attrs: &CustomAttributes, index: usize) -> Color {
    (attrs.get::<Stripe>())
        .and_then(|stripe| stripe.0.get(index).copied())
        .unwrap_or(Color::NONE)
}

/// On an operator's settings type: the icon left of its heading, an image path under `assets/`.
#[derive(Debug, Clone, Reflect)]
pub struct Icon(pub &'static str);

/// A number in a [`ValueRange`] shown without the slider fill, as a spinbox with step arrows.
#[derive(Debug, Clone, Reflect)]
pub struct Spinbox;

/// On an array of structs: the sibling `i32` field holding how many of its items are shown, from
/// the first. Each shown item is a subpane, captioned with the array's caption and its number,
/// beside the struct's other rows. The last one's add menu shows the next item, and each header
/// past the first has a ✕ hiding its item and those after it, within the count field's
/// [`ValueRange`] and the array's length, and never below 1. The count field gets no row of its
/// own.
#[derive(Debug, Clone, Reflect)]
pub struct ShownCount(pub &'static str);

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
/// by the item's name (`levels[0]`): figures about the item that the panel's owner knows and the
/// item does not.
pub type HeaderNotes = HashMap<String, String>;

/// Adds a row for each field of `value`, which must be a struct, editing the fields in place.
pub fn edit_fields(ui: &mut Imm<CapsPanel>, value: &mut dyn PartialReflect, notes: &HeaderNotes) {
    edit_struct(ui, value, notes, false);
}

/// The rows of `value`'s fields. A [`ControlGroup`] or a nested struct is a subpane at the panel's
/// top level and, in a subpane (`in_pane`), a heading over its rows, so panes do not nest boxes in
/// boxes.
fn edit_struct(
    ui: &mut Imm<CapsPanel>,
    value: &mut dyn PartialReflect,
    notes: &HeaderNotes,
    in_pane: bool,
) {
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
    let group_of = |index: usize| {
        info.field_at(index)
            .and_then(|field| field.custom_attributes().get::<ControlGroup>())
            .map(|group| group.0)
    };
    for index in 0..fields.field_len() {
        let Some(field_info) = info.field_at(index) else {
            continue;
        };
        let Some(caption) = group_of(index) else {
            field_row(
                ui,
                field_info,
                fields,
                index,
                &counts,
                &mut new_counts,
                notes,
                in_pane,
            );
            continue;
        };
        if (0..index).any(|earlier| group_of(earlier) == Some(caption)) {
            continue;
        }
        let members: Vec<usize> = (index..fields.field_len())
            .filter(|&member| group_of(member) == Some(caption))
            .collect();
        let rows = |ui: &mut Imm<CapsPanel>| {
            for member in members {
                if let Some(member_info) = info.field_at(member) {
                    field_row(
                        ui,
                        member_info,
                        fields,
                        member,
                        &counts,
                        &mut new_counts,
                        notes,
                        true,
                    );
                }
            }
        };
        if in_pane {
            ui.ch_id(caption)
                .on_spawn_apply_scene(move || group_heading(caption));
            rows(ui);
        } else {
            let color = stripe(field_info.custom_attributes(), 0);
            group(ui.ch_id(caption), caption, color, rows);
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

/// The row for field `index` of `fields`, described by `field_info`, in a subpane if `in_pane`. A
/// count field named by a [`ShownCount`] in `counts` gets none; the array it counts pushes its new
/// count to `new_counts`.
fn field_row(
    ui: &mut Imm<CapsPanel>,
    field_info: &NamedField,
    fields: &mut dyn Struct,
    index: usize,
    counts: &[(&'static str, i32, RangeInclusive<i32>)],
    new_counts: &mut Vec<(&'static str, i32)>,
    notes: &HeaderNotes,
    in_pane: bool,
) {
    let Some(field) = fields.field_at_mut(index) else {
        return;
    };
    let name = field_info.name();
    let attrs = field_info.custom_attributes();
    let caption = attrs.get::<Caption>().map_or(name, |caption| caption.0);
    if counts.iter().any(|(count, ..)| *count == name) {
        return;
    }
    if let Some(shown) = attrs.get::<ShownCount>()
        && let Some((_, count, range)) = counts.iter().find(|(count, ..)| *count == shown.0)
        && let ReflectMut::Array(items) = field.reflect_mut()
    {
        let new_count = shown_items(
            ui,
            name,
            caption,
            attrs,
            items,
            *count,
            range.clone(),
            notes,
        );
        if new_count != *count {
            new_counts.push((shown.0, new_count));
        }
        return;
    }
    if attrs.get::<Flatten>().is_some() {
        edit_struct(ui, field, notes, in_pane);
        return;
    }
    if let Some(state) = toggle_state(field) {
        // Off, it has no rows: its struct's add menu lists it.
        if state != OperatorState::Off {
            operator(ui, name, caption, state, field, notes);
        }
        return;
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
    } else if let Some(interval) = field.try_downcast_mut::<Interval>() {
        interval_row(row, name, caption, attrs, interval);
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
        let row = row.on_spawn_insert(move || Name::new(name));
        if in_pane {
            row.on_spawn_insert(heading_node).add(|ui| {
                ui.ch().on_spawn_apply_scene(move || group_heading(caption));
                edit_struct(ui, field, notes, true);
            });
        } else {
            let toggles = toggles(field);
            let mut added = None;
            let tools = |ui: &mut Imm<CapsPanel>| {
                if !toggles.is_empty()
                    && let Some(Picked::Operator(field)) = add_menu(ui, &toggles, None)
                {
                    added = Some(field);
                }
            };
            group_with_tools(row, caption.to_string(), stripe(attrs, 0), tools, |ui| {
                edit_struct(ui, field, notes, true);
            });
            if let Some(added) = added {
                turn_on(field, added);
            }
        }
    } else {
        let text = format!("{caption}: {}", field.reflect_short_type_path());
        row.on_spawn_apply_scene(move || label(text))
            .on_spawn_insert(move || Name::new(name));
    }
}

/// The first `count` of `items`, a subpane each named `name[index]` and headed with its note, if
/// any, striped as the array's attributes `attrs` say. The last one's add menu ends with the next
/// item, named `add`, and each header past the fewest shown has a ✕ named `remove`, hiding its
/// item and those after it. Returns the count as they leave it.
fn shown_items(
    ui: &mut Imm<CapsPanel>,
    name: &'static str,
    caption: &'static str,
    attrs: &CustomAttributes,
    items: &mut dyn Array,
    count: i32,
    range: RangeInclusive<i32>,
    notes: &HeaderNotes,
) -> i32 {
    // Never none: with no subpane there would be no add menu to bring one back.
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
        let next = (last && count < max).then(|| format!("{caption} {}", index + 2));
        let toggles = toggles(item);
        let mut added = None;
        let tools = |ui: &mut Imm<CapsPanel>| {
            match add_menu(ui, &toggles, next) {
                Some(Picked::Operator(field)) => added = Some(field),
                Some(Picked::Item) => new_count = count + 1,
                None => {}
            }
            if index as i32 >= min && icon_button(ui, "remove", icons::X, px(14)) {
                new_count = index as i32;
            }
        };
        group_with_tools(pane, header, stripe(attrs, index), tools, |ui| {
            edit_struct(ui, item, notes, true);
        });
        if let Some(added) = added {
            turn_on(item, added);
        }
    }
    new_count.clamp(min, max)
}

/// What was picked from an add menu.
enum Picked {
    /// An operator that was off, by its field's name.
    Operator(&'static str),
    /// The next item of a [`ShownCount`] array.
    Item,
}

/// A [`Toggle`] field of a struct: its name, its caption and its state.
///
/// [`Toggle`]: crate::toggle::Toggle
type ToggleField = (&'static str, &'static str, OperatorState);

/// `value`'s fields that are [`Toggle`]s.
///
/// [`Toggle`]: crate::toggle::Toggle
fn toggles(value: &dyn PartialReflect) -> Vec<ToggleField> {
    let Some(TypeInfo::Struct(info)) = value.get_represented_type_info() else {
        return Vec::new();
    };
    let ReflectRef::Struct(fields) = value.reflect_ref() else {
        return Vec::new();
    };
    info.iter()
        .filter_map(|field| {
            let state = toggle_state(fields.field(field.name())?)?;
            let caption = field.custom_attributes().get::<Caption>();
            Some((
                field.name(),
                caption.map_or(field.name(), |caption| caption.0),
                state,
            ))
        })
        .collect()
}

/// Turns on the toggle in `value`'s field `name`.
fn turn_on(value: &mut dyn PartialReflect, name: &str) {
    if let ReflectMut::Struct(fields) = value.reflect_mut()
        && let Some(toggle) = fields.field_mut(name)
    {
        set_toggle_state(toggle, OperatorState::On);
    }
}

/// A header's add menu, named `add_operator`: a boxed plus listing those of `toggles` that are off,
/// each item named `add_` and its field's name, then below a divider `next`, if given, the next
/// item of an array, named `add`. Disabled while it lists nothing. Returns what was picked.
fn add_menu(
    ui: &mut Imm<CapsPanel>,
    toggles: &[ToggleField],
    next: Option<String>,
) -> Option<Picked> {
    let off: Vec<_> = (toggles.iter())
        .filter(|(.., state)| *state == OperatorState::Off)
        .map(|&(field, caption, _)| (field, caption))
        .collect();
    let mut picked = None;
    ui.ch()
        .on_spawn_apply_scene(|| bsn! { @FeathersMenu #add_operator })
        .add(|ui| {
            ui.ch()
                .on_spawn_apply_scene(|| {
                    bsn! {
                        @FeathersMenuToolButton {
                            @caption: bsn! { @icon("icons/add_box.png") Node { height: px(18) } },
                            @arrow: false,
                        }
                    }
                })
                .interactions_disabled(off.is_empty() && next.is_none());
            ui.ch()
                .on_spawn_apply_scene(|| {
                    bsn! {
                        @FeathersMenuPopup
                        // Right-aligned, since the add button sits at the header's right end.
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
                    }
                })
                .add(|ui| {
                    let operators = !off.is_empty();
                    for (field, caption) in off {
                        let item = format!("add_{field}");
                        if menu_item(ui.ch_id(field), caption.to_string(), None, item) {
                            picked = Some(Picked::Operator(field));
                        }
                    }
                    if let Some(next) = next {
                        if operators {
                            ui.ch_id("divider")
                                .on_spawn_apply_scene(|| bsn! { @FeathersMenuDivider });
                        }
                        if menu_item(ui.ch_id("add"), next, None, "add".to_string()) {
                            picked = Some(Picked::Item);
                        }
                    }
                });
        });
    picked
}

/// A feathers menu item showing `caption`, and its shortcut `keys` at its right end if given,
/// named `name`. Returns whether it was picked.
fn menu_item(
    item: ImmEntity<CapsPanel>,
    caption: String,
    keys: Option<&'static str>,
    name: String,
) -> bool {
    item.on_spawn_apply_scene(move || {
        let caption = caption.clone();
        let label: Box<dyn SceneList> = match keys {
            Some(keys) => Box::new(bsn_list! { Text(caption) ThemedText -- @shortcut(keys) }),
            None => Box::new(bsn_list! { Text(caption) ThemedText }),
        };
        bsn! {
            @FeathersMenuItem { @caption: label }
            Name({name.clone()})
        }
    })
    .activated()
}

/// An operator that is on or disabled, in a subpane: its heading, after its settings type's
/// [`Icon`] if it has one, over its settings' rows, in a node named `name`. At the heading's
/// right, an eye named `enabled` switches it between on and disabled, showing which, and a ✕
/// named `off` turns it off.
fn operator(
    ui: &mut Imm<CapsPanel>,
    name: &'static str,
    caption: &'static str,
    state: OperatorState,
    toggle: &mut dyn PartialReflect,
    notes: &HeaderNotes,
) {
    let mut new_state = state;
    let ReflectMut::Struct(fields) = toggle.reflect_mut() else {
        return;
    };
    let kind_icon = fields
        .field("settings")
        .and_then(|settings| settings.get_represented_type_info())
        .and_then(|info| info.as_struct().ok())
        .and_then(|info| info.get_attribute::<Icon>())
        .map(|icon| icon.0);
    ui.ch_id(name)
        .on_spawn_insert(move || (Name::new(name), heading_node()))
        .add(|ui| {
            ui.ch()
                .on_spawn_insert(|| Node {
                    margin: UiRect::top(px(6)),
                    ..caption_line()
                })
                .add(|ui| {
                    // Brighter while on; keyed by it, so the eye's click redraws it.
                    let on = state == OperatorState::On;
                    let color = if on {
                        tokens::TEXT_MAIN
                    } else {
                        tokens::TEXT_DIM
                    };
                    ui.ch_id(("title", on))
                        .on_spawn_insert(widget_line)
                        .add(|ui| {
                            if let Some(image) = kind_icon {
                                let color = color.clone();
                                ui.ch().on_spawn_apply_scene(move || {
                                    bsn! {
                                        @icon(image)
                                        ThemeTextColor({color})
                                        Name("icon")
                                    }
                                });
                            }
                            ui.ch()
                                .on_spawn_apply_scene(move || heading_text(caption, color));
                        });
                    ui.ch().on_spawn_insert(widget_line).add(|ui| {
                        let image = match state {
                            OperatorState::On => "icons/visibility.png",
                            _ => "icons/visibility_off.png",
                        };
                        if icon_button(ui, "enabled", image, px(16)) {
                            new_state = match state {
                                OperatorState::On => OperatorState::Disabled,
                                _ => OperatorState::On,
                            };
                        }
                        if icon_button(ui, "off", icons::X, px(14)) {
                            new_state = OperatorState::Off;
                        }
                    });
                });
            if let Some(settings) = fields.field_mut("settings") {
                edit_struct(ui, settings, notes, true);
            }
        });
    if new_state != state {
        set_toggle_state(toggle, new_state);
    }
}

/// A plain tool button named `name`, showing `image` `height` high, keyed by the image so a new
/// one replaces it. Returns whether it was clicked.
fn icon_button(
    ui: &mut Imm<CapsPanel>,
    name: &'static str,
    image: &'static str,
    height: Val,
) -> bool {
    let mut button = ui.ch_id(name).on_spawn_apply_scene(move || {
        bsn! {
            @FeathersToolButton { @variant: {ButtonVariant::Plain} }
            Name({name})
        }
    });
    let clicked = button.activated();
    button.add(|ui| {
        ui.ch_id(image)
            .on_spawn_apply_scene(move || bsn! { @icon(image) Node { height: {height} } });
    });
    clicked
}

/// A feathers subpane headed `caption`, striped in `stripe`, its body built by `body`.
pub fn group<'w, 's>(
    pane: ImmEntity<'_, 'w, 's, CapsPanel>,
    caption: &'static str,
    stripe: Color,
    body: impl FnOnce(&mut Imm<'w, 's, CapsPanel>),
) {
    group_with_tools(pane, caption.to_string(), stripe, |_| {}, body);
}

/// A feathers subpane headed `caption`, with a [`Stripe`] in `stripe` down its left side and
/// buttons built by `tools` at the header's right end, its body built by `body`. The caption
/// follows `caption` as it changes.
fn group_with_tools<'w, 's>(
    pane: ImmEntity<'_, 'w, 's, CapsPanel>,
    caption: String,
    stripe: Color,
    tools: impl FnOnce(&mut Imm<'w, 's, CapsPanel>),
    body: impl FnOnce(&mut Imm<'w, 's, CapsPanel>),
) {
    // Set on the node, since feathers' theming cannot shade a subtree.
    let pane = pane.on_spawn_apply_scene(move || {
        bsn! {
            @subpane()
            Node { border: UiRect::left(px(4)), padding: UiRect::left(px(4)) }
            BorderColor { left: {stripe} }
        }
    });
    pane.add(|ui| {
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

/// A row node named after its field, holding widgets that carry their caption inside.
fn row_node(name: &'static str) -> (Name, Node) {
    (
        Name::new(name),
        Node {
            flex_direction: FlexDirection::Column,
            ..default()
        },
    )
}

/// A control group's heading in a subpane: bold and dimmed, so it reads apart from the row
/// captions below it, with a gap above that sets it off from the rows before.
fn group_heading(caption: &'static str) -> impl Scene {
    bsn! {
        @heading_text(caption, tokens::TEXT_DIM)
        Node { margin: UiRect::top(px(6)) }
    }
}

/// A heading's text, bold, in `color`.
fn heading_text(caption: &'static str, color: ThemeToken) -> impl Scene {
    bsn! {
        Text(caption)
        TextFont {
            font: FontSourceTemplate::Handle(fonts::BOLD),
            font_size: size::SMALL_FONT,
            weight: FontWeight::BOLD,
        }
        PropagateOver<TextFont>
        ThemeTextColor({color})
    }
}

/// A nested struct's heading and rows in a subpane, spaced as the subpane's own rows are.
fn heading_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(4),
        ..default()
    }
}

/// Widgets laid out left to right.
fn widget_line() -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        // As wide as the gap between rows.
        column_gap: px(4),
        ..default()
    }
}

/// A caption at the left end of a row and a widget at the right.
fn caption_line() -> Node {
    Node {
        justify_content: JustifyContent::SpaceBetween,
        ..widget_line()
    }
}

/// A number input captioned inside, named `input` so plans can read its value.
fn number_row<T: NumberField>(
    row: ImmEntity<CapsPanel>,
    name: &'static str,
    caption: &'static str,
    attrs: &CustomAttributes,
    value: &mut T,
) {
    let look = NumberLook::from_attributes::<T>(attrs);
    row.on_spawn_insert(move || row_node(name)).add(|ui| {
        number_input(ui.ch(), "input", Some(caption), look, percent(100), value);
    });
}

/// A number input named `name`, its caption, if any, inside it, `width` wide, or taking a widget
/// line's spare width if `Val::Auto`. Returns the input, for adding children of its own.
fn number_input<'r, 'w, 's, T: NumberField>(
    input: ImmEntity<'r, 'w, 's, CapsPanel>,
    name: &'static str,
    caption: Option<&'static str>,
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
        .on_spawn_apply_scene(move || {
            bsn! {
                @FeathersNumberInput { @interior_label_text: {caption} }
                Node { flex_grow: {grow}, width: {width} }
            }
        });
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
        ui.ch().on_spawn_insert(widget_line).add(|ui| {
            number_input(
                ui.ch(),
                "input",
                Some(caption),
                look,
                Val::Auto,
                &mut seed.0,
            );
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

/// A fuzzy number's row: its center and spread (`center_and_spread`), and a button that toggles
/// the distribution's shape, showing the current one.
fn fuzzy_row(
    row: ImmEntity<CapsPanel>,
    name: &'static str,
    caption: &'static str,
    attrs: &CustomAttributes,
    fuzzy: &mut Fuzzy,
) {
    row.on_spawn_insert(move || row_node(name)).add(|ui| {
        ui.ch().on_spawn_insert(widget_line).add(|ui| {
            let (center, spread) = (&mut fuzzy.center, &mut fuzzy.spread);
            center_and_spread(ui, caption, attrs, center, spread, fuzzy.shape);
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

/// An interval's row: a fuzzy number's, with an empty space where its shape button would be, so
/// the inputs line up with those of fuzzy rows.
fn interval_row(
    row: ImmEntity<CapsPanel>,
    name: &'static str,
    caption: &'static str,
    attrs: &CustomAttributes,
    interval: &mut Interval,
) {
    row.on_spawn_insert(move || row_node(name)).add(|ui| {
        ui.ch().on_spawn_insert(widget_line).add(|ui| {
            let (center, spread) = (&mut interval.center, &mut interval.spread);
            center_and_spread(ui, caption, attrs, center, spread, Distribution::Uniform);
            // As wide as the shape button: its icon and padding.
            ui.ch().on_spawn_insert(|| Node {
                width: px(26),
                flex_shrink: 0.0,
                ..default()
            });
        });
    });
}

/// A center as a slider and a spread as a spinbox, for fuzzy and interval rows. The field's
/// [`ValueRange`] bounds the center; the spread goes up to half of it, so a spread can reach
/// across the whole range. With a range, the center input also shows the spread strip, in
/// `shape`.
fn center_and_spread(
    ui: &mut Imm<CapsPanel>,
    caption: &'static str,
    attrs: &CustomAttributes,
    center: &mut f32,
    spread: &mut f32,
    shape: Distribution,
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
    let input = number_input(
        ui.ch(),
        "center",
        Some(caption),
        center_look,
        Val::Auto,
        center,
    );
    // Drawn from this frame's center and last frame's spread and shape, a frame behind when they
    // change.
    if let Some(range) = strip_range {
        let drawn = Fuzzy::new(*center, *spread, shape);
        input.add(|ui| spread_strip(ui, drawn, range));
    }
    number_input(ui.ch(), "spread", None, spread_look, px(56), spread);
}

/// A caption with a feathers color input, a swatch that opens a picker, at the row's right end.
/// The input is named `input`.
fn color_row(
    row: ImmEntity<CapsPanel>,
    name: &'static str,
    caption: &'static str,
    color: &mut Color,
) {
    row.on_spawn_insert(move || (Name::new(name), caption_line()))
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

/// A caption with a select of the enum's variants at the row's right end, the select named
/// `select` and each option after its variant, so plans can pick one.
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
    row.on_spawn_insert(move || (Name::new(name), caption_line()))
        .add(|ui| {
        ui.ch().on_spawn_apply_scene(move || label(caption));
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
