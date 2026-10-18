//! The species file (`docs/patches.md`, The file): the species written by reflection as JSON,
//! after the version of the generator it was made with.

use std::{any::TypeId, fmt};

use bevy::reflect::{
    ApplyError, PartialReflect, TypeRegistry,
    serde::{TypedReflectDeserializer, TypedReflectSerializer},
    structs::Struct,
};
use serde::{
    de::DeserializeSeed,
    ser::{Serialize, SerializeMap, Serializer},
};

use crate::{species::Species, toggle::all_off};

/// The generator's version, raised when output changes on purpose outside an operator (X15).
pub const GENERATOR: u32 = 1;

/// Why a species file was not read.
#[derive(Debug)]
pub enum ReadError {
    /// Not JSON, or not a species: a key the species lacks, a value of the wrong type, or too many
    /// levels. Schema changes before the first species ships break files (D32), so a stale file
    /// lands here, its error naming the key.
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

impl From<serde_json::Error> for ReadError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

/// The species in a species file, read strictly: a key the species lacks fails, naming it. A field
/// the file leaves out keeps the template's value, the default species with every operator off,
/// so a level's keeps its depth's (`LimbLevels`), and an operator left out is off.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "read from a glTF's extra by Open (C41)")
)]
pub fn read(json: &str, registry: &TypeRegistry) -> Result<Species, ReadError> {
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
    let read = TypedReflectDeserializer::new(registration, registry).deserialize(file)?;
    let mut species = Species::default();
    all_off(&mut species);
    species.try_apply(&*read).map_err(ReadError::Apply)?;
    Ok(species)
}

/// `species` as a species file: `generator`, then each of the species' fields by reflection, in the
/// order they are declared.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "a glTF's extra, written by C41")
)]
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
    ) -> Result<Species, ReadError> {
        let registry = registry();
        let mut file: serde_json::Value =
            serde_json::from_str(&write(species, &registry).unwrap()).unwrap();
        edit(&mut file);
        read(&file.to_string(), &registry)
    }

    #[test]
    fn a_species_reads_back_as_it_was_written() {
        let registry = registry();
        let mut species = Species::default();
        species.limbs.shown = 3;
        species.limbs.levels[2].fork.state = OperatorState::On;
        species.limbs.levels[2].fork.settings.chance = 0.5;
        species.trunk.wander = Toggle::on(Wander { angle: 0.3 });
        species.ornaments.tip_leaves.state = OperatorState::Off;
        let written = write(&species, &registry).unwrap();
        assert!(read(&written, &registry).unwrap() == species);
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
        .unwrap();
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
        let read = edited(&species, |_| {}).unwrap();
        assert!(read == Species::default());
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
        assert_eq!(read.taper, Species::default().taper);
        assert_eq!(read.limbs.levels[1].density, 3.5);
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
        assert!(read(&written, &registry).unwrap() == species);
    }

    #[test]
    fn an_operator_is_read_whole() {
        let mut species = Species::default();
        species.limbs.levels[0].wander.state = OperatorState::On;
        let error = edited(&species, |file| {
            file["limbs"][0]["wander"]
                .as_object_mut()
                .unwrap()
                .remove("angle");
        })
        .err()
        .unwrap();
        assert!(error.to_string().contains("Wander"), "{error}");
    }

    #[test]
    fn a_key_the_species_lacks_fails_naming_it() {
        let error = edited(&Species::default(), |file| {
            file["limbs"][0]["fork"]["wobble"] = 1.into();
        })
        .err()
        .unwrap();
        assert!(error.to_string().contains("wobble"), "{error}");
        let error = edited(&Species::default(), |file| {
            file["roots"] = true.into();
        })
        .err()
        .unwrap();
        assert!(error.to_string().contains("roots"), "{error}");
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
        });
        assert!(matches!(four, Err(ReadError::Json(_))));
        let none = edited(&species, |file| file["limbs"] = serde_json::json!([]));
        assert!(matches!(none, Err(ReadError::Json(_))));
    }
}
