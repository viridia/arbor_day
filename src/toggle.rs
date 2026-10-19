//! An operator's switch (`docs/patches.md`, Stacks): its settings and its state, off, disabled or
//! on, as Blender's modifiers are removed, hidden or shown. The generator runs it only when on.
//! Off, the panel shows none of its rows but lists it in its level's add menu, and the file leaves
//! it out; its settings are kept, so it comes back as it was. Disabled, its rows stay, and the
//! file keeps it, marked `"enabled": false`.

use std::any::TypeId;

use bevy::reflect::{
    FromReflect, PartialReflect, Reflect, ReflectMut, ReflectRef, TypeInfo, TypePath, TypeRegistry,
    serde::{
        DeserializeWithRegistry, ReflectDeserializeWithRegistry, ReflectSerializeWithRegistry,
        SerializeWithRegistry, TypedReflectDeserializer, TypedReflectSerializer,
    },
    structs::Struct,
};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, DeserializeSeed},
    ser::{Error as _, SerializeMap},
};

/// The key marking a disabled operator in a file, beside its settings.
pub const ENABLED: &str = "enabled";

/// Whether an operator is in its level, and if so whether it runs.
#[derive(Reflect, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OperatorState {
    /// Not in the level: no rows, and not in the file.
    Off,
    /// In the level, its rows shown, but skipped by the generator.
    Disabled,
    /// In the level and run.
    On,
}

/// An operator's settings and its state. A file holds its settings bare, under its slot's name,
/// unless it is off.
#[derive(Reflect, Clone, PartialEq)]
#[reflect(@IsToggle, SerializeWithRegistry, DeserializeWithRegistry)]
pub struct Toggle<T> {
    pub state: OperatorState,
    pub settings: T,
}

impl<T> Toggle<T> {
    /// `settings`, on.
    #[cfg(test)]
    pub fn on(settings: T) -> Self {
        Self {
            state: OperatorState::On,
            settings,
        }
    }

    /// `settings`, off.
    pub fn off(settings: T) -> Self {
        Self {
            state: OperatorState::Off,
            settings,
        }
    }

    /// The settings, if on.
    pub fn get(&self) -> Option<&T> {
        (self.state == OperatorState::On).then_some(&self.settings)
    }
}

/// Marks [`Toggle`]'s type, so reflection finds a toggle whatever it holds.
#[derive(Debug, Clone, Reflect)]
pub struct IsToggle;

/// `value`'s state, if it is a [`Toggle`].
pub fn toggle_state(value: &dyn PartialReflect) -> Option<OperatorState> {
    let Some(TypeInfo::Struct(info)) = value.get_represented_type_info() else {
        return None;
    };
    info.get_attribute::<IsToggle>()?;
    let ReflectRef::Struct(fields) = value.reflect_ref() else {
        return None;
    };
    fields.field("state")?.try_downcast_ref().copied()
}

/// Sets `value`'s state, if it is a [`Toggle`].
pub fn set_toggle_state(value: &mut dyn PartialReflect, state: OperatorState) {
    if toggle_state(value).is_none() {
        return;
    }
    if let ReflectMut::Struct(fields) = value.reflect_mut()
        && let Some(field) = fields.field_mut("state").and_then(|f| f.try_downcast_mut())
    {
        *field = state;
    }
}

/// Turns off every [`Toggle`] in `value`, at any depth, keeping their settings.
pub fn all_off(value: &mut dyn PartialReflect) {
    if toggle_state(value).is_some() {
        set_toggle_state(value, OperatorState::Off);
        return;
    }
    match value.reflect_mut() {
        ReflectMut::Struct(fields) => {
            for index in 0..fields.field_len() {
                if let Some(field) = fields.field_at_mut(index) {
                    all_off(field);
                }
            }
        }
        ReflectMut::Array(items) => {
            for index in 0..items.len() {
                if let Some(item) = items.get_mut(index) {
                    all_off(item);
                }
            }
        }
        _ => {}
    }
}

/// `value`'s fields written by reflection, leaving out its toggles that are off: a struct holding
/// operators writes itself with this, as its `SerializeWithRegistry`.
pub fn serialize_on<S: Serializer>(
    value: &dyn Struct,
    serializer: S,
    registry: &TypeRegistry,
) -> Result<S::Ok, S::Error> {
    let fields: Vec<_> = (0..value.field_len())
        .filter_map(|index| Some((value.name_at(index)?, value.field_at(index)?)))
        .filter(|(_, field)| toggle_state(*field) != Some(OperatorState::Off))
        .collect();
    let mut map = serializer.serialize_map(Some(fields.len()))?;
    for (name, field) in fields {
        map.serialize_entry(name, &TypedReflectSerializer::new(field, registry))?;
    }
    map.end()
}

/// On, the settings bare; disabled, `"enabled": false` and then the settings' fields.
impl<T: PartialReflect> SerializeWithRegistry for Toggle<T> {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
        registry: &TypeRegistry,
    ) -> Result<S::Ok, S::Error> {
        if self.state != OperatorState::Disabled {
            return TypedReflectSerializer::new(&self.settings, registry).serialize(serializer);
        }
        let ReflectRef::Struct(settings) = self.settings.reflect_ref() else {
            return Err(S::Error::custom("an operator's settings are a struct"));
        };
        let mut map = serializer.serialize_map(Some(settings.field_len() + 1))?;
        map.serialize_entry(ENABLED, &false)?;
        for index in 0..settings.field_len() {
            if let (Some(name), Some(field)) = (settings.name_at(index), settings.field_at(index)) {
                map.serialize_entry(name, &TypedReflectSerializer::new(field, registry))?;
            }
        }
        map.end()
    }
}

/// Read, a toggle is on, or disabled if marked so: one that is off is not in the file. Its
/// settings are read whole, since an operator's fields are fixed for its version
/// (`docs/patches.md`, The file), through a JSON value, whatever the format, so that `"enabled"`
/// can be taken off first.
impl<'de, T: FromReflect + TypePath> DeserializeWithRegistry<'de> for Toggle<T> {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
        registry: &TypeRegistry,
    ) -> Result<Self, D::Error> {
        let mut value = serde_json::Value::deserialize(deserializer)?;
        let state = match value.as_object_mut().and_then(|map| map.remove(ENABLED)) {
            None | Some(serde_json::Value::Bool(true)) => OperatorState::On,
            Some(serde_json::Value::Bool(false)) => OperatorState::Disabled,
            Some(other) => {
                return Err(de::Error::custom(format!(
                    "`{ENABLED}` is {other}, not a bool"
                )));
            }
        };
        let registration = registry
            .get(TypeId::of::<T>())
            .ok_or_else(|| de::Error::custom(format!("{} is not registered", T::type_path())))?;
        let read = TypedReflectDeserializer::new(registration, registry)
            .deserialize(value)
            .map_err(de::Error::custom)?;
        let settings = T::from_reflect(&*read).ok_or_else(|| {
            de::Error::custom(format!("{} is missing a field", T::short_type_path()))
        })?;
        Ok(Self { state, settings })
    }
}
