//! The species file (`docs/patches.md`, The file): the species written by reflection as JSON,
//! after the version of the generator it was made with.

use std::{any::TypeId, fmt};

use bevy::reflect::{
    ApplyError, PartialReflect, TypeInfo, TypeRegistration, TypeRegistry,
    serde::{ReflectDeserializerProcessor, TypedReflectDeserializer, TypedReflectSerializer},
    structs::{DynamicStruct, Struct, StructInfo},
};
use serde::{
    Deserialize, Deserializer,
    de::{self, DeserializeSeed},
    ser::{Serialize, SerializeMap, Serializer},
};

use crate::{
    species::{LimbLevel, LimbLevels, MAX_LIMB_LEVELS, Species},
    toggle::{ENABLED, IsToggle, OperatorState, all_off},
};

/// The generator's version, raised when output changes on purpose outside an operator (X15).
pub const GENERATOR: u32 = 1;

/// Why a species file was not read.
#[derive(Debug)]
pub enum ReadError {
    /// Not JSON, or not a species: not an object.
    Json(serde_json::Error),
    /// No `generator`, so not a species file.
    NoGenerator,
    /// Made by a newer generator than this one, which cannot grow it as that one did.
    Newer(u64),
    /// A value the species could not take.
    Apply(ApplyError),
}

impl fmt::Display for ReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "{error}"),
            Self::NoGenerator => write!(formatter, "not a species file: no `generator`"),
            Self::Newer(version) => write!(
                formatter,
                "made by generator {version}, newer than this one ({GENERATOR})"
            ),
            Self::Apply(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for ReadError {}

impl From<serde_json::Error> for ReadError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

/// A species read from a file, and what of the file was left out.
pub struct Read {
    pub species: Species,
    /// Each key the species lacks and each value it could not take, by path, with the reason
    /// (`limbs[0].wobble: not a field`). Its place keeps the template's value.
    pub pruned: Vec<String>,
}

/// The species in a species file, read as far as it goes. A field the file leaves out keeps the
/// template's value, the default species with every operator off, so a level's keeps its depth's
/// (`LimbLevels`), and an operator left out is off. So does a field the file holds but the species
/// cannot take, which is listed in `pruned`: a file from before a schema change opens (D32).
pub fn read(json: &str, registry: &TypeRegistry) -> Result<Read, ReadError> {
    let mut file: serde_json::Value = serde_json::from_str(json)?;
    let generator = file
        .as_object_mut()
        .and_then(|file| file.remove("generator"))
        .and_then(|generator| generator.as_u64())
        .ok_or(ReadError::NoGenerator)?;
    if generator > GENERATOR as u64 {
        return Err(ReadError::Newer(generator));
    }
    let registration = registry
        .get(TypeId::of::<Species>())
        .expect("the species is registered");
    let mut lenient = Lenient::default();
    let read = TypedReflectDeserializer::with_processor(registration, registry, &mut lenient)
        .deserialize(file)?;
    let mut species = Species::default();
    all_off(&mut species);
    species.try_apply(&*read).map_err(ReadError::Apply)?;
    Ok(Read {
        species,
        pruned: lenient.pruned,
    })
}

/// Reads every struct key by key, through a JSON value, so that a key its type lacks, or a value
/// that fails to read, is left out and noted rather than failing the file. Takes over from the
/// `DeserializeWithRegistry` of `Toggle` and `LimbLevels`, whose own reads are strict.
#[derive(Default)]
struct Lenient {
    /// Where the value being read sits, as `.name` and `[index]` segments.
    path: Vec<String>,
    pruned: Vec<String>,
}

impl ReflectDeserializerProcessor for Lenient {
    fn try_deserialize<'de, D: Deserializer<'de>>(
        &mut self,
        registration: &TypeRegistration,
        registry: &TypeRegistry,
        deserializer: D,
    ) -> Result<Result<Box<dyn PartialReflect>, D>, D::Error> {
        let TypeInfo::Struct(info) = registration.type_info() else {
            return Ok(Err(deserializer));
        };
        let value = serde_json::Value::deserialize(deserializer)?;
        let read = if registration.type_id() == TypeId::of::<LimbLevels>() {
            self.levels(value, registry)
        } else {
            self.fields(registration.type_info(), info, value, registry)
        };
        read.map(Ok).map_err(de::Error::custom)
    }
}

impl Lenient {
    /// A struct's fields, those it lacks or cannot take left out; a toggle's settings bare, beside
    /// `"enabled"` when disabled.
    fn fields(
        &mut self,
        type_info: &'static TypeInfo,
        info: &StructInfo,
        value: serde_json::Value,
        registry: &TypeRegistry,
    ) -> Result<Box<dyn PartialReflect>, String> {
        let serde_json::Value::Object(mut map) = value else {
            return Err(format!("{value} is not a {}", info.ty().short_path()));
        };
        let mut read = DynamicStruct::default();
        read.set_represented_type(Some(type_info));
        if info.get_attribute::<IsToggle>().is_some() {
            let state = match map.remove(ENABLED) {
                None | Some(serde_json::Value::Bool(true)) => OperatorState::On,
                Some(serde_json::Value::Bool(false)) => OperatorState::Disabled,
                Some(other) => {
                    self.prune(ENABLED, format!("{other} is not a bool"));
                    OperatorState::On
                }
            };
            read.insert("state", state);
            let settings = info.field("settings").expect("a toggle has settings");
            if let Some(settings) =
                self.value(settings.type_id(), serde_json::Value::Object(map), registry)
            {
                read.insert_boxed("settings", settings);
            }
            return Ok(Box::new(read));
        }
        for (key, value) in map {
            let Some(field) = info.field(&key) else {
                self.prune(&key, "not a field".into());
                continue;
            };
            self.path.push(format!(".{key}"));
            if let Some(value) = self.value(field.type_id(), value, registry) {
                read.insert_boxed(field.name(), value);
            }
            self.path.pop();
        }
        Ok(Box::new(read))
    }

    /// The limb levels, a list in the file: past the last level, or failing to read, a level is
    /// left out; a level failing to read keeps its depth's defaults.
    fn levels(
        &mut self,
        value: serde_json::Value,
        registry: &TypeRegistry,
    ) -> Result<Box<dyn PartialReflect>, String> {
        let serde_json::Value::Array(items) = value else {
            return Err(format!("{value} is not a list of limb levels"));
        };
        if items.is_empty() {
            return Err("no limb levels".into());
        }
        let mut levels: Vec<Box<dyn PartialReflect>> = Vec::new();
        for (index, item) in items.into_iter().enumerate() {
            if index == MAX_LIMB_LEVELS {
                self.prune(
                    &format!("[{index}]"),
                    format!("past the last of {MAX_LIMB_LEVELS} levels"),
                );
                break;
            }
            self.path.push(format!("[{index}]"));
            let level = self.value(TypeId::of::<LimbLevel>(), item, registry);
            self.path.pop();
            levels.push(level.unwrap_or_else(|| Box::new(DynamicStruct::default())));
        }
        Ok(Box::new(
            LimbLevels::from_read(&levels).map_err(|error| error.to_string())?,
        ))
    }

    /// `value` read as the type `type_id`, or none, noted at the current path, if it cannot be.
    fn value(
        &mut self,
        type_id: TypeId,
        value: serde_json::Value,
        registry: &TypeRegistry,
    ) -> Option<Box<dyn PartialReflect>> {
        let registration = registry
            .get(type_id)
            .expect("every species type is registered");
        match TypedReflectDeserializer::with_processor(registration, registry, self)
            .deserialize(value)
        {
            Ok(read) => Some(read),
            Err(error) => {
                let path = self.path.concat();
                self.pruned
                    .push(format!("{}: {error}", path.trim_start_matches('.')));
                None
            }
        }
    }

    /// Notes `key`, below the current path, as left out.
    fn prune(&mut self, key: &str, reason: String) {
        let key = if key.starts_with('[') {
            key.to_string()
        } else {
            format!(".{key}")
        };
        let path = self.path.concat() + &key;
        self.pruned
            .push(format!("{}: {reason}", path.trim_start_matches('.')));
    }
}

/// `species` as a species file: `generator`, then each of the species' fields by reflection, in the
/// order they are declared.
pub fn write(species: &Species, registry: &TypeRegistry) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&File { species, registry })
}

/// A species with the registry its fields are written through.
struct File<'a> {
    species: &'a Species,
    registry: &'a TypeRegistry,
}

impl Serialize for File<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let species = self.species;
        let mut file = serializer.serialize_map(Some(species.field_len() + 1))?;
        file.serialize_entry("generator", &GENERATOR)?;
        for index in 0..species.field_len() {
            if let (Some(name), Some(value)) = (species.name_at(index), species.field_at(index)) {
                file.serialize_entry(name, &TypedReflectSerializer::new(value, self.registry))?;
            }
        }
        file.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        species::Wander,
        toggle::{OperatorState, Toggle},
    };

    /// A registry holding the species and every type in it.
    fn registry() -> TypeRegistry {
        let mut registry = TypeRegistry::new();
        registry.register::<Species>();
        registry
    }

    #[test]
    fn the_default_species_is_written_as_the_golden_file() {
        let written = write(&Species::default(), &registry()).unwrap();
        let golden = include_str!("default_species.json");
        assert!(
            written.trim_end() == golden.trim_end(),
            "The default species is written differently; if that is meant, paste this into \
             src/default_species.json:\n{written}"
        );
    }

    #[test]
    fn only_the_shown_levels_are_written() {
        let mut species = Species::default();
        for shown in 1..=3 {
            species.limbs.shown = shown;
            let written = write(&species, &registry()).unwrap();
            let file: serde_json::Value = serde_json::from_str(&written).unwrap();
            assert_eq!(file["limbs"].as_array().unwrap().len(), shown as usize);
        }
    }

    /// `species` written, edited as JSON by `edit`, and read again.
    fn edited(
        species: &Species,
        edit: impl FnOnce(&mut serde_json::Value),
    ) -> Result<Read, ReadError> {
        let registry = registry();
        let mut file: serde_json::Value =
            serde_json::from_str(&write(species, &registry).unwrap()).unwrap();
        edit(&mut file);
        read(&file.to_string(), &registry)
    }

    /// `species` written and read again, nothing pruned.
    fn round_trip(species: &Species) -> Species {
        let registry = registry();
        let read = read(&write(species, &registry).unwrap(), &registry).unwrap();
        assert!(read.pruned.is_empty(), "{:?}", read.pruned);
        read.species
    }

    #[test]
    fn a_species_reads_back_as_it_was_written() {
        let mut species = Species::default();
        species.limbs.shown = 3;
        species.limbs.levels[2].fork.state = OperatorState::On;
        species.limbs.levels[2].fork.settings.chance = 0.5;
        species.trunk.wander = Toggle::on(Wander {
            angle: 0.3,
            ..Wander::degrees(0.0)
        });
        species.ornaments.tip_leaves.state = OperatorState::Off;
        assert!(round_trip(&species) == species);
    }

    #[test]
    fn a_place_curve_is_written_as_pairs_and_reads_back() {
        let mut species = Species::default();
        let bias = &mut species.limbs.levels[0].bias;
        bias.state = OperatorState::On;
        bias.settings.size.0[0][1] = 1.6;
        let written = write(&species, &registry()).unwrap();
        let file: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert_eq!(
            file["limbs"][0]["bias"]["size"],
            serde_json::json!([[0.0, 1.6], [1.0, 1.0]])
        );
        assert!(round_trip(&species) == species);
        // A third point, past the template's two.
        let read = edited(&species, |file| {
            file["limbs"][0]["bias"]["size"] =
                serde_json::json!([[0.0, 1.6], [0.5, 1.0], [1.0, 0.4]]);
        })
        .unwrap();
        assert!(read.pruned.is_empty(), "{:?}", read.pruned);
        let points = &read.species.limbs.levels[0].bias.settings.size.0;
        assert_eq!(points, &[[0.0, 1.6], [0.5, 1.0], [1.0, 0.4]]);
    }

    #[test]
    fn a_bare_plant_has_no_tip_leaves() {
        // On in the default, so a file without them is bare.
        let mut species = Species::default();
        let read_bare = edited(&species, |file| {
            file["ornaments"]
                .as_object_mut()
                .unwrap()
                .remove("tip_leaves");
        })
        .unwrap()
        .species;
        assert_eq!(read_bare.ornaments.tip_leaves.state, OperatorState::Off);
        species.ornaments.tip_leaves.state = OperatorState::Off;
        let written = write(&species, &registry()).unwrap();
        let file: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert!(file["ornaments"].get("tip_leaves").is_none());
    }

    #[test]
    fn a_hidden_level_is_not_kept() {
        // A hidden level lasts a session: the file holds the shown ones, so it reads as its
        // depth's defaults.
        let mut species = Species::default();
        species.limbs.levels[1].density = 1.0;
        assert!(round_trip(&species) == Species::default());
    }

    #[test]
    fn what_a_file_leaves_out_is_the_default_at_its_depth() {
        let mut species = Species::default();
        species.limbs.shown = 2;
        let read = edited(&species, |file| {
            file.as_object_mut().unwrap().remove("taper");
            file["limbs"][1].as_object_mut().unwrap().remove("density");
        })
        .unwrap();
        assert!(read.pruned.is_empty(), "{:?}", read.pruned);
        assert_eq!(read.species.taper, Species::default().taper);
        assert_eq!(read.species.limbs.levels[1].density, 3.5);
    }

    #[test]
    fn operators_are_written_and_read_in_each_state() {
        // Off, left out; disabled, marked; on, bare.
        let registry = registry();
        let mut species = Species::default();
        species.limbs.shown = 3;
        species.trunk.wander.state = OperatorState::Off;
        species.limbs.levels[0].wander.state = OperatorState::On;
        species.limbs.levels[1].wander.state = OperatorState::Disabled;
        species.limbs.levels[2].wander.state = OperatorState::Off;
        let written = write(&species, &registry).unwrap();
        let file: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert!(file["trunk"].get("wander").is_none());
        assert!(file["limbs"][0]["wander"].get("enabled").is_none());
        assert_eq!(file["limbs"][1]["wander"]["enabled"], false);
        assert!(file["limbs"][1]["wander"].get("angle").is_some());
        assert!(file["limbs"][2].get("wander").is_none());
        assert!(round_trip(&species) == species);
    }

    #[test]
    fn an_operator_missing_a_field_takes_its_default() {
        let mut species = Species::default();
        species.limbs.levels[0].wander.state = OperatorState::On;
        species.limbs.levels[0].wander.settings.angle = 0.5;
        let read = edited(&species, |file| {
            file["limbs"][0]["wander"]
                .as_object_mut()
                .unwrap()
                .remove("angle");
        })
        .unwrap();
        let wander = &read.species.limbs.levels[0].wander;
        assert_eq!(wander.state, OperatorState::On);
        assert_eq!(
            wander.settings.angle,
            Species::default().limbs.levels[0].wander.settings.angle
        );
    }

    #[test]
    fn what_the_species_cannot_take_is_pruned_by_path() {
        let species = Species {
            taper: 0.5,
            ..Species::default()
        };
        let read = edited(&species, |file| {
            file["limbs"][0]["wobble"] = 1.into();
            file["roots"] = true.into();
            file["trunk"]["height"] = "tall".into();
            file["crown"]["profile"] = "Pyramid".into();
            file["ornaments"]["tip_leaves"]["enabled"] = 0.into();
        })
        .unwrap();
        let mut pruned: Vec<_> = (read.pruned.iter())
            .map(|line| line.split(':').next().unwrap())
            .collect();
        pruned.sort();
        assert_eq!(
            pruned,
            [
                "crown.profile",
                "limbs[0].wobble",
                "ornaments.tip_leaves.enabled",
                "roots",
                "trunk.height",
            ]
        );
        let default = Species::default();
        assert_eq!(read.species.trunk.height, default.trunk.height);
        assert!(read.species.crown.profile == default.crown.profile);
        assert_eq!(read.species.taper, 0.5);
        assert_eq!(read.species.ornaments.tip_leaves.state, OperatorState::On);
    }

    #[test]
    fn a_file_from_a_newer_generator_is_refused() {
        let error = edited(&Species::default(), |file| {
            file["generator"] = (GENERATOR + 1).into();
        });
        assert!(matches!(error, Err(ReadError::Newer(2))));
        let error = edited(&Species::default(), |file| {
            file.as_object_mut().unwrap().remove("generator");
        });
        assert!(matches!(error, Err(ReadError::NoGenerator)));
    }

    #[test]
    fn levels_number_one_to_three() {
        let mut species = Species::default();
        species.limbs.shown = 3;
        let four = edited(&species, |file| {
            let level = file["limbs"][0].clone();
            file["limbs"].as_array_mut().unwrap().push(level);
        })
        .unwrap();
        assert!(four.pruned[0].starts_with("limbs[3]:"), "{:?}", four.pruned);
        assert_eq!(four.species.limbs.shown, 3);
        let none = edited(&species, |file| file["limbs"] = serde_json::json!([])).unwrap();
        assert!(none.pruned[0].starts_with("limbs:"), "{:?}", none.pruned);
        assert!(none.species.limbs == Species::default().limbs);
    }
}
