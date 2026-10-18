//! The leaf texture (D29): a sprig of leaves painted with `tiny-skia` (D20), after seedling's.
//! Lengths are in leaf lengths, the middle leaf's, until fitted to the texture.

use bevy::{
    color::{Color, Oklcha, Srgba},
    math::Vec2,
};
use tiny_skia::{
    FillRule, GradientStop, LineCap, LineJoin, LinearGradient, Paint, Path, PathBuilder, Pixmap,
    Point, Shader, SpreadMode, Stroke, Transform,
};

use crate::{
    random::Seed,
    species::{Gradient, Leaf, Sprig, TipLeaves, limits},
};

/// The texture's width and height, in pixels.
pub const TEXTURE_SIZE: u32 = 256;

/// The painted stem's width.
const STEM_WIDTH: f32 = 0.06;

/// The most a leaf's hue turns, in degrees, and its lightness shifts, in Oklch, at a variation of 1.
const HUE_SHIFT: f32 = 144.0;
const LIGHTNESS_SHIFT: f32 = 0.4;

/// The alpha of the translucent stroke parting leaves within the sprig.
const PARTING_ALPHA: u8 = 0x44;

/// The leaf texture: square, sRGB, alpha not premultiplied, rows from the top. Its bottom edge is
/// the card's base.
#[derive(Default)]
pub struct LeafTexture {
    pub size: u32,
    pub pixels: Vec<u8>,
}

impl std::fmt::Debug for LeafTexture {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "LeafTexture({0}×{0})", self.size)
    }
}

/// A cubic Bézier from its first point to its last, pulled toward the two between.
type Segment = [Vec2; 4];

/// One leaf of the sprig: turned from the stem by `angle`, scaled, and moved down the stem by
/// `drop`, from the fan's start.
struct Stamp {
    angle: f32,
    scale: f32,
    drop: f32,
}

/// The sprig laid out and fitted to the texture.
pub(super) struct Layout {
    /// The leaf's right half.
    half: Vec<Segment>,
    stamps: Vec<Stamp>,
    /// From leaf lengths to texture pixels: the stem on the vertical midline, the tip up.
    to_pixels: Transform,
    /// Texture pixels per leaf length.
    scale: f32,
    /// Where the fan starts, as a share of the texture's height from its bottom edge.
    pub fold: f32,
}

/// Lays out the sprig of `tip_leaves` for `seed`, fitted to a texture outlined `outline` pixels
/// wide: square, the stem on its vertical midline, every leaf and its outline inside.
pub(super) fn layout(tip_leaves: &TipLeaves, seed: Seed, outline: Option<f32>) -> Layout {
    let half = half_outline(&tip_leaves.leaf, seed);
    let stamps = stamps(&tip_leaves.sprig, seed);
    let path = leaf_path(&half);
    // Leaf y runs up, so a rect's top is its lowest edge.
    let (mut half_width, mut bottom, mut top) = (0.0_f32, 0.0_f32, 0.0_f32);
    for stamp in &stamps {
        let placed = path.clone().transform(stamp_transform(stamp));
        let Some(rect) = placed.and_then(|placed| placed.compute_tight_bounds()) else {
            continue;
        };
        // Symmetric about the stem, so it lies on the card's midrib.
        half_width = half_width.max(rect.left().abs()).max(rect.right().abs());
        bottom = bottom.min(rect.top());
        top = top.max(rect.bottom());
    }
    let extent = (2.0 * half_width).max(top - bottom).max(f32::EPSILON);
    let middle = (bottom + top) * 0.5;
    let size = TEXTURE_SIZE as f32;
    let padding = outline.unwrap_or(0.0) * 0.5 + 1.0;
    let scale = (size - 2.0 * padding) / extent;
    let to_pixels = Transform::from_row(scale, 0.0, 0.0, -scale, size * 0.5, size * 0.5 + middle * scale);
    Layout {
        half,
        stamps,
        to_pixels,
        scale,
        fold: 0.5 - middle * scale / size,
    }
}

/// Paints the sprig `layout` lays out, with the leaf colors of `tip_leaves`, its stem in
/// `bark_color`, for `seed`. With an outline, every leaf and the stem are first stroked black
/// that many pixels wide, and each leaf, filled in turn, is stroked translucent, so the black shows
/// only around the sprig and faint lines part the leaves within it.
pub(super) fn paint(
    layout: &Layout,
    tip_leaves: &TipLeaves,
    bark_color: Color,
    seed: Seed,
    outline: Option<f32>,
) -> LeafTexture {
    let leaf = &tip_leaves.leaf;
    let mut pixmap = Pixmap::new(TEXTURE_SIZE, TEXTURE_SIZE).expect("the texture has a size");
    let path = leaf_path(&layout.half);
    let stem = stem_path(layout);
    let stem_width = STEM_WIDTH * layout.scale;
    let leaf_transform = |stamp: &Stamp| layout.to_pixels.pre_concat(stamp_transform(stamp));
    // Widths are given in pixels and drawn through the transform, which scales them.
    let stroke = |pixels: f32, scale: f32| Stroke {
        width: pixels / (layout.scale * scale),
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    };
    let solid = |color: tiny_skia::Color| Paint {
        shader: Shader::SolidColor(color),
        anti_alias: true,
        ..Paint::default()
    };

    if let Some(width) = outline {
        let black = solid(tiny_skia::Color::BLACK);
        pixmap.stroke_path(&stem, &black, &stroke(stem_width + width, 1.0), layout.to_pixels, None);
        for stamp in &layout.stamps {
            let transform = leaf_transform(stamp);
            pixmap.stroke_path(&path, &black, &stroke(width, stamp.scale), transform, None);
        }
    }
    let bark = solid(skia_color(bark_color));
    pixmap.stroke_path(&stem, &bark, &stroke(stem_width, 1.0), layout.to_pixels, None);

    let (width, length) = layout.half.iter().flatten().fold((0.0_f32, 0.0_f32), |(w, l), p| {
        (w.max(p.x), l.max(p.y))
    });
    let mut rng = seed.stream("leaf color");
    let parting = solid(tiny_skia::Color::from_rgba8(0, 0, 0, PARTING_ALPHA));
    for stamp in &layout.stamps {
        let hue = leaf.variation * HUE_SHIFT * (2.0 * rng.unit() - 1.0);
        let lightness = leaf.variation * LIGHTNESS_SHIFT * (2.0 * rng.unit() - 1.0);
        let inner = shifted(leaf.inner_color, hue, lightness);
        let outer = shifted(leaf.outer_color, hue, lightness);
        let (start, end, stops) = match leaf.gradient {
            Gradient::Axial => (
                Point::from_xy(0.0, 0.0),
                Point::from_xy(0.0, length),
                vec![GradientStop::new(0.0, inner), GradientStop::new(1.0, outer)],
            ),
            Gradient::Lateral => (
                Point::from_xy(-0.6 * width, 0.0),
                Point::from_xy(0.6 * width, 0.0),
                vec![
                    GradientStop::new(0.0, outer),
                    GradientStop::new(0.5, inner),
                    GradientStop::new(1.0, outer),
                ],
            ),
        };
        let shader = LinearGradient::new(start, end, stops, SpreadMode::Pad, Transform::identity())
            .unwrap_or(Shader::SolidColor(inner));
        let fill = Paint {
            shader,
            anti_alias: true,
            ..Paint::default()
        };
        let transform = leaf_transform(stamp);
        if let Some(width) = outline {
            pixmap.stroke_path(&path, &parting, &stroke(width, stamp.scale), transform, None);
        }
        pixmap.fill_path(&path, &fill, FillRule::Winding, transform, None);
    }

    let pixels = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let color = pixel.demultiply();
            [color.red(), color.green(), color.blue(), color.alpha()]
        })
        .collect();
    LeafTexture {
        size: TEXTURE_SIZE,
        pixels,
    }
}

/// The leaf's right half from base to tip: a curve from the base, (0, 0), to the tip, (0, 1),
/// pulled out by the widths and along by the tapers, cut into the leaf's segments. Each segment
/// but the last has its end pushed out by the serration, toward the tip by the rake, and at random
/// by the jitter, and the line from there back to the next segment's start makes the tooth; the
/// serration also sharpens the tip.
fn half_outline(leaf: &Leaf, seed: Seed) -> Vec<Segment> {
    let whole = [
        Vec2::ZERO,
        Vec2::new(leaf.base_width, leaf.base_taper),
        Vec2::new(leaf.tip_width, 1.0 - leaf.tip_taper),
        Vec2::Y,
    ];
    let count = leaf.segments.max(1) as usize;
    let mut segments: Vec<Segment> = (0..count)
        .map(|i| part(&whole, i as f32 / count as f32, (i + 1) as f32 / count as f32))
        .collect();
    let mut rng = seed.stream("leaf shape");
    for segment in &mut segments[..count - 1] {
        let tangent = (segment[3] - segment[2]).normalize_or(Vec2::Y);
        let normal = Vec2::new(tangent.y, -tangent.x);
        let jitter = Vec2::new(2.0 * rng.unit() - 1.0, 2.0 * rng.unit() - 1.0) * leaf.jitter;
        let push = normal * leaf.serration + tangent * (leaf.rake / count as f32) + jitter;
        // The control points move in proportion to how far along the segment they lie.
        let along = (segment[3] - segment[0]).dot(tangent);
        let share = |point: Vec2| {
            if along.abs() > f32::EPSILON {
                (point - segment[0]).dot(tangent) / along
            } else {
                0.0
            }
        };
        let (first, second) = (share(segment[1]), share(segment[2]));
        segment[1] += push * first;
        segment[2] += push * second;
        segment[3] += push;
    }
    let tip = &mut segments[count - 1];
    tip[2].y += leaf.serration;
    tip[3].y += leaf.serration;
    segments
}

/// The part of the cubic `curve` from `t0` to `t1`, by blossoming.
fn part(curve: &Segment, t0: f32, t1: f32) -> Segment {
    let blossom = |a: f32, b: f32, c: f32| {
        let q = [
            curve[0].lerp(curve[1], a),
            curve[1].lerp(curve[2], a),
            curve[2].lerp(curve[3], a),
        ];
        let r = [q[0].lerp(q[1], b), q[1].lerp(q[2], b)];
        r[0].lerp(r[1], c)
    };
    [
        blossom(t0, t0, t0),
        blossom(t0, t0, t1),
        blossom(t0, t1, t1),
        blossom(t1, t1, t1),
    ]
}

/// The sprig's leaves: the middle one up the stem, the fan either side of it, each leaf out from
/// the middle turned further and smaller, down to the outer size, and the pairs down the stem
/// below, at the fan's angle and the outer size.
fn stamps(sprig: &Sprig, seed: Seed) -> Vec<Stamp> {
    let mut rng = seed.stream("sprig");
    let mut stamps = vec![Stamp {
        angle: 0.0,
        scale: 1.0,
        drop: 0.0,
    }];
    let fan = sprig.fan.max(0);
    for i in 1..=fan {
        let t = i as f32 / fan as f32;
        let scale = sprig.outer_size.powf(t);
        for side in [1.0, -1.0] {
            stamps.push(Stamp {
                angle: side * t * sprig.angle.center + sprig.angle.offset(&mut rng),
                scale,
                drop: 0.0,
            });
        }
    }
    let angles = limits::<Sprig>("angle");
    for i in 1..=sprig.pairs.max(0) {
        for side in [1.0, -1.0] {
            let angle = sprig.angle.sample(&mut rng).clamp(*angles.start(), *angles.end());
            let drop = i as f32 * sprig.spacing.center + sprig.spacing.offset(&mut rng);
            stamps.push(Stamp {
                angle: side * angle,
                scale: sprig.outer_size,
                drop: drop.max(0.0),
            });
        }
    }
    stamps
}

/// From a leaf's own frame, its base at the origin and its tip up, to the sprig's.
fn stamp_transform(stamp: &Stamp) -> Transform {
    Transform::from_translate(0.0, -stamp.drop)
        .pre_scale(stamp.scale, stamp.scale)
        .pre_rotate(stamp.angle.to_degrees())
}

/// The whole leaf's outline: the right half from base to tip, then the left, its mirror, back.
fn leaf_path(half: &[Segment]) -> Path {
    let mirror = |point: Vec2| (-point.x, point.y);
    let mut path = PathBuilder::new();
    path.move_to(half[0][0].x, half[0][0].y);
    for [start, a, b, end] in half {
        path.line_to(start.x, start.y);
        path.cubic_to(a.x, a.y, b.x, b.y, end.x, end.y);
    }
    for &[start, a, b, end] in half.iter().rev() {
        let (end, b, a, start) = (mirror(end), mirror(b), mirror(a), mirror(start));
        path.line_to(end.0, end.1);
        path.cubic_to(b.0, b.1, a.0, a.1, start.0, start.1);
    }
    path.close();
    path.finish().expect("a leaf has an outline")
}

/// The stem, from below the texture's bottom edge, so its end is cut off square, up to the fan.
fn stem_path(layout: &Layout) -> Path {
    let below = (layout.fold + 0.1) * TEXTURE_SIZE as f32 / layout.scale;
    let mut path = PathBuilder::new();
    path.move_to(0.0, -below);
    path.line_to(0.0, 0.0);
    path.finish().expect("the stem has a length")
}

/// `color` with its hue turned by `hue` degrees and its lightness shifted by `lightness`, in
/// Oklch, so a step of lightness looks alike on every hue.
fn shifted(color: Color, hue: f32, lightness: f32) -> tiny_skia::Color {
    let mut oklch = Oklcha::from(color);
    oklch.hue = (oklch.hue + hue).rem_euclid(360.0);
    oklch.lightness = (oklch.lightness + lightness).clamp(0.0, 1.0);
    skia_color(oklch.into())
}

/// `color` as painted: its sRGB components as picked, clamped into the gamut.
fn skia_color(color: Color) -> tiny_skia::Color {
    let Srgba {
        red,
        green,
        blue,
        alpha,
    } = color.to_srgba();
    let unit = |value: f32| value.clamp(0.0, 1.0);
    tiny_skia::Color::from_rgba(unit(red), unit(green), unit(blue), unit(alpha))
        .unwrap_or(tiny_skia::Color::BLACK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segments_meet_on_the_curve() {
        let curve = [Vec2::ZERO, Vec2::new(0.3, 0.1), Vec2::new(0.2, 0.6), Vec2::Y];
        let (first, second) = (part(&curve, 0.0, 0.4), part(&curve, 0.4, 1.0));
        assert_eq!(first[0], curve[0]);
        assert!(first[3].distance(second[0]) < 1e-6);
        assert!(second[3].distance(curve[3]) < 1e-6);
    }

    #[test]
    fn the_sprig_fits_the_texture_about_its_stem() {
        let tip_leaves = TipLeaves::default();
        let layout = layout(&tip_leaves, Seed(1), Some(4.0));
        // Nine leaves: the middle one, two either side in the fan, and two pairs.
        assert_eq!(layout.stamps.len(), 9);
        let mut stem = Point::zero();
        layout.to_pixels.map_point(&mut stem);
        assert!((stem.x - TEXTURE_SIZE as f32 * 0.5).abs() < 1e-3);
        // The pairs hang below the fan, so it starts partway up.
        assert!((0.2..0.8).contains(&layout.fold), "{}", layout.fold);
    }

    /// The default species' texture, as painted when it was last checked and judged by eye.
    const GOLDEN: &str = "src/generator/leaf-golden.png";

    /// The most a channel may differ from the golden texture: SIMD may round differently.
    const TOLERANCE: u8 = 2;

    #[test]
    fn the_default_leaf_is_painted_as_the_golden_texture() {
        use bevy::{
            asset::RenderAssetUsages,
            image::{CompressedImageFormats, Image, ImageSampler, ImageType},
            render::render_resource::{Extent3d, TextureDimension, TextureFormat},
        };

        let species = crate::species::Species::default();
        let tip_leaves = &species.ornaments.tip_leaves.settings;
        let stroke = species.leaf_stroke();
        let layout = layout(tip_leaves, species.seed, stroke);
        let texture = paint(&layout, tip_leaves, species.bark_color, species.seed, stroke);
        let golden = std::fs::read(GOLDEN).ok().and_then(|bytes| {
            Image::from_buffer(
                &bytes,
                ImageType::Extension("png"),
                CompressedImageFormats::NONE,
                true,
                ImageSampler::Default,
                RenderAssetUsages::default(),
            )
            .ok()
        });
        let golden = golden.as_ref().and_then(|image| image.data.as_deref());
        let matches = golden.is_some_and(|golden| {
            golden.len() == texture.pixels.len()
                && (golden.iter().zip(&texture.pixels)).all(|(a, b)| a.abs_diff(*b) <= TOLERANCE)
        });
        if !matches {
            let painted = "target/leaf-golden.png";
            let extent = Extent3d {
                width: texture.size,
                height: texture.size,
                depth_or_array_layers: 1,
            };
            let image = Image::new(
                extent,
                TextureDimension::D2,
                texture.pixels,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            );
            image
                .try_into_dynamic()
                .expect("RGBA8 converts")
                .save(painted)
                .expect("the painted texture saves");
            panic!("the leaf texture differs from {GOLDEN}; if {painted} is right, copy it there");
        }
    }
}
