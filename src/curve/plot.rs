//! The path material: a UI material that strokes a plot's [`CurvePath`] in a theme color.

use bevy::{
    asset::embedded_asset,
    feathers::{
        theme::{ThemeToken, UiTheme},
        tokens,
    },
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    ui_render::prelude::{MaterialNode, UiMaterial, UiMaterialPlugin},
};

use super::view::{CurvePath, MAX_PATH_POINTS};

/// The most lines a path material strokes. Must match `MAX_LINES` in `plot.wesl`.
const MAX_LINES: usize = MAX_PATH_POINTS - 1;

/// Strokes a polyline of up to [`MAX_PATH_POINTS`] points, held in a uniform array so it draws on
/// WebGL2 as well as WebGPU. Every node with a [`CurvePath`] and a [`CurveStroke`] gets one, kept
/// up to date, so it is never made by hand.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct PathMaterial {
    #[uniform(0)]
    path: PathUniform,
}

#[derive(ShaderType, Debug, Clone)]
struct PathUniform {
    color: Vec4,
    /// In logical pixels.
    width: f32,
    /// The inverse of the node's scale factor.
    scale: f32,
    count: u32,
    /// Each line as `(a.x, a.y, b.x, b.y)`, in logical pixels from the node's top left; the first
    /// `count` are drawn.
    lines: [Vec4; MAX_LINES],
}

impl UiMaterial for PathMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://arbor_day/curve/plot.wesl".into()
    }
}

/// How a plot's [`CurvePath`] is stroked: by default, in the main text color, 1.5 pixels wide.
#[derive(Component, Clone, Debug)]
pub struct CurveStroke {
    pub color: ThemeToken,
    /// In logical pixels.
    pub width: f32,
}

impl Default for CurveStroke {
    fn default() -> Self {
        Self {
            color: tokens::TEXT_MAIN,
            width: 1.5,
        }
    }
}

pub(super) struct PathMaterialPlugin;

impl Plugin for PathMaterialPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "plot.wesl");
        app.add_plugins(UiMaterialPlugin::<PathMaterial>::default())
            .add_systems(
                PostUpdate,
                draw_paths.after(super::view::update_curve_paths),
            );
    }
}

/// Gives each stroked path a material, and rewrites its lines, color and scale when they change.
fn draw_paths(
    mut commands: Commands,
    paths: Query<(
        Entity,
        Ref<CurvePath>,
        Ref<CurveStroke>,
        Ref<ComputedNode>,
        Option<&MaterialNode<PathMaterial>>,
    )>,
    theme: Res<UiTheme>,
    mut materials: ResMut<Assets<PathMaterial>>,
) {
    for (entity, path, stroke, node, material) in &paths {
        let changed = path.is_changed() || stroke.is_changed() || node.is_changed();
        if !(changed || theme.is_changed()) {
            continue;
        }
        let mut uniform = PathUniform {
            color: theme.color(&stroke.color).to_linear().to_vec4(),
            width: stroke.width,
            scale: node.inverse_scale_factor(),
            count: 0,
            lines: [Vec4::ZERO; MAX_LINES],
        };
        for (line, pair) in uniform.lines.iter_mut().zip(path.0.windows(2)) {
            *line = Vec4::new(pair[0].x, pair[0].y, pair[1].x, pair[1].y);
            uniform.count += 1;
        }
        match material {
            Some(material) => {
                if let Some(mut material) = materials.get_mut(&material.0) {
                    material.path = uniform;
                }
            }
            None => {
                let material = materials.add(PathMaterial { path: uniform });
                commands.entity(entity).insert(MaterialNode(material));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_shader_holds_as_many_lines() {
        let declared = format!("const MAX_LINES: u32 = {}u;", super::MAX_LINES);
        assert!(include_str!("plot.wesl").contains(&declared));
    }
}
