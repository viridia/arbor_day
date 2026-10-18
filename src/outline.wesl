// Outline by normal extrusion (inverted hull), as Panoply's `src/materials/outline.wesl`. The mesh is
// drawn again, pushed out along its normals; the material culls front faces, so only the expanded
// back faces show, as a rim.
import bevy_pbr::render::{
    mesh_functions,
    forward_io::{Vertex, decompress_vertex},
    view_transformations::position_world_to_clip,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0)
var<uniform> color: vec4<f32>;

@group(constants::MATERIAL_BIND_GROUP) @binding(1)
var<uniform> width: f32;

@vertex
fn vertex(vertex_in: Vertex) -> @builtin(position) vec4<f32> {
    let vertex = decompress_vertex(vertex_in, vertex_in.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(vertex_in.instance_index);
    // Extruded in the mesh's own space, so the line scales with the plant, as in the game.
    let local_position = vec4<f32>(vertex.position + vertex.normal * width, 1.0);
    let world_position = mesh_functions::mesh_position_local_to_world(world_from_local, local_position);
    return position_world_to_clip(world_position.xyz);
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return color;
}
