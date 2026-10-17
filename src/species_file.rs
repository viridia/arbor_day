//! The species file (`docs/patches.md`, The file): the species written by reflection as JSON,
//! after the version of the generator it was made with.

use bevy::reflect::{TypeRegistry, serde::TypedReflectSerializer, structs::Struct};
use serde::ser::{Serialize, SerializeMap, Serializer};

use crate::species::Species;

/// The generator's version, raised when output changes on purpose outside an operator (X15).
pub const GENERATOR: u32 = 1;

/// `species` as a species file: `generator`, then each of the species' fields by reflection, in the
/// order they are declared.
#[cfg_attr(not(test), expect(dead_code, reason = "saved by C40 and exported by C41"))]
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
}
