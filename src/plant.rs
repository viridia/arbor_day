//! The plant in the view, rebuilt from the [`Species`] whenever it changes.

use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

use crate::{
    generator::{self, CARD_CORNERS, Geometry},
    species::Species,
};

/// The leaf texture until the species paints its own (ledger C38): seedling's, from
/// `poplar-green.glb`.
const LEAF_TEXTURE: &str = "leaf-placeholder.png";

pub struct PlantPlugin;

impl Plugin for PlantPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Species>()
            .add_observer(add_leaf_material)
            .add_systems(Update, (build_plant, build_envelope));
    }
}

/// The entity showing the plant's leaf cards, beside the [`Plant`]. Its mesh is added on the
/// first build that grows any, and hidden, not removed, while none grow.
#[derive(Component, Default, Clone)]
pub struct PlantLeaves;

/// The entity drawing the crown's envelope as lines. Its mesh is added on the first build.
#[derive(Component, Default, Clone)]
pub struct PlantEnvelope;

/// Figures for the envelope as last drawn, for plans.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct EnvelopeStats {
    pub segments: u32,
}

/// The entity showing the generated plant. Its mesh is added on the first build.
#[derive(Component, Default, Clone)]
pub struct Plant;

/// Figures for the plant as last built, for the panel and for plans.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct PlantStats {
    /// The bark's and the leaves'.
    pub triangles: u32,
    pub limbs: u32,
    /// Leaf cards: one at each leafy tip, or more where a steep tip bears a rosette.
    pub cards: u32,
    /// Branch ends bearing leaves.
    pub leafy_tips: u32,
    /// Limbs on each grown level, first to last.
    pub level_limbs: Vec<u32>,
    /// The bark's vertex color, as sRGB hex.
    pub bark_color: String,
}

/// The species the plant was last built from, for plans, since the driver reads components, not
/// resources.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct GrownFrom(pub Species);

fn build_plant(
    mut commands: Commands,
    species: Res<Species>,
    plants: Query<(Entity, Option<&Mesh3d>), With<Plant>>,
    leaves: Query<(Entity, Option<&Mesh3d>), With<PlantLeaves>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, current) in &plants {
        if current.is_some() && !species.is_changed() {
            continue;
        }
        let grown = generator::plant(&species);
        let color = species.bark_color;
        let level_limbs = &grown.level_limbs;
        let stats = PlantStats {
            triangles: (grown.bark.triangles() + grown.leaves.triangles()) as u32,
            limbs: level_limbs.iter().sum::<usize>() as u32,
            cards: (grown.leaves.positions.len() / CARD_CORNERS) as u32,
            leafy_tips: grown.leafy_tips as u32,
            level_limbs: level_limbs.iter().map(|&count| count as u32).collect(),
            bark_color: color.to_srgba().to_hex(),
        };
        let mesh = to_mesh(grown.bark, color);
        set_mesh(&mut commands, &mut meshes, entity, current, mesh);
        commands
            .entity(entity)
            .insert((stats, GrownFrom(species.clone())));
        let leaf_mesh =
            (!grown.leaves.indices.is_empty()).then(|| to_mesh(grown.leaves, Color::WHITE));
        for (entity, current) in &leaves {
            match &leaf_mesh {
                Some(mesh) => {
                    set_mesh(&mut commands, &mut meshes, entity, current, mesh.clone());
                    commands.entity(entity).insert(Visibility::Inherited);
                }
                // Hidden, keeping the last mesh: an empty mesh has no bounds to cull by, and a
                // `Mesh3d` removed and inserted again was never drawn.
                None => {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }
        }
    }
}

/// The leaves' material: the leaf texture, cut out where it is transparent, and both faces drawn.
fn add_leaf_material(
    add: On<Add<PlantLeaves>>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = StandardMaterial {
        base_color_texture: Some(assets.load(LEAF_TEXTURE)),
        alpha_mode: AlphaMode::Mask(0.5),
        double_sided: true,
        cull_mode: None,
        perceptual_roughness: 1.0,
        ..default()
    };
    commands
        .entity(add.event_target())
        .insert(MeshMaterial3d(materials.add(material)));
}

fn build_envelope(
    mut commands: Commands,
    species: Res<Species>,
    envelopes: Query<(Entity, Option<&Mesh3d>), With<PlantEnvelope>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, current) in &envelopes {
        if current.is_some() && !species.is_changed() {
            continue;
        }
        let lines = generator::envelope_helper(&generator::Envelope::of(&species));
        let stats = EnvelopeStats {
            segments: (lines.len() / 2) as u32,
        };
        let positions: Vec<[f32; 3]> = lines.iter().map(|p| p.to_array()).collect();
        let mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        set_mesh(&mut commands, &mut meshes, entity, current, mesh);
        commands.entity(entity).insert(stats);
    }
}

/// Gives `entity` the mesh, replacing its current one in place: the handle stays, and the
/// entity's bounds follow the new mesh, so the turntable keeps it framed.
fn set_mesh(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    entity: Entity,
    current: Option<&Mesh3d>,
    mesh: Mesh,
) {
    match current {
        Some(Mesh3d(handle)) => {
            let _ = meshes.insert(handle, mesh);
        }
        None => {
            commands.entity(entity).insert(Mesh3d(meshes.add(mesh)));
        }
    }
}

/// A triangle mesh of `geometry`, every vertex colored `color`, with its texture coordinates if it
/// has them.
fn to_mesh(geometry: Geometry, color: Color) -> Mesh {
    let positions: Vec<[f32; 3]> = geometry.positions.iter().map(|p| p.to_array()).collect();
    let normals: Vec<[f32; 3]> = geometry.normals.iter().map(|n| n.to_array()).collect();
    // Vertex colors are linear.
    let colors = vec![color.to_linear().to_f32_array(); positions.len()];
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(geometry.indices));
    if !geometry.uvs.is_empty() {
        let uvs: Vec<[f32; 2]> = geometry.uvs.iter().map(|uv| uv.to_array()).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    }
    mesh
}
