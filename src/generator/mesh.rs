//! Meshing a grown plant: a tube along each axis, and the foliage's cards.

use std::f32::consts::TAU;

use bevy::math::{Vec2, Vec3};

use super::{
    envelope::{Envelope, outward_normal},
    hint::simplify,
    leaf::{self, CardShape, LeafTexture},
    skeleton::{Axis, AxisKind, Frame, Ring, grow, upper_side},
};
use crate::species::{Species, TipLeaves};

/// Triangles with one normal per vertex. Every triangle meeting at a vertex shares it, so normals
/// are continuous across the surface and the outline shell has no gaps.
#[derive(Debug, Default)]
pub struct Geometry {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    /// Texture coordinates, one per vertex, or none for an untextured mesh.
    pub uvs: Vec<Vec2>,
    /// Counter-clockwise seen from outside, as glTF and Bevy take front faces.
    pub indices: Vec<u32>,
}

impl Geometry {
    pub fn triangles(&self) -> usize {
        self.indices.len() / 3
    }

    /// The share of the surface's area a camera looking down at `elevation` sees, turned all the
    /// way around it: 1 if every triangle faced it, 0 if every one were edge-on. Either side counts,
    /// as for double-sided cards. None for no area.
    pub fn facing(&self, elevation: f32) -> Option<f32> {
        const AZIMUTHS: usize = 32;
        let (down, out) = elevation.sin_cos();
        let views: Vec<Vec3> = (0..AZIMUTHS)
            .map(|i| {
                let (sin, cos) = (TAU * i as f32 / AZIMUTHS as f32).sin_cos();
                Vec3::new(out * cos, -down, out * sin)
            })
            .collect();
        let (mut seen, mut area) = (0.0, 0.0);
        for triangle in self.indices.as_chunks::<3>().0 {
            let [a, b, c] = triangle.map(|i| self.positions[i as usize]);
            // Twice the area, square to the triangle.
            let normal = (b - a).cross(c - a);
            area += normal.length();
            seen += views
                .iter()
                .map(|view| normal.dot(*view).abs())
                .sum::<f32>()
                / AZIMUTHS as f32;
        }
        (area > 0.0).then(|| seen / area)
    }
}

/// A plant as generated.
#[derive(Debug, Default)]
pub struct Grown {
    pub bark: Geometry,
    /// The leaf cards, textured with `leaf_texture`.
    pub leaves: Geometry,
    pub leaf_texture: LeafTexture,
    /// Leaf cards.
    pub cards: usize,
    /// Branch ends bearing leaves, one card each or a rosette.
    pub leafy_tips: usize,
    /// Limbs grown on each grown level.
    pub level_limbs: Vec<usize>,
}

/// The whole plant, grown (`grow`) and meshed: the stems and their limbs, none drawn narrower than
/// the bark's detail threshold, a card (`card`) for each leaf cluster, and the leaf texture
/// painted (`leaf::paint`), whether or not any leaves grow. Each stem or limb is
/// drawn as one tube with its fork's first copy (`chain`), left open
/// at its base, which stands in the ground or inside its parent, and closed at its tip by a fan.
pub fn plant(species: &Species) -> Grown {
    let skeleton = grow(species);
    let axes = &skeleton.axes;
    let min_radius = species.detail * 0.5;
    let levels = species.limbs.grown();
    let sides_of = |axis: &Axis| {
        let sides = match axis.level {
            0 => species.trunk.sides,
            level => levels[level - 1].sides,
        };
        sides.max(3) as u32
    };
    let mut bark = Geometry::default();
    let mut level_limbs = vec![0; levels.len()];
    for (index, axis) in axes.iter().enumerate() {
        let carried =
            (axis.parent).is_some_and(|(parent, _)| carrier(axes, &axes[parent]) == Some(index));
        if !carried {
            let (rings, sides) = chain(axes, axis, sides_of);
            tube(&mut bark, &rings, &sides, min_radius);
        }
        if axis.level > 0 {
            level_limbs[axis.level - 1] += 1;
        }
    }
    let tip_leaves = &species.ornaments.tip_leaves.settings;
    let sprig = leaf::layout(tip_leaves, species.seed, species.leaf_stroke());
    let envelope = Envelope::of(species);
    let mut leaves = Geometry::default();
    let mut leafy_tips = 0;
    let mut last_axis = None;
    for cluster in &skeleton.cards {
        if last_axis != Some(cluster.axis) {
            leafy_tips += 1;
            last_axis = Some(cluster.axis);
        }
        let twig = &axes[cluster.axis];
        let run = sprig.card.fold * sprig.card.height * cluster.size;
        let spine = if cluster.bends {
            spine(twig, cluster.at, run, species.detail)
        } else {
            let frame = Frame {
                tangent: cluster.direction,
                ..twig.frame_at(cluster.at)
            };
            straight_spine(&frame, run).to_vec()
        };
        card(
            &mut leaves,
            &spine,
            cluster.size,
            &sprig.card,
            tip_leaves,
            &envelope,
        );
    }
    let leaf_texture = leaf::paint(
        &sprig,
        tip_leaves,
        species.bark_color,
        species.seed,
        species.leaf_stroke(),
    );
    Grown {
        bark,
        leaves,
        leaf_texture,
        cards: skeleton.cards.len(),
        leafy_tips,
        level_limbs,
    }
}

/// A card's spine along `twig`, from `at` metres along it to `run` metres on: the twig there, and
/// those of its rings between that `simplify` keeps at `tolerance`, the bark's detail threshold,
/// so a card bends only where a straight one would lie visibly off its twig.
pub(super) fn spine(twig: &Axis, at: f32, run: f32, tolerance: f32) -> Vec<Frame> {
    const APART: f32 = 1e-4;
    let between =
        (twig.rings.iter()).filter(|ring| ring.at > at + APART && ring.at < at + run - APART);
    let rings = between.map(|ring| Frame {
        point: ring.center,
        tangent: ring.tangent,
        side: ring.side,
        radius: ring.radius,
    });
    let mut frames = vec![twig.frame_at(at)];
    frames.extend(rings);
    frames.push(twig.frame_at(at + run));
    let points: Vec<Vec3> = frames.iter().map(|frame| frame.point).collect();
    (simplify(&points, tolerance).into_iter())
        .map(|i| frames[i])
        .collect()
}

/// A straight spine from `frame`, `run` metres along its tangent.
pub(super) fn straight_spine(frame: &Frame, run: f32) -> [Frame; 2] {
    let end = Frame {
        point: frame.point + frame.tangent * run,
        ..*frame
    };
    [*frame, end]
}

/// Adds a card shaped as `shape`, its leaves `size` long, for a sprig along `spine` (`spine`,
/// `straight_spine`), frames from its base to its fold: the midrib through each frame's point,
/// so a sprig along its twig follows it, and the face toward each frame's upper side, world up
/// made square to its tangent, so it is turned as near up as the twig allows. It shows the part of
/// the texture `shape` says, its bottom at the base, stretched along the spine by its length.
///
/// The card is two quads across, after seedling, a row from each frame to the next and one beyond
/// the fold, folded twice: each half turns down about the midrib by the lateral droop (up,
/// cupping, if negative), and the far row bends down about the line across at the fold, by the
/// axial droop. The folds shape the outline but not the shading: each corner keeps the normal of
/// its frame's flat card, so a facet turned from the light is not cut dark against its
/// neighbours. The corners go near rows before far, left before right, each quad's
/// counter-clockwise from its face starting at its corner nearest the base on the left.
///
/// Above 0, the crown softness blends each corner's normal toward the envelope's outward normal
/// there, so the crown shades as one mass.
pub(super) fn card(
    geometry: &mut Geometry,
    spine: &[Frame],
    size: f32,
    shape: &CardShape,
    leaves: &TipLeaves,
    envelope: &Envelope,
) {
    let shading = |normal: Vec3, position: Vec3| {
        if leaves.softness <= 0.0 {
            return normal;
        }
        (normal.lerp(outward_normal(envelope, position), leaves.softness)).normalize_or(normal)
    };
    let (lateral_sin, lateral_cos) = leaves.lateral_droop.sin_cos();
    let (width, height, fold) = (shape.width, shape.height, shape.fold);
    // Each line across the card: how far along it lies in heights, and its corners from the left
    // edge to the right, `u` across from the midrib in widths, with their normal. `beyond` is its
    // distance past `frame` in heights, turned down about the line across `frame` by `axial`.
    let line = |frame: &Frame, v: f32, beyond: f32, axial: f32| {
        let (axial_sin, axial_cos) = axial.sin_cos();
        let normal = upper_side(frame.tangent, frame.side);
        // Square to both, so corners run counter-clockwise seen from the face.
        let across = frame.tangent.cross(normal);
        let corner = |u: f32| {
            let x = u * width;
            let (x, y) = (x * lateral_cos, -x.abs() * lateral_sin);
            let (z, y) = (
                beyond * axial_cos + y * axial_sin,
                y * axial_cos - beyond * axial_sin,
            );
            frame.point + (across * x + normal * y + frame.tangent * z) * size
        };
        (v, [-0.5, 0.0, 0.5].map(|u| (u, corner(u))), normal)
    };
    let lengths: Vec<f32> = (spine.windows(2))
        .scan(0.0, |length, pair| {
            *length += pair[0].point.distance(pair[1].point);
            Some(*length)
        })
        .collect();
    let total = lengths.last().copied().unwrap_or(0.0).max(f32::EPSILON);
    let mut lines = vec![line(&spine[0], 0.0, 0.0, 0.0)];
    for (frame, length) in spine[1..].iter().zip(&lengths) {
        lines.push(line(frame, fold * length / total, 0.0, 0.0));
    }
    let last = &spine[spine.len() - 1];
    let beyond = (1.0 - fold) * height;
    lines.push(line(last, 1.0, beyond, leaves.axial_droop));
    let uv = |u: f32, v: f32| {
        let (min, max) = (shape.uv_min, shape.uv_max);
        Vec2::new(
            min.x + (u + 0.5) * (max.x - min.x),
            max.y - v * (max.y - min.y),
        )
    };
    for pair in lines.windows(2) {
        let [(v0, near, near_normal), (v1, far, far_normal)] = pair else {
            unreachable!()
        };
        for half in 0..2 {
            let first = geometry.positions.len() as u32;
            let corners = [
                (near[half], *v0, *near_normal),
                (near[half + 1], *v0, *near_normal),
                (far[half + 1], *v1, *far_normal),
                (far[half], *v1, *far_normal),
            ];
            for ((u, position), v, normal) in corners {
                geometry.positions.push(position);
                geometry.normals.push(shading(normal, position));
                geometry.uvs.push(uv(u, v));
            }
            geometry
                .indices
                .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
        }
    }
}

/// The scion of `axis`, among `axes`, drawn in one tube with it: if it forked, its first copy
/// (D31).
fn carrier(axes: &[Axis], axis: &Axis) -> Option<usize> {
    (axis.scions.iter())
        .find(|&&scion| axes[scion].kind == AxisKind::ForkCopy)
        .copied()
}

/// The rings of `axis` and of each first copy carrying it on in turn (`carrier`), as one path,
/// with the sides each ring is drawn with: its axis's (`sides_of`). Each copy starts on the ring
/// where its axis was cut, which the path keeps once, with the copy's sides, facing halfway
/// between the segments meeting there, so a fork's turn is an elbow. Distances run on along the
/// path.
pub(super) fn chain(
    axes: &[Axis],
    axis: &Axis,
    sides_of: impl Fn(&Axis) -> u32,
) -> (Vec<Ring>, Vec<u32>) {
    let mut rings = axis.rings.clone();
    let mut sides = vec![sides_of(axis); rings.len()];
    let mut end = axis;
    while let Some(next) = carrier(axes, end) {
        let next = &axes[next];
        let joint = rings.len() - 1;
        let offset = rings[joint].at;
        let into = rings[joint].center - rings[joint - 1].center;
        let out = next.rings[1].center - next.rings[0].center;
        let ring = &mut rings[joint];
        ring.tangent =
            (into.normalize_or_zero() + out.normalize_or_zero()).normalize_or(ring.tangent);
        ring.side = (ring.side.reject_from_normalized(ring.tangent)).normalize_or(ring.side);
        sides[joint] = sides_of(next);
        for ring in &next.rings[1..] {
            rings.push(Ring {
                at: ring.at + offset,
                ..*ring
            });
            sides.push(sides_of(next));
        }
        end = next;
    }
    (rings, sides)
}

/// Adds a tube through `rings`, each with `sides` of its own, turned to its ring's side, so the
/// tube rolls only as the sides do, and drawn at its ring's radius, but none narrower than
/// `min_radius`. A segment between rings of the same count is a band of quads; between counts N
/// and M, a stitch of N + M triangles, walking both rings by angle. The end is closed by a fan;
/// the start is left open.
pub(super) fn tube(geometry: &mut Geometry, rings: &[Ring], sides: &[u32], min_radius: f32) {
    let drawn = |ring: &Ring| ring.radius.max(min_radius);
    // Each ring's first vertex.
    let mut starts = Vec::with_capacity(rings.len());
    for (i, ring) in rings.iter().enumerate() {
        starts.push(geometry.positions.len() as u32);
        let sides = sides[i];
        // A narrowing tube's surface leans in, so its normals lean forward by the same angle,
        // measured between the rings either side.
        let (before, after) = (
            &rings[i.saturating_sub(1)],
            &rings[(i + 1).min(rings.len() - 1)],
        );
        let slope = (drawn(before) - drawn(after)) / (after.at - before.at).max(1e-4);
        let (lean_sin, lean_cos) = slope.atan().sin_cos();
        let across = ring.side.cross(ring.tangent);
        let radius = drawn(ring);
        for side in 0..sides {
            let (sin, cos) = (TAU * side as f32 / sides as f32).sin_cos();
            let outward = ring.side * cos + across * sin;
            geometry.positions.push(ring.center + outward * radius);
            geometry
                .normals
                .push((outward * lean_cos + ring.tangent * lean_sin).normalize());
        }
    }
    for ring in 0..rings.len() - 1 {
        let (n, m) = (sides[ring], sides[ring + 1]);
        let near = |side: u32| starts[ring] + side % n;
        let far = |side: u32| starts[ring + 1] + side % m;
        // Each step advances along the ring whose next vertex comes first by angle, n + m steps.
        let (mut i, mut j) = (0, 0);
        while i < n || j < m {
            if j < m && (i == n || (j + 1) * n <= (i + 1) * m) {
                geometry.indices.extend([near(i), far(j), far(j + 1)]);
                j += 1;
            } else {
                geometry.indices.extend([near(i), far(j), near(i + 1)]);
                i += 1;
            }
        }
    }

    let end = rings[rings.len() - 1];
    let tip = geometry.positions.len() as u32;
    geometry.positions.push(end.center);
    geometry.normals.push(end.tangent);
    let (last, sides) = (starts[rings.len() - 1], sides[rings.len() - 1]);
    for side in 0..sides {
        geometry
            .indices
            .extend([last + side, tip, last + (side + 1) % sides]);
    }
}
