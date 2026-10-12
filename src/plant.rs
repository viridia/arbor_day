//! The plant in the view, rebuilt from the [`Species`] whenever it changes.

use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*,
    render::render_resource::PrimitiveTopology,
};

use crate::{
    generator::{self, Geometry},
    species::Species,
};

pub struct PlantPlugin;

impl Plugin for PlantPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Species>()
            .add_systems(Update, (build_plant, build_envelope));
    }
}

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
    pub triangles: u32,
    /// The bark's vertex color, as sRGB hex.
    pub bark_color: String,
}

fn build_plant(
    mut commands: Commands,
    species: Res<Species>,
    plants: Query<(Entity, Option<&Mesh3d>), With<Plant>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, current) in &plants {
        if current.is_some() && !species.is_changed() {
            continue;
        }
        let geometry = generator::trunk(&species.trunk);
        let color = species.bark.color;
        let stats = PlantStats {
            triangles: geometry.triangles() as u32,
            bark_color: color.to_srgba().to_hex(),
        };
        let mesh = to_mesh(geometry, color);
        set_mesh(&mut commands, &mut meshes, entity, current, mesh);
        commands.entity(entity).insert(stats);
    }
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
        let lines = generator::envelope_outline(&species.envelope);
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

/// A triangle mesh of `geometry`, every vertex colored `color`.
fn to_mesh(geometry: Geometry, color: Color) -> Mesh {
    let positions: Vec<[f32; 3]> = geometry.positions.iter().map(|p| p.to_array()).collect();
    let normals: Vec<[f32; 3]> = geometry.normals.iter().map(|n| n.to_array()).collect();
    // Vertex colors are linear.
    let colors = vec![color.to_linear().to_f32_array(); positions.len()];
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(geometry.indices))
}
