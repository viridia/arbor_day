//! The bark's outline in the view, drawn as the game draws it: Panoply's `OutlineMaterial`, an
//! inverted hull over the bark's mesh.

use bevy::{
    asset::embedded_asset,
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, Face, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

use crate::{plant::Plant, species::Species};

pub struct OutlinePlugin;

impl Plugin for OutlinePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "outline.wesl");
        app.add_plugins(MaterialPlugin::<OutlineMaterial>::default())
            .add_observer(add_outline_material)
            .add_systems(Update, build_outline);
    }
}

/// The entity drawing the bark's outline, sharing the [`Plant`]'s mesh, shown while the species is
/// outlined.
#[derive(Component, Default, Clone)]
pub struct PlantOutline;

/// The mesh drawn again, pushed out along its normals by `width`, with front faces culled, so only
/// the rim behind the original shows. Unlit, and kept out of the depth prepass and shadow maps.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct OutlineMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    /// In the mesh's own units.
    #[uniform(1)]
    pub width: f32,
}

impl Material for OutlineMaterial {
    fn vertex_shader() -> ShaderRef {
        "embedded://arbor_day/outline.wesl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://arbor_day/outline.wesl".into()
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Face::Front);
        Ok(())
    }
}

fn add_outline_material(
    add: On<Add<PlantOutline>>,
    mut commands: Commands,
    species: Res<Species>,
    mut materials: ResMut<Assets<OutlineMaterial>>,
) {
    let material = OutlineMaterial {
        color: LinearRgba::BLACK,
        width: species.bark_outline,
    };
    commands
        .entity(add.event_target())
        .insert(MeshMaterial3d(materials.add(material)));
}

/// Gives the outline the plant's mesh once it has one, and follows the species' switch and width.
/// The plant's mesh is replaced in place (`plant::set_mesh`), so the shared handle stays right.
fn build_outline(
    mut commands: Commands,
    species: Res<Species>,
    plant: Single<&Mesh3d, With<Plant>>,
    outlines: Query<
        (Entity, Option<&Mesh3d>, &MeshMaterial3d<OutlineMaterial>),
        With<PlantOutline>,
    >,
    mut materials: ResMut<Assets<OutlineMaterial>>,
) {
    for (entity, current, material) in &outlines {
        if current.is_some() && !species.is_changed() {
            continue;
        }
        if let Some(mut material) = materials.get_mut(&material.0) {
            material.width = species.bark_outline;
        }
        let visibility = if species.outline {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        commands
            .entity(entity)
            .insert((Mesh3d(plant.0.clone()), visibility));
    }
}
