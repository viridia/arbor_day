//! Golden meshes: a hash of the bark and leaf meshes of a few species, so no change to the
//! generator alters its output by accident. A change meant to alter it replaces the hashes with
//! the ones the failure prints, and says why.

use crate::{
    generator::{Geometry, plant},
    species::Species,
};

/// Each species' name, and the hashes of its bark and its leaves.
const GOLDEN: [(&str, u64, u64); 4] = [
    ("default", 0x97d764fdae4015f4, 0x1fa1e381d18767ce),
    ("three levels", 0xeb51bb0b61acd5c4, 0x225bc09cde40a116),
    ("forks", 0xdecb3c844ef1ae33, 0xfcf0dcf0b8c6da72),
    ("three stems", 0xe29e7abf02d0e977, 0x16020666669b0405),
];

fn species(name: &str) -> Species {
    let mut species = Species::default();
    match name {
        "default" => {}
        "three levels" => species.limb_levels = 3,
        "forks" => {
            species.limb_levels = 2;
            species.limbs[0].fork_chance = 0.5;
            species.limbs[1].fork_chance = 0.3;
            species.limbs[1].fork_copies = 3;
        }
        "three stems" => {
            species.trunk.stems = 3;
            species.trunk.splay = 20.0_f32.to_radians();
        }
        _ => unreachable!("{name}"),
    }
    species
}

/// FNV-1a over every bit of the mesh, so the hash is the same with any Rust version, as std's
/// hashers are not promised to be.
fn hash(geometry: &Geometry) -> u64 {
    let floats = (geometry.positions.iter())
        .chain(&geometry.normals)
        .flat_map(|v| v.to_array())
        .chain(geometry.uvs.iter().flat_map(|v| v.to_array()));
    let words = floats
        .map(f32::to_bits)
        .chain(geometry.indices.iter().copied());
    words
        .flat_map(u32::to_le_bytes)
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ byte as u64).wrapping_mul(0x0100_0000_01b3)
        })
}

#[test]
fn meshes_are_unchanged() {
    let hashes = GOLDEN.map(|(name, ..)| {
        let grown = plant(&species(name));
        (name, hash(&grown.bark), hash(&grown.leaves))
    });
    let table: Vec<String> = (hashes.iter())
        .map(|(name, bark, leaves)| format!("    ({name:?}, {bark:#018x}, {leaves:#018x}),"))
        .collect();
    assert!(
        hashes == GOLDEN,
        "meshes changed; if on purpose, `GOLDEN` becomes:\n{}",
        table.join("\n")
    );
}
