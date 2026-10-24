//! The saved file (D34): a glTF of the plant as Panoply loads it (requirements, Output), with the
//! species as a JSON extra on its scene, under [`SPECIES_KEY`]. Beside the `.gltf` go its buffer
//! (`.bin`) and the leaf texture (`.png`), named after it; the asset pipeline packs, mipmaps and
//! converts them.

use std::{collections::BTreeMap, error::Error, fs, io, path::Path};

use bevy::{prelude::*, reflect::TypeRegistry};
use gltf_json::{
    Accessor, Index, Root, Value,
    accessor::{ComponentType, GenericComponentType, Type},
    buffer::{self, Target},
    extras::RawValue,
    image::MimeType,
    material::{AlphaCutoff, AlphaMode, PbrMetallicRoughness, StrengthFactor},
    mesh::{Mode, Primitive, Semantic},
    texture::{self, MagFilter, MinFilter, WrappingMode},
    validation::{Checked::Valid, USize64},
};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

use crate::{
    generator::{self, Geometry, LeafTexture},
    plant::LEAF_CUTOFF,
    species::Species,
    species_file,
};

/// The scene extra holding the species file.
pub const SPECIES_KEY: &str = "arbor_day";

/// What a URI escapes in a file name: all but letters, digits, `.`, `-` and `_`.
const URI_ESCAPED: &AsciiSet = &NON_ALPHANUMERIC.remove(b'.').remove(b'-').remove(b'_');

/// Writes `species`, grown, to `path` as a glTF named `name`, with its buffer and leaf texture
/// beside it.
pub fn write(
    path: &Path,
    name: &str,
    species: &Species,
    registry: &TypeRegistry,
) -> Result<(), Box<dyn Error>> {
    let grown = generator::plant(species);
    let bin = path.with_extension("bin");
    let png = path.with_extension("png");
    let mut file = File::default();

    let bark_color = species.bark_color.to_linear().to_f32_array_no_alpha();
    let colors = vec![bark_color; grown.bark.positions.len()];
    let bark_material = file.root.push(gltf_json::Material {
        name: Some("bark".into()),
        pbr_metallic_roughness: matte(None),
        extras: species
            .outline
            .then(|| raw(&serde_json::json!({ "outline": species.bark_outline })))
            .transpose()?,
        ..default()
    });
    let mut nodes = vec![file.mesh("bark", &grown.bark, Some(&colors), bark_material)];

    if !grown.leaves.indices.is_empty() {
        let texture = file.texture(&uri(&png));
        let leaf_material = file.root.push(gltf_json::Material {
            name: Some("leaves".into()),
            pbr_metallic_roughness: matte(Some(texture)),
            alpha_mode: Valid(AlphaMode::Mask),
            alpha_cutoff: Some(AlphaCutoff(LEAF_CUTOFF)),
            double_sided: true,
            ..default()
        });
        nodes.push(file.mesh("leaves", &grown.leaves, None, leaf_material));
    }

    let species_json: Value = serde_json::from_str(&species_file::write(species, registry)?)?;
    let scene = file.root.push(gltf_json::Scene {
        extensions: None,
        extras: Some(raw(&serde_json::json!({ SPECIES_KEY: species_json }))?),
        name: Some(name.into()),
        nodes,
    });
    file.root.scene = Some(scene);
    file.root.push(gltf_json::Buffer {
        byte_length: USize64::from(file.data.len()),
        name: None,
        uri: Some(uri(&bin)),
        extensions: None,
        extras: None,
    });

    fs::write(&bin, &file.data)?;
    if !grown.leaves.indices.is_empty() {
        write_png(&png, &grown.leaf_texture)?;
    }
    fs::write(path, file.root.to_string_pretty()?)?;
    Ok(())
}

/// The species saved in the glTF at `path`, and what of it was pruned (`species_file::read`).
pub fn read(path: &Path, registry: &TypeRegistry) -> Result<species_file::Read, Box<dyn Error>> {
    let root = Root::from_slice(&fs::read(path)?)?;
    let scene = root.scene.map_or(0, |scene| scene.value());
    let extras = (root.scenes.get(scene))
        .and_then(|scene| scene.extras.as_ref())
        .ok_or("not saved by arbor_day: its scene has no extras")?;
    let extras: Value = serde_json::from_str(extras.get())?;
    let species = extras
        .get(SPECIES_KEY)
        .ok_or_else(|| format!("not saved by arbor_day: no `{SPECIES_KEY}` extra"))?;
    Ok(species_file::read(&species.to_string(), registry)?)
}

/// A glTF being built, and its buffer's bytes.
#[derive(Default)]
struct File {
    root: Root,
    data: Vec<u8>,
}

impl File {
    /// A node drawing `geometry` in `material`, named `name`, its vertices colored `colors` if
    /// given.
    fn mesh(
        &mut self,
        name: &str,
        geometry: &Geometry,
        colors: Option<&[[f32; 3]]>,
        material: Index<gltf_json::Material>,
    ) -> Index<gltf_json::Node> {
        let positions: Vec<[f32; 3]> = geometry.positions.iter().map(|p| p.to_array()).collect();
        let (min, max) = geometry.positions.iter().fold(
            (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
            |(min, max), &p| (min.min(p), max.max(p)),
        );
        let mut attributes = BTreeMap::new();
        let position = self.vertices(&positions, Type::Vec3);
        self.root.accessors[position.value()].min = Some(serde_json::json!(min.to_array()));
        self.root.accessors[position.value()].max = Some(serde_json::json!(max.to_array()));
        attributes.insert(Valid(Semantic::Positions), position);
        let normals: Vec<[f32; 3]> = geometry.normals.iter().map(|n| n.to_array()).collect();
        let normal = self.vertices(&normals, Type::Vec3);
        attributes.insert(Valid(Semantic::Normals), normal);
        if let Some(colors) = colors {
            let color = self.vertices(colors, Type::Vec3);
            attributes.insert(Valid(Semantic::Colors(0)), color);
        }
        if !geometry.uvs.is_empty() {
            let uvs: Vec<[f32; 2]> = geometry.uvs.iter().map(|uv| uv.to_array()).collect();
            let uv = self.vertices(&uvs, Type::Vec2);
            attributes.insert(Valid(Semantic::TexCoords(0)), uv);
        }
        let index_bytes = geometry
            .indices
            .iter()
            .flat_map(|i| i.to_le_bytes())
            .collect();
        let indices = self.accessor(
            index_bytes,
            geometry.indices.len(),
            Type::Scalar,
            ComponentType::U32,
            Target::ElementArrayBuffer,
        );
        let mesh = self.root.push(gltf_json::Mesh {
            extensions: None,
            extras: None,
            name: Some(name.into()),
            primitives: vec![Primitive {
                attributes,
                extensions: None,
                extras: None,
                indices: Some(indices),
                material: Some(material),
                mode: Valid(Mode::Triangles),
                targets: None,
            }],
            weights: None,
        });
        self.root.push(gltf_json::Node {
            mesh: Some(mesh),
            name: Some(name.into()),
            ..default()
        })
    }

    /// An accessor of one float vector per vertex.
    fn vertices<const N: usize>(&mut self, items: &[[f32; N]], type_: Type) -> Index<Accessor> {
        let bytes = items
            .iter()
            .flatten()
            .flat_map(|x| x.to_le_bytes())
            .collect();
        self.accessor(
            bytes,
            items.len(),
            type_,
            ComponentType::F32,
            Target::ArrayBuffer,
        )
    }

    /// An accessor of `count` items in `bytes`, appended to the buffer in a view of their own.
    /// Every component is four bytes, so each view starts aligned.
    fn accessor(
        &mut self,
        bytes: Vec<u8>,
        count: usize,
        type_: Type,
        component: ComponentType,
        target: Target,
    ) -> Index<Accessor> {
        let view = self.root.push(buffer::View {
            buffer: Index::new(0),
            byte_length: USize64::from(bytes.len()),
            byte_offset: Some(USize64::from(self.data.len())),
            byte_stride: None,
            name: None,
            target: Some(Valid(target)),
            extensions: None,
            extras: None,
        });
        self.data.extend_from_slice(&bytes);
        self.root.push(Accessor {
            buffer_view: Some(view),
            byte_offset: None,
            count: USize64::from(count),
            component_type: Valid(GenericComponentType(component)),
            extensions: None,
            extras: None,
            type_: Valid(type_),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        })
    }

    /// A texture of the image at `uri`, filtered for the mipmaps the pipeline adds.
    fn texture(&mut self, uri: &str) -> Index<gltf_json::Texture> {
        let sampler = self.root.push(texture::Sampler {
            mag_filter: Some(Valid(MagFilter::Linear)),
            min_filter: Some(Valid(MinFilter::LinearMipmapLinear)),
            wrap_s: Valid(WrappingMode::ClampToEdge),
            wrap_t: Valid(WrappingMode::ClampToEdge),
            ..default()
        });
        let source = self.root.push(gltf_json::Image {
            buffer_view: None,
            mime_type: Some(MimeType("image/png".into())),
            name: None,
            uri: Some(uri.into()),
            extensions: None,
            extras: None,
        });
        self.root.push(gltf_json::Texture {
            name: None,
            sampler: Some(sampler),
            source,
            extensions: None,
            extras: None,
        })
    }
}

/// Matte and not metallic, as the preview's materials are, colored by `texture` if given.
fn matte(texture: Option<Index<gltf_json::Texture>>) -> PbrMetallicRoughness {
    PbrMetallicRoughness {
        base_color_texture: texture.map(|index| texture::Info {
            index,
            tex_coord: 0,
            extensions: None,
            extras: None,
        }),
        metallic_factor: StrengthFactor(0.0),
        roughness_factor: StrengthFactor(1.0),
        ..default()
    }
}

/// `value` as a glTF extra.
fn raw(value: &Value) -> Result<Box<RawValue>, serde_json::Error> {
    RawValue::from_string(value.to_string())
}

/// The URI of a file beside the glTF: its name, escaped.
fn uri(path: &Path) -> String {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    utf8_percent_encode(&name, URI_ESCAPED).to_string()
}

/// The leaf texture's full-size level as an sRGB PNG.
fn write_png(path: &Path, texture: &LeafTexture) -> Result<(), Box<dyn Error>> {
    let file = io::BufWriter::new(fs::File::create(path)?);
    let mut encoder = png::Encoder::new(file, texture.size, texture.size);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    encoder.write_header()?.write_image_data(&texture.pixels)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_species_is_valid_gltf_and_reads_back() {
        let mut registry = TypeRegistry::new();
        registry.register::<Species>();
        let dir = std::env::temp_dir().join(format!("arbor_day_gltf_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("oak.gltf");
        let species = Species::default();

        write(&path, "oak", &species, &registry).unwrap();
        let gltf = gltf::Gltf::from_slice(&fs::read(&path).unwrap()).unwrap();
        let bin = fs::metadata(dir.join("oak.bin")).unwrap().len() as usize;
        assert_eq!(gltf.buffers().next().unwrap().length(), bin);
        assert_eq!(gltf.meshes().count(), 2, "bark and leaves");
        assert!(read(&path, &registry).unwrap().species == species);
        fs::remove_dir_all(&dir).unwrap();
    }

    /// The author's finished trees in `reference/`, each added when done (G6).
    const FINISHED: &[&str] = &[
        "trees-temperate/poplar-orange",
        "crops/palecress",
        "trees-coniferous/pine-fluffy",
        "trees-coniferous/pine-drooping",
        "trees-temperate/hemlock1",
    ];

    /// The most a position or texture coordinate may move, in metres or UV units, and a normal.
    const DISTANCE: f32 = 1e-4;
    const NORMAL: f32 = 1e-3;

    /// The most a texture channel may differ: SIMD may round differently.
    const CHANNEL: u8 = 2;

    #[test]
    fn finished_trees_grow_as_saved() {
        let mut registry = TypeRegistry::new();
        registry.register::<Species>();
        let failures: Vec<String> = FINISHED
            .iter()
            .flat_map(|tree| {
                golden_differences(tree, &registry)
                    .unwrap_or_else(|error| vec![error.to_string()])
                    .into_iter()
                    .map(move |difference| format!("{tree}: {difference}"))
            })
            .collect();
        assert!(
            failures.is_empty(),
            "finished trees grow unlike their saved meshes; look at each, then save it again, \
             retune it, or put the change behind an option:\n{}",
            failures.join("\n")
        );
    }

    /// How the tree at `reference/<tree>.gltf`, grown from its species, differs from its saved
    /// meshes and leaf texture.
    fn golden_differences(
        tree: &str,
        registry: &TypeRegistry,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let path = Path::new("reference").join(tree).with_extension("gltf");
        let grown = generator::plant(&read(&path, registry)?.species);
        let gltf = gltf::Gltf::from_slice(&fs::read(&path)?)?;
        let bin = fs::read(path.with_extension("bin"))?;
        let mut differences = Vec::new();
        for (name, geometry) in [("bark", &grown.bark), ("leaves", &grown.leaves)] {
            let saved = gltf.meshes().find(|mesh| mesh.name() == Some(name));
            let Some(primitive) = saved.and_then(|mesh| mesh.primitives().next()) else {
                if !geometry.indices.is_empty() {
                    differences.push(format!("{name}: grown, but none saved"));
                }
                continue;
            };
            let reader = primitive.reader(|_| Some(&bin));
            let indices: Vec<u32> = reader
                .read_indices()
                .ok_or("no indices")?
                .into_u32()
                .collect();
            if indices != geometry.indices {
                differences.push(format!(
                    "{name}: {} triangles saved, {} grown, or joined differently",
                    indices.len() / 3,
                    geometry.indices.len() / 3
                ));
                continue;
            }
            let positions: Vec<Vec3> = reader
                .read_positions()
                .ok_or("no positions")?
                .map(Vec3::from)
                .collect();
            let normals: Vec<Vec3> = reader
                .read_normals()
                .ok_or("no normals")?
                .map(Vec3::from)
                .collect();
            let uvs: Vec<Vec3> = (reader.read_tex_coords(0))
                .map(|uvs| {
                    uvs.into_f32()
                        .map(|uv| Vec2::from(uv).extend(0.0))
                        .collect()
                })
                .unwrap_or_default();
            let grown_uvs: Vec<Vec3> = geometry.uvs.iter().map(|uv| uv.extend(0.0)).collect();
            differences.extend(
                [
                    furthest(name, "positions", &positions, &geometry.positions, DISTANCE),
                    furthest(name, "normals", &normals, &geometry.normals, NORMAL),
                    furthest(name, "uvs", &uvs, &grown_uvs, DISTANCE),
                ]
                .into_iter()
                .flatten(),
            );
        }
        if !grown.leaves.indices.is_empty() {
            let decoder = png::Decoder::new(io::BufReader::new(fs::File::open(
                path.with_extension("png"),
            )?));
            let mut reader = decoder.read_info()?;
            let mut pixels = vec![0; reader.output_buffer_size().ok_or("too large")?];
            let info = reader.next_frame(&mut pixels)?;
            pixels.truncate(info.buffer_size());
            let painted = &grown.leaf_texture.pixels;
            if pixels.len() != painted.len() {
                differences.push(format!(
                    "leaf texture: {}×{} saved, {}×{} painted",
                    info.width, info.height, grown.leaf_texture.size, grown.leaf_texture.size
                ));
            } else if let Some(most) = pixels
                .iter()
                .zip(painted)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .filter(|&most| most > CHANNEL)
            {
                differences.push(format!("leaf texture: a channel differs by {most}"));
            }
        }
        Ok(differences)
    }

    /// Where `grown` strays furthest from `saved`, if by more than `tolerance`.
    fn furthest(
        mesh: &str,
        attribute: &str,
        saved: &[Vec3],
        grown: &[Vec3],
        tolerance: f32,
    ) -> Option<String> {
        if saved.len() != grown.len() {
            return Some(format!(
                "{mesh}: {} {attribute} saved, {} grown",
                saved.len(),
                grown.len()
            ));
        }
        let (vertex, distance) = (saved.iter().zip(grown))
            .map(|(a, b)| a.distance(*b))
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(&b.1))?;
        (distance > tolerance)
            .then(|| format!("{mesh}: {attribute} moved by up to {distance}, at vertex {vertex}"))
    }
}
