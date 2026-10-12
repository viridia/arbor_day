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
            .add_systems(Update, build_plant);
    }
}

/// The entity showing the generated plant. Its mesh is added on the first build.
#[derive(Component, Default, Clone)]
pub struct Plant;

/// Figures for the plant as last built, for the panel and for plans.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct PlantStats {
    pub triangles: u32,
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
        let stats = PlantStats {
            triangles: geometry.triangles() as u32,
        };
        let mesh = to_mesh(geometry);
        match current {
            // Replaced in place: the handle stays, and the entity's bounds follow the new mesh, so
            // the turntable keeps it framed.
            Some(Mesh3d(handle)) => {
                let _ = meshes.insert(handle, mesh);
            }
            None => {
                commands.entity(entity).insert(Mesh3d(meshes.add(mesh)));
            }
        }
        commands.entity(entity).insert(stats);
    }
}

fn to_mesh(geometry: Geometry) -> Mesh {
    let positions: Vec<[f32; 3]> = geometry.positions.iter().map(|p| p.to_array()).collect();
    let normals: Vec<[f32; 3]> = geometry.normals.iter().map(|n| n.to_array()).collect();
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(Indices::U32(geometry.indices))
}
