//! The property editor: panel rows for the fields of a reflected struct, one row per field, each
//! named after its field. Booleans get a checkbox, numbers a number input, and every other field a
//! label giving its name and type. Field attributes (below) set a row's caption and a number's
//! limits, precision, step, units and wrapping.

use std::ops::RangeInclusive;

use bevy::{
    feathers::{
        controls::{
            FeathersCheckbox, FeathersNumberInput, HardLimit, NumberInputPrecision,
            NumberInputStep, NumberInputUnits, NumberInputWrap, SoftLimit,
        },
        display::label,
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
        checked::ImmUiChecked,
        number_input::{ImmUiNumberInput, ImmUiNumericValue},
    },
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

/// A label and a number input, the input named `input` so plans can read its value.
fn number_row<T: NumberField>(
    row: ImmEntity<CapsUiFeathers>,
    name: &'static str,
    caption: &'static str,
    attrs: &CustomAttributes,
    value: &mut T,
) {
    let range = attrs
        .get::<ValueRange<T>>()
        .map(|range| T::range(range.0.clone()));
    let wrap = range.is_some() && attrs.get::<Wrap>().is_some();
    // Dragging clamps to the soft limit before wrapping, so a wrapping input has none.
    let slider = range.is_some() && attrs.get::<Spinbox>().is_none() && !wrap;
    let precision = attrs.get::<Precision>().map(|precision| precision.0);
    let step = attrs.get::<Step<T>>().map(|step| T::to_step(step.0));
    let units = attrs.get::<NumberInputUnits>().cloned();
    let initial = value.to_number_input_value();

    row.on_spawn_insert(move || {
        (
            Name::new(name),
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
        )
    })
    .add(|ui| {
        ui.ch()
            .on_spawn_apply_scene(move || bsn! { @label(caption) Node { flex_basis: percent(40) } });
        // The input writes its text from the components present when its text field spawns, so
        // the value, limits and units go in before the scene. Binding on the same frame would
        // write the text a second time while the first write is still queued, and both would
        // show ("11" for 1); so the binding starts the frame after.
        let input = ui
            .ch()
            .on_spawn_apply_commands(move |commands| {
                commands.insert((Name::new("input"), initial));
                if let Some(range) = range {
                    if slider {
                        commands.insert(SoftLimit(range.clone()));
                    }
                    commands.insert(HardLimit(range));
                }
                if wrap {
                    commands.insert(NumberInputWrap::Wrap);
                }
                if let Some(precision) = precision {
                    commands.insert(NumberInputPrecision(precision));
                }
                if let Some(step) = step {
                    commands.insert(NumberInputStep(step));
                }
                if let Some(units) = units {
                    commands.insert(units);
                }
            })
            .on_spawn_apply_scene(|| bsn! { @FeathersNumberInput Node { flex_grow: 1.0 } });
        if !input.will_be_spawned() {
            input.number_input(value);
        }
    });
}
