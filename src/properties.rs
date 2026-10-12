//! The property editor: panel rows for the fields of a reflected struct, one row per field, each
//! named after its field. Booleans get a checkbox, numbers a number input, fuzzy numbers a center,
//! a spread and a shape toggle, and every other field a label giving its name and type. A row's
//! caption sits above its inputs, so they get the panel's full width. Field attributes (below) set
//! a row's caption and a number's limits, precision, step, units and wrapping.

use std::ops::RangeInclusive;

use bevy::{
    feathers::{
        controls::{
            ButtonVariant, FeathersCheckbox, FeathersNumberInput, FeathersToolButton, HardLimit,
            NumberInputPrecision, NumberInputStep, NumberInputUnits, NumberInputWrap, SoftLimit,
        },
        display::{icon, label, label_small},
        theme::ThemedText,
    },
    prelude::*,
    reflect::{
        FromReflect, GetTypeRegistration, ReflectMut, TypeInfo, Typed, attributes::CustomAttributes,
    },
    ui_widgets::NumericRange,
};
use bevy_immediate::{
    Imm, ImmEntity,
    ui::{
        CapsUiFeathers,
        activated::ImmUiActivated,
        checked::ImmUiChecked,
        number_input::{ImmUiNumberInput, ImmUiNumericValue},
    },
};

use crate::{
    fuzzy::{Distribution, Fuzzy},
    spread_strip::{StripRange, spread_strip},
};

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

/// A number that wraps around its [`ValueRange`] instead of stopping at its ends, such as an angle
/// of a full turn. It is shown as a spinbox, since a slider's fill would stop at the ends.
#[derive(Debug, Clone, Reflect)]
pub struct Wrap;

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

/// Adds a row for each field of `value`, which must be a struct, editing the fields in place.
pub fn edit_fields(ui: &mut Imm<CapsUiFeathers>, value: &mut dyn PartialReflect) {
    // Attributes belong to the type, so read them from its info, not the value.
    let Some(TypeInfo::Struct(info)) = value.get_represented_type_info() else {
        return;
    };
    let ReflectMut::Struct(fields) = value.reflect_mut() else {
        return;
    };
    for index in 0..fields.field_len() {
        let (Some(field_info), Some(field)) = (info.field_at(index), fields.field_at_mut(index))
        else {
            continue;
        };
        let name = field_info.name();
        let attrs = field_info.custom_attributes();
        let caption = attrs.get::<Caption>().map_or(name, |caption| caption.0);
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
        } else if let Some(number) = field.try_downcast_mut::<f32>() {
            number_row(row, name, caption, attrs, number);
        } else if let Some(number) = field.try_downcast_mut::<f64>() {
            number_row(row, name, caption, attrs, number);
        } else if let Some(number) = field.try_downcast_mut::<i32>() {
            number_row(row, name, caption, attrs, number);
        } else if let Some(number) = field.try_downcast_mut::<i64>() {
            number_row(row, name, caption, attrs, number);
        } else {
            let text = format!("{caption}: {}", field.reflect_short_type_path());
            row.on_spawn_apply_scene(move || label(text))
                .on_spawn_insert(move || Name::new(name));
        }
    }
}

/// How a number input limits and shows its number, read from a field's attributes.
#[derive(Clone)]
struct NumberLook {
    range: Option<NumericRange>,
    /// The fill bar dragged like a slider, rather than a spinbox.
    slider: bool,
    wrap: bool,
    precision: Option<i32>,
    step: Option<f64>,
    units: Option<NumberInputUnits>,
}

impl NumberLook {
    fn from_attributes<T: NumberField>(attrs: &CustomAttributes) -> Self {
        let range = attrs
            .get::<ValueRange<T>>()
            .map(|range| T::range(range.0.clone()));
        let wrap = range.is_some() && attrs.get::<Wrap>().is_some();
        Self {
            // Dragging clamps to the soft limit before wrapping, so a wrapping input has none.
            slider: range.is_some() && attrs.get::<Spinbox>().is_none() && !wrap,
            range,
            wrap,
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
fn row_caption(ui: &mut Imm<CapsUiFeathers>, caption: &'static str) {
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
    row: ImmEntity<CapsUiFeathers>,
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
    input: ImmEntity<'r, 'w, 's, CapsUiFeathers>,
    name: &'static str,
    look: NumberLook,
    width: Val,
    value: &mut T,
) -> ImmEntity<'r, 'w, 's, CapsUiFeathers> {
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
            if look.wrap {
                commands.insert(NumberInputWrap::Wrap);
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

/// A fuzzy number's row: its center as a slider, its spread as a spinbox, and a button that
/// toggles the distribution's shape, showing the current one. The field's [`ValueRange`] bounds
/// the center; the spread goes up to half of it, so a spread can reach across the whole range.
/// With a range, the center input also shows the spread strip.
fn fuzzy_row(
    row: ImmEntity<CapsUiFeathers>,
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
            wrap: center_look.wrap,
        }),
        _ => None,
    };
    let half_range = strip_range.map_or(f32::MAX, |range| (range.end - range.start) * 0.5);
    let spread_look = NumberLook {
        range: Some(NumericRange::F32(0.0..=half_range)),
        slider: false,
        wrap: false,
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
                    Name("shape")
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
