//! The generator: a species in, a plant's geometry out. Plain functions on Bevy's math types, with
//! no ECS, so they can move to a library of their own (ledger X1).

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::math::{Mat3, Quat, Vec2, Vec3};

use crate::{
    fuzzy::Fuzzy,
    random::Rng,
    species::{Facing, Foliage, LimbLevel, Profile, Species},
};

/// The crown's envelope in meters, as the species' crown and trunk height place it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Envelope {
    pub profile: Profile,
    /// The heights of the crown's base and top above the ground.
    pub base: f32,
    pub top: f32,
    /// The radius where the crown is widest.
    pub radius: f32,
}

impl Envelope {
    pub fn of(species: &Species) -> Self {
        let (height, crown) = (species.trunk.height.max(0.01), &species.crown);
        let top = height * crown.top.max(0.01);
        Self {
            profile: crown.profile,
            base: (height * crown.base).clamp(0.0, top),
            top,
            radius: crown.radius,
        }
    }
}

#[cfg(test)]
impl Default for Envelope {
    fn default() -> Self {
        Self::of(&Species::default())
    }
}

/// The distance between kinks on every limb and stem, in crown radii: a limb's zigzag reads by
/// its period, so long limbs kink several times and short twigs once or not at all.
const KINK_SPACING: f32 = 0.15;
/// The most a stretch between kinks is longer or shorter than `KINK_SPACING`, as a share of it,
/// so kinks fall unevenly without two of them running together.
const KINK_JITTER: f32 = 0.4;

/// The step in tracing a limb, in metres.
const TRACE_STEP: f32 = 0.02;

/// Streams forked from a stem's own: its children take theirs by index from 0, so these take
/// indices no child reaches.
const WANDER_STREAM: u64 = u64::MAX;
/// The values drawn for a limb itself: its angle and reach.
const OWN_VALUES_STREAM: u64 = u64::MAX - 1;
/// The layout of a stem's children: their count, and each node's turn, limb count and place.
const LAYOUT_STREAM: u64 = u64::MAX - 2;
/// A limb's leaf clusters: their count, and each one's place and turn.
const FOLIAGE_STREAM: u64 = u64::MAX - 3;
/// The limb carrying a stem on past its last node.
const CARRY_ON_STREAM: u64 = u64::MAX - 4;

/// The most a node moves from its even place, as a share of the gap between places, so nodes are
/// not spaced like rungs. Under half, so neighbours never swap.
const NODE_JITTER: f32 = 0.3;

/// Turns tried for each node, evenly around the parent.
const AZIMUTH_CANDIDATES: u32 = 72;
/// How much of a placed limb's crowding is left one mean gap further along the parent.
const CROWDING_FADE: f32 = 0.7;
/// The power the nearness of two azimuths is raised to, so crowding falls off sharply around a
/// limb. With `CROWDING_FADE`, it gives the golden angle for one limb a node; gentler, the turn
/// falls to 180°, and sharper, it turns erratic.
const CROWDING_SHARPNESS: i32 = 4;
/// How much a limb pointing straight out from the plant's axis gains, a crown radius off it,
/// against crowding of 1 for a limb at the same azimuth one gap away.
const OUTWARD_WEIGHT: f32 = 1.0;

/// Halvings in finding where a ray leaves the envelope: the largest envelope's 90 m bound to under
/// a tenth of a millimetre.
const ENVELOPE_BISECTIONS: u32 = 20;

/// Points down each meridian of the envelope's outline, base to top.
const OUTLINE_STEPS: u32 = 16;
/// Meridians around the envelope's outline.
const OUTLINE_MERIDIANS: u32 = 8;
/// A ring around the outline every this many steps.
const OUTLINE_RING_EVERY: u32 = 2;
/// Segments in each ring of the outline.
const OUTLINE_RING_SIDES: u32 = 24;

/// Segments of the envelope's profile, base to top, in measuring how deep a point lies.
const DEPTH_STEPS: u32 = 32;

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
}

/// A ring along a stem, the trunk or a limb, as meshed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    pub center: Vec3,
    /// Unit direction of the stem.
    pub tangent: Vec3,
    /// Unit vector square to the tangent, carried along the stem without twisting (a
    /// rotation-minimizing frame). It sets the ring's roll, and azimuth 0 for children starting
    /// there.
    pub side: Vec3,
    /// Distance along the stem from its base.
    pub at: f32,
}

/// A limb as grown on its parent, before meshing.
#[derive(Debug, Clone, PartialEq)]
pub struct Limb {
    /// The limb's level, 0 for the first.
    pub level: usize,
    /// At least two, base to tip.
    pub rings: Vec<Ring>,
    pub radii: (f32, f32),
    /// The limb's own random stream, before any draw; its children's streams fork from it.
    rng: Rng,
    /// Whether it bears limbs; if not, it bears leaves.
    children: bool,
    /// Whether a limb carries it on past its tip.
    carried_on: bool,
    /// How far along it keeps its base radius before narrowing to its top.
    narrows_from: f32,
}

impl Limb {
    /// A limb before its own limbs have grown.
    fn new(level: usize, rings: Vec<Ring>, radii: (f32, f32), rng: Rng) -> Self {
        Self {
            level,
            rings,
            radii,
            rng,
            children: false,
            carried_on: false,
            narrows_from: 0.0,
        }
    }
}

#[cfg(test)]
impl Limb {
    pub fn origin(&self) -> Vec3 {
        self.rings[0].center
    }

    /// Unit direction at the base.
    pub fn direction(&self) -> Vec3 {
        self.rings[0].tangent
    }

    pub fn length(&self) -> f32 {
        self.rings[self.rings.len() - 1].at
    }
}

/// A parent at one point along it: where its children start and how they are oriented.
#[derive(Debug, Clone, Copy)]
struct Frame {
    point: Vec3,
    /// Unit direction of the parent.
    tangent: Vec3,
    /// Unit vector square to the tangent, toward azimuth 0.
    side: Vec3,
}

/// A plant as generated.
#[derive(Debug, Default)]
pub struct Grown {
    pub bark: Geometry,
    /// One card for each leaf cluster, textured with the leaf.
    pub leaves: Geometry,
    /// Limbs grown on each grown level.
    pub level_limbs: Vec<usize>,
}

/// The whole plant: the stems and their limbs, none drawn narrower than the bark's detail
/// threshold, and the foliage (`foliage`). Each stem is a tube left open at the base, since it
/// stands in the ground. A tube is closed at the top by a fan, unless a limb carries it on: that
/// limb starts on its top ring, at its top radius.
pub fn plant(species: &Species) -> Grown {
    let (stems, limbs) = grow(species);
    let min_radius = species.bark.detail * 0.5;
    let drawn = |(base, top): (f32, f32)| (base.max(min_radius), top.max(min_radius));
    let mut bark = Geometry::default();
    let trunk_sides = species.trunk.sides.max(3) as u32;
    for stem in &stems {
        let radii = drawn(stem.radii);
        tube(
            &mut bark,
            &stem.rings,
            radii,
            stem.narrows_from,
            trunk_sides,
            !stem.carried_on,
        );
    }
    let levels = species.grown_levels();
    for limb in &limbs {
        let sides = levels[limb.level].sides.max(3) as u32;
        let radii = drawn(limb.radii);
        tube(
            &mut bark,
            &limb.rings,
            radii,
            limb.narrows_from,
            sides,
            !limb.carried_on,
        );
    }
    let mut level_limbs = vec![0; levels.len()];
    for limb in &limbs {
        level_limbs[limb.level] += 1;
    }
    let ends = limbs.iter().filter(|limb| !limb.children);
    Grown {
        bark,
        leaves: foliage(&Envelope::of(species), &species.foliage, ends),
        level_limbs,
    }
}

/// Leaf clusters on `twigs`, the limbs bearing none: one at each twig's tip, wherever it ends, so no
/// branch end is bare, and more along the parts of it within the foliage's shell of the envelope,
/// as many as its density gives for that length, each placed as `node_place` places nodes, so they
/// are spread evenly but not like rungs. Each is a card (`card`) whose sprig grows from its place
/// along the twig. No density leaves the plant bare.
fn foliage<'a>(
    envelope: &Envelope,
    foliage: &Foliage,
    twigs: impl Iterator<Item = &'a Limb>,
) -> Geometry {
    let crown = envelope.radius.max(0.01);
    let shell = foliage.shell * crown;
    let size = foliage.size * crown;
    let mut leaves = Geometry::default();
    if foliage.density <= 0.0 {
        return leaves;
    }
    for twig in twigs {
        let parent = Parent {
            rings: &twig.rings,
            radius: twig.radii.0,
            rng: &twig.rng,
        };
        card(&mut leaves, &parent.frame(1.0), size);
        let length = twig.rings[twig.rings.len() - 1].at;
        let steps = ((length / TRACE_STEP).ceil() as u32).max(1);
        // Places a trace step apart in the shell, each standing for a step of the twig.
        let in_shell: Vec<Frame> = (0..steps)
            .map(|step| parent.frame((step as f32 + 0.5) / steps as f32))
            .filter(|frame| {
                depth_in_envelope(envelope, frame.point).is_some_and(|depth| depth < shell)
            })
            .collect();
        if in_shell.is_empty() {
            continue;
        }
        let mut rng = twig.rng.fork(FOLIAGE_STREAM);
        let shell_length = in_shell.len() as f32 * length / steps as f32;
        let clusters = rng.round(foliage.density * shell_length / crown);
        for cluster in 0..clusters {
            let place = node_place(cluster, clusters, 0.0, rng.unit());
            let index = ((place * in_shell.len() as f32) as usize).min(in_shell.len() - 1);
            card(&mut leaves, &in_shell[index], size);
        }
    }
    leaves
}

/// Adds a square card `size` across for a sprig growing from `frame`: its base on the twig, its
/// axis along the twig, so the sprig's painted stem lines up with it, and its face toward the
/// twig's upper side, world up made square to the twig, so it is turned as near up as the twig
/// allows. The texture's bottom is the base.
fn card(geometry: &mut Geometry, frame: &Frame, size: f32) {
    let first = geometry.positions.len() as u32;
    let axis = frame.tangent;
    let normal = upper_side(axis, frame.side);
    // Square to both, so the corners run counter-clockwise seen from the face.
    let half_across = axis.cross(normal) * (0.5 * size);
    let along = axis * size;
    for (corner, uv) in [
        (-half_across, Vec2::new(0.0, 1.0)),
        (half_across, Vec2::new(1.0, 1.0)),
        (half_across + along, Vec2::new(1.0, 0.0)),
        (-half_across + along, Vec2::new(0.0, 0.0)),
    ] {
        geometry.positions.push(frame.point + corner);
        geometry.normals.push(normal);
        geometry.uvs.push(uv);
    }
    geometry
        .indices
        .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
}

/// A stem rising from the ground, as grown, before meshing.
#[derive(Debug, Clone, PartialEq)]
pub struct Stem {
    /// At least two, base to tip.
    pub rings: Vec<Ring>,
    pub radii: (f32, f32),
    /// The stem's own random stream; its limbs' streams fork from it.
    rng: Rng,
    /// Whether a limb carries it on past its tip.
    carried_on: bool,
    /// How far along it keeps its base radius before narrowing to its top.
    narrows_from: f32,
}

/// The stems: `stems` of them from the origin, spaced evenly around, each leaving the ground at
/// the splay from up and turning by the lean as it rises, away from the middle, but never past
/// level, where it would sink again, and kinking about that course by the trunk's wander. The
/// first's compass heading is drawn from the seed.
fn stems(species: &Species) -> Vec<Stem> {
    let trunk = &species.trunk;
    let count = trunk.stems.max(1) as u32;
    let length = trunk.height.max(0.01);
    let lean = trunk.lean.min((FRAC_PI_2 - trunk.splay).max(0.0));
    let heading = species.seed.stream("trunk").unit() * TAU;
    let branching = species.seed.stream("branching");
    let top = trunk.radius * (1.0 - trunk.taper.clamp(0.0, 1.0));
    let spacing = kink_spacing(&Envelope::of(species));
    (0..count)
        .map(|stem| {
            let rng = branching.fork(stem as u64);
            let kinks = Kinks::new(spacing, trunk.wander, rng.fork(WANDER_STREAM));
            Stem {
                rings: stem_rings(
                    length,
                    trunk.splay,
                    lean,
                    heading + TAU * stem as f32 / count as f32,
                    kinks,
                    species.bark.detail,
                ),
                radii: (trunk.radius, top),
                rng,
                carried_on: false,
                narrows_from: 0.0,
            }
        })
        .collect()
}

/// The limbs, as `grow` grows them.
#[cfg(test)]
pub fn limbs(species: &Species) -> Vec<Limb> {
    grow(species).1
}

/// The stems, and their limbs level by level, each level on the limbs of the one before and the
/// first on the stems, in level order. A stem's top, its radius there and where it ends, is what
/// its limbs leave it (`grow_level`), so it is set once the level after has grown.
fn grow(species: &Species) -> (Vec<Stem>, Vec<Limb>) {
    let mut stems = stems(species);
    let mut grown: Vec<Limb> = Vec::new();
    for level in 0..species.grown_levels().len() {
        let mut children = Vec::new();
        if level == 0 {
            for stem in &mut stems {
                let parent = Parent {
                    rings: &stem.rings,
                    radius: stem.radii.0,
                    rng: &stem.rng,
                };
                let bearing = grow_level(species, level, parent, &mut children);
                stem.carried_on =
                    bearing.end(&mut stem.rings, &mut stem.radii, &mut stem.narrows_from);
            }
        }
        for limb in grown.iter_mut().filter(|limb| limb.level + 1 == level) {
            let parent = Parent {
                rings: &limb.rings,
                radius: limb.radii.0,
                rng: &limb.rng,
            };
            let bearing = grow_level(species, level, parent, &mut children);
            limb.children = bearing.children;
            limb.carried_on = bearing.end(&mut limb.rings, &mut limb.radii, &mut limb.narrows_from);
        }
        grown.extend(children);
    }
    (stems, grown)
}

/// A stem limbs grow on: the trunk, or a limb of the level before.
#[derive(Clone, Copy)]
struct Parent<'a> {
    rings: &'a [Ring],
    /// Radius at the base.
    radius: f32,
    rng: &'a Rng,
}

impl Parent<'_> {
    /// The parent at `t` (0 to 1) along it as meshed: on the straight chord between ring
    /// centers, so children start inside the bark, with that chord's direction.
    fn frame(&self, t: f32) -> Frame {
        let t = t.clamp(0.0, 1.0);
        let at = t * self.rings[self.rings.len() - 1].at;
        let next = self
            .rings
            .partition_point(|ring| ring.at <= at)
            .clamp(1, self.rings.len() - 1);
        let (a, b) = (self.rings[next - 1], self.rings[next]);
        let along = if b.at > a.at {
            (at - a.at) / (b.at - a.at)
        } else {
            0.0
        };
        let tangent = (b.center - a.center).normalize_or(a.tangent);
        let side = a
            .side
            .lerp(b.side, along)
            .reject_from_normalized(tangent)
            .normalize_or(a.side);
        Frame {
            point: a.center.lerp(b.center, along),
            tangent,
            side,
        }
    }

    /// The part of the parent inside the crown, from the first of its points inside to the last,
    /// as fractions of its length; `None` if no point a trace step apart is inside. A limb lies
    /// wholly inside, so only a stem's part is shorter than the stem.
    fn inside_part(&self, crown: &Envelope) -> Option<(f32, f32)> {
        let length = self.rings[self.rings.len() - 1].at;
        let steps = ((length / TRACE_STEP).ceil() as u32).max(1);
        let inside = |t: f32| inside_envelope(crown, self.frame(t).point);
        let at = |step: u32| step as f32 / steps as f32;
        let first = (0..=steps).find(|&step| inside(at(step)))?;
        let last = (first..=steps).rev().find(|&step| inside(at(step)))?;
        // Each end lies between a step inside and its neighbour outside: halve the gap.
        let edge = |mut inner: f32, mut outer: f32| {
            for _ in 0..ENVELOPE_BISECTIONS {
                let middle = (inner + outer) * 0.5;
                if inside(middle) {
                    inner = middle;
                } else {
                    outer = middle;
                }
            }
            inner
        };
        let start = if first > 0 {
            edge(at(first), at(first - 1))
        } else {
            0.0
        };
        let end = if last < steps {
            edge(at(last), at(last + 1))
        } else {
            1.0
        };
        Some((start, end))
    }
}

/// Adds one parent's limbs of level `level`: nodes along the part of the parent inside the crown,
/// as many as the level's density gives for its length, each placed by `node_place`, its limbs
/// turned about the parent by `node_azimuths` and their skew, each leaving at the level's angle
/// from the parent and then grown (`grow_limb`). A limb whose start lies outside the crown, or
/// that would be no longer than it is thick, is left out.
///
/// A parent with limbs ends at its last node, and one more limb of the level carries it on from
/// there, along it: at angle 0, with no skew, but otherwise grown as the others are.
///
/// Radii follow the pipe model: a stem's radius raised to the bark's exponent is shared among the
/// limbs leaving it and what carries on past them. Each limb's radius is the level's share of the
/// parent's radius past where it leaves, so the parent is never used up. The parent's
/// radius at its top is what its limbs leave it, or its base radius tapered if that is thinner, so
/// the taper is the least a stem narrows; the limb carrying it on starts at that radius.
fn grow_level(species: &Species, level: usize, parent: Parent, grown: &mut Vec<Limb>) -> Bearing {
    let envelope = &Envelope::of(species);
    let levels = species.grown_levels();
    let params = &levels[level];
    let taper = species.trunk.taper.clamp(0.0, 1.0);
    let exponent = species.bark.pipe_exponent.max(1.0);
    let spacing = kink_spacing(envelope);
    let kinks = |rng: &Rng| Kinks::new(spacing, params.wander, rng.fork(WANDER_STREAM));
    let tolerance = species.bark.detail;
    // The parent's radius raised to the exponent, which each limb leaving it takes its share of.
    let mut section = parent.radius.powf(exponent);
    let tapered = parent.radius * (1.0 - taper);
    let Some((inside_start, inside_end)) = parent.inside_part(envelope) else {
        return Bearing {
            top: tapered,
            children: false,
            tip: None,
            hold: None,
        };
    };
    let first_child = grown.len();
    // Where along the parent the first and last limbs grown leave it.
    let mut first_node = None;
    let mut last_node = 0.0;
    // The node count and each node's turn, limb count and place are the parent's, drawn from a
    // stream of its own beside its limbs'.
    let mut layout = parent.rng.fork(LAYOUT_STREAM);
    let length = parent.rings[parent.rings.len() - 1].at;
    let inside = inside_end - inside_start;
    let inside_length = inside * length;
    let nodes = layout.round(params.density * inside_length / envelope.radius.max(0.01));
    let gap = inside_length / nodes.max(1) as f32;
    // Each limb placed so far: how far along the parent it leaves, and its azimuth.
    let mut placed: Vec<(f32, f32)> = Vec::new();
    let mut index = 0;
    for node in 0..nodes {
        let per_node = layout.round(params.per_node).max(1);
        let place = node_place(node, nodes, params.start, layout.unit());
        let t = inside_start + inside * place;
        let frame = parent.frame(t);
        let across = frame.side.cross(frame.tangent);
        let crowd = Crowd {
            at: t * length,
            gap,
            placed: &placed,
        };
        let azimuths = node_azimuths(&frame, per_node, params.facing, &crowd, envelope);
        // The node's limbs and the stem carrying on past them share its wood, each limb the
        // level's share of the stem's radius past it, so an even fork at 100%.
        let shares = 1.0 + per_node as f32 * params.radius.powf(exponent);
        let base_radius = params.radius * (section / shares).powf(exponent.recip());
        for azimuth in azimuths {
            let rng = parent.rng.fork(index);
            index += 1;
            let mut own = rng.fork(OWN_VALUES_STREAM);
            let angle = draw(&params.angle, "angle", &mut own);
            let reach = draw(&params.reach, "reach", &mut own);
            let azimuth = azimuth + (own.unit() * 2.0 - 1.0) * params.skew;
            placed.push((t * length, azimuth));
            let (sin, cos) = azimuth.sin_cos();
            let (angle_sin, angle_cos) = angle.sin_cos();
            let direction =
                frame.tangent * angle_cos + (frame.side * cos + across * sin) * angle_sin;
            let reach = reach * distance_to_envelope(envelope, frame.point, direction);
            let start = Ring {
                center: frame.point,
                tangent: direction,
                side: upper_side(direction, frame.side),
                at: 0.0,
            };
            let rings = grow_limb(envelope, params, start, reach, kinks(&rng), tolerance);
            if rings.len() >= 2 && rings[rings.len() - 1].at > base_radius {
                section = (section - base_radius.powf(exponent)).max(0.0);
                first_node.get_or_insert(t);
                last_node = t;
                grown.push(Limb::new(
                    level,
                    rings,
                    (base_radius, base_radius * (1.0 - taper)),
                    rng,
                ));
            }
        }
    }
    // The limbs' narrowing, or at least the taper's.
    let top = section.powf(exponent.recip()).min(tapered);
    let children = grown.len() > first_child;
    if !children {
        return Bearing {
            top,
            children,
            tip: None,
            hold: None,
        };
    }
    // The limb carrying the parent on, from a stream of its own, so the limbs before it leave its
    // draws alone. It starts on the parent's new tip, turned as that is, so the two tubes meet.
    let frame = parent.frame(last_node);
    let tip = Ring {
        center: frame.point,
        tangent: frame.tangent,
        side: frame.side,
        at: last_node * length,
    };
    let rng = parent.rng.fork(CARRY_ON_STREAM);
    let reach = draw(&params.reach, "reach", &mut rng.fork(OWN_VALUES_STREAM))
        * distance_to_envelope(envelope, frame.point, frame.tangent);
    let start = Ring { at: 0.0, ..tip };
    let rings = grow_limb(envelope, params, start, reach, kinks(&rng), tolerance);
    let carried_on = rings.len() >= 2 && rings[rings.len() - 1].at > top;
    if carried_on {
        grown.push(Limb::new(level, rings, (top, top * (1.0 - taper)), rng));
    }
    // No wood leaves the parent below its first limb, so it keeps its radius to there and narrows
    // past it: a ring there, where narrowing all the way instead would show, moving its outline by
    // more than the detail threshold.
    let end = if carried_on { tip.at } else { length };
    let hold = first_node
        .map(|t| t * length)
        .filter(|&at| at < end - TRACE_STEP && (parent.radius - top) * at / end > tolerance);
    let hold = hold.map(|at| {
        let frame = parent.frame(at / length);
        Ring {
            center: frame.point,
            tangent: frame.tangent,
            side: frame.side,
            at,
        }
    });
    Bearing {
        top,
        children,
        tip: carried_on.then_some(tip),
        hold,
    }
}

/// What a parent's limbs leave it.
struct Bearing {
    /// Its radius at its top.
    top: f32,
    /// Whether it bears limbs.
    children: bool,
    /// Its new tip, at its last node, if a limb carries it on from there.
    tip: Option<Ring>,
    /// A ring at its first limb, which it keeps its radius to, if that shows.
    hold: Option<Ring>,
}

impl Bearing {
    /// Ends the parent with these rings and radii as its limbs leave it: narrowed to its top from
    /// `narrows_from`, and cut at its new tip if it has one. Returns whether a limb carries it on.
    fn end(&self, rings: &mut Vec<Ring>, radii: &mut (f32, f32), narrows_from: &mut f32) -> bool {
        radii.1 = self.top;
        if let Some(tip) = self.tip {
            end_at(rings, tip);
        }
        if let Some(hold) = self.hold {
            let i = rings.partition_point(|ring| ring.at < hold.at);
            if (rings[i].at - hold.at).abs() > 1e-4 {
                rings.insert(i, hold);
            }
            *narrows_from = hold.at;
        }
        self.tip.is_some()
    }
}

/// A limb's rings, from `start` for up to `length` along it, kept to `tolerance`
/// (`Trace::finish`). Its course is traced in short steps from its start, turning toward the
/// level's bend, and the limb kinks about its course (`Kinks`). It stops early where it would
/// leave the envelope.
fn grow_limb(
    envelope: &Envelope,
    params: &LimbLevel,
    start: Ring,
    length: f32,
    mut kinks: Kinks,
    tolerance: f32,
) -> Vec<Ring> {
    let (bend_sin, bend_cos) = params.bend_toward.sin_cos();
    // The share of the angle left to the bend that one step closes. Shares compound, so a run of
    // steps closes the same share whatever the step.
    let kept_per_crown = 1.0 - params.bend.clamp(0.0, 0.99);
    let crown = envelope.radius.max(0.01);

    // The course, and a side square to it carried along without twisting.
    let mut course = start.tangent;
    let mut course_side = start.side;
    let mut trace = Trace::new(start);
    let (mut center, mut tangent, mut at) = (start.center, start.tangent, 0.0);
    while at < length {
        let step = kinks.step(at, TRACE_STEP.min(length - at));
        let next = center + tangent * step;
        if !inside_envelope(envelope, next) {
            break;
        }
        center = next;
        at += step;
        // Toward the bend, keeping the compass heading; a course pointing straight up or down
        // takes its side's.
        let heading = Vec3::new(course.x, 0.0, course.z)
            .try_normalize()
            .or_else(|| Vec3::new(course_side.x, 0.0, course_side.z).try_normalize());
        if let Some(heading) = heading {
            let toward = Vec3::Y * bend_cos + heading * bend_sin;
            let closed = 1.0 - kept_per_crown.powf(step / crown);
            let turned = turn_toward(course, toward, course.angle_between(toward) * closed);
            course_side = (Quat::from_rotation_arc(course, turned) * course_side)
                .reject_from_normalized(turned)
                .normalize_or(course_side);
            course = turned;
        }
        tangent = kinks.direction(at, length, course, course_side);
        trace.push(center, at, tangent);
    }
    trace.finish(tolerance)
}

/// Where node `node` of `nodes` sits, as a fraction of the stretch bearing them: centred in its
/// equal share of the stretch past `start`, moved by up to `NODE_JITTER` of a share by `unit` (0
/// to 1).
fn node_place(node: u32, nodes: u32, start: f32, unit: f32) -> f32 {
    let jitter = (unit * 2.0 - 1.0) * NODE_JITTER;
    let even = (node as f32 + 0.5 + jitter) / nodes as f32;
    let start = start.clamp(0.0, 1.0);
    start + (1.0 - start) * even
}

/// The limbs already on a parent, as a node about to be placed at `at` metres along it sees them.
struct Crowd<'a> {
    at: f32,
    /// The mean gap between nodes, in metres, over which crowding fades by `CROWDING_FADE`.
    gap: f32,
    /// Each limb's distance along the parent, in metres, and its azimuth.
    placed: &'a [(f32, f32)],
}

/// The azimuths of a node's `per_node` limbs, as `facing` places them before their skew:
/// - around, spaced evenly, turned to the least cost of `AZIMUTH_CANDIDATES` turns;
/// - to the sides, on alternate flanks, the first on the flank of lower cost;
/// - up or down, all on the parent's upper side or its underside.
///
/// A turn's cost is how crowded its limbs are by those already placed, less how far they point
/// out from the plant's axis. Crowding is how near each limb's azimuth is to each placed one's,
/// sharpened, and faded with their distance along the parent; so around, one limb a node settles
/// into a spiral of about 137.5°, pairs cross and whorls alternate, as phyllotaxis does.
fn node_azimuths(
    frame: &Frame,
    per_node: u32,
    facing: Facing,
    crowd: &Crowd,
    envelope: &Envelope,
) -> Vec<f32> {
    let across = frame.side.cross(frame.tangent);
    // The azimuth of the parent's upper side, nearest world up; an upright parent's is its side's.
    let up = Vec3::Y.reject_from_normalized(frame.tangent);
    let upper = if up.length_squared() > 1e-6 {
        up.dot(across).atan2(up.dot(frame.side))
    } else {
        0.0
    };
    // Out from the axis, as much as the node is off it, up to a crown radius.
    let out = Vec3::new(frame.point.x, 0.0, frame.point.z) / envelope.radius.max(0.01);
    let out = out.clamp_length_max(1.0);
    let cost = |azimuth: f32| {
        let crowding: f32 = crowd
            .placed
            .iter()
            .map(|&(at, other)| {
                let near = (1.0 + (azimuth - other).cos()) * 0.5;
                let fade = CROWDING_FADE.powf((crowd.at - at).abs() / crowd.gap.max(1e-3));
                fade * near.powi(CROWDING_SHARPNESS)
            })
            .sum();
        let (sin, cos) = azimuth.sin_cos();
        crowding - OUTWARD_WEIGHT * (frame.side * cos + across * sin).dot(out)
    };
    // The limbs from a first azimuth, `apart` between each, and the first of `candidates` whose
    // limbs cost least together.
    let group = |first: f32, apart: f32| -> Vec<f32> {
        (0..per_node)
            .map(|limb| first + apart * limb as f32)
            .collect()
    };
    let least = |candidates: &mut dyn Iterator<Item = f32>, apart: f32| -> Vec<f32> {
        let mut best = (0.0, f32::INFINITY);
        for first in candidates {
            let total: f32 = group(first, apart).into_iter().map(cost).sum();
            if total < best.1 {
                best = (first, total);
            }
        }
        group(best.0, apart)
    };
    match facing {
        Facing::Around => least(
            &mut (0..AZIMUTH_CANDIDATES).map(|i| TAU * i as f32 / AZIMUTH_CANDIDATES as f32),
            TAU / per_node as f32,
        ),
        Facing::Sides => least(&mut [upper + FRAC_PI_2, upper - FRAC_PI_2].into_iter(), PI),
        Facing::Up => group(upper, 0.0),
        Facing::Down => group(upper + PI, 0.0),
    }
}

/// A value drawn from `fuzzy`, clamped to the limits of `LimbLevel`'s field `field`.
fn draw(fuzzy: &Fuzzy, field: &str, rng: &mut Rng) -> f32 {
    let limits = LimbLevel::limits(field);
    fuzzy.sample(rng).clamp(*limits.start(), *limits.end())
}

/// `from` turned toward `toward` by at most `angle`, both unit vectors.
fn turn_toward(from: Vec3, toward: Vec3, angle: f32) -> Vec3 {
    let between = from.angle_between(toward);
    if between < 1e-6 {
        return from;
    }
    let axis = from
        .cross(toward)
        .try_normalize()
        .unwrap_or_else(|| from.any_orthonormal_vector());
    Quat::from_axis_angle(axis, between.min(angle)) * from
}

/// The unit vector square to `direction` nearest world up: a limb's upper side. A limb pointing
/// straight up or down has none, and takes its parent's side, made square to it.
fn upper_side(direction: Vec3, parent_side: Vec3) -> Vec3 {
    let up = Vec3::Y.reject_from_normalized(direction);
    if up.length_squared() > 1e-6 {
        up.normalize()
    } else {
        parent_side
            .reject_from_normalized(direction)
            .try_normalize()
            .unwrap_or_else(|| direction.any_orthonormal_vector())
    }
}

/// A stem's rings, kept to `tolerance` (`Trace::finish`). Its course is an arc (`arc`) turned
/// about the vertical to `heading`, which lies in one vertical plane, so a side square to it in
/// that plane never twists: azimuth 0 is toward the heading. The stem kinks about its course as a
/// limb does (`Kinks`).
fn stem_rings(
    length: f32,
    splay: f32,
    lean: f32,
    heading: f32,
    mut kinks: Kinks,
    tolerance: f32,
) -> Vec<Ring> {
    let turn = Quat::from_rotation_y(heading);
    let course_at = |at: f32| {
        let (_, tangent) = arc(length, splay, lean, at / length);
        (turn * tangent, turn * tangent.cross(Vec3::Z))
    };
    let (tangent, side) = course_at(0.0);
    let mut trace = Trace::new(Ring {
        center: Vec3::ZERO,
        tangent,
        side,
        at: 0.0,
    });
    let (mut center, mut tangent, mut at) = (Vec3::ZERO, tangent, 0.0);
    while at < length {
        let step = kinks.step(at, TRACE_STEP.min(length - at));
        center += tangent * step;
        at += step;
        let (course, course_side) = course_at(at);
        tangent = kinks.direction(at, length, course, course_side);
        trace.push(center, at, tangent);
    }
    trace.finish(tolerance)
}

/// The distance between kinks on any limb or stem: `KINK_SPACING` crown radii, so a scaled
/// species keeps its form.
fn kink_spacing(envelope: &Envelope) -> f32 {
    KINK_SPACING * envelope.radius.max(0.01)
}

/// Where a limb or stem kinks, and which way each stretch between kinks points off its course: up
/// to `wander` from it, about a random axis, held in the course's frame so it turns with the
/// course. Each stretch is `spacing` long on average, give or take `KINK_JITTER` of it, so kinks
/// fall unevenly. The first stretch follows the course. Offsets are from the course, not from the
/// stretch before, so the limb zigzags about its course rather than curling away from it.
struct Kinks {
    spacing: f32,
    wander: f32,
    next: f32,
    offset: Quat,
    rng: Rng,
}

impl Kinks {
    fn new(spacing: f32, wander: f32, rng: Rng) -> Self {
        let mut kinks = Self {
            spacing,
            wander,
            next: 0.0,
            offset: Quat::IDENTITY,
            rng,
        };
        kinks.next = kinks.stretch();
        kinks
    }

    /// The length of the next stretch, drawn.
    fn stretch(&mut self) -> f32 {
        let jitter = (self.rng.unit() * 2.0 - 1.0) * KINK_JITTER;
        (self.spacing * (1.0 + jitter)).max(TRACE_STEP)
    }

    /// `step`, cut short to land on the next kink.
    fn step(&self, at: f32, step: f32) -> f32 {
        step.min(self.next - at)
    }

    /// The direction at `at`, `length` being where the limb ends, given its course there and a
    /// side square to the course. A kink falling there draws the next offset.
    fn direction(&mut self, at: f32, length: f32, course: Vec3, side: Vec3) -> Vec3 {
        if at >= self.next - 1e-5 && at < length {
            self.next += self.stretch();
            let tilt = self.rng.unit() * self.wander;
            let around = self.rng.unit() * TAU;
            self.offset = Quat::from_axis_angle(Vec3::new(around.cos(), around.sin(), 0.0), tilt);
        }
        let frame = Quat::from_mat3(&Mat3::from_cols(side, course.cross(side), course));
        (frame * self.offset * Vec3::Z).normalize()
    }
}

/// A tube as it is traced: a point every step, from its start ring, and the direction it was
/// heading at the last.
struct Trace {
    start: Ring,
    /// Each point's center and distance along the tube, the start's first.
    points: Vec<(Vec3, f32)>,
    tip_tangent: Vec3,
}

impl Trace {
    fn new(start: Ring) -> Self {
        Self {
            start,
            points: vec![(start.center, 0.0)],
            tip_tangent: start.tangent,
        }
    }

    /// The point a step on, at `center` and `at`, the tube heading `tangent` there.
    fn push(&mut self, center: Vec3, at: f32, tangent: Vec3) {
        self.points.push((center, at));
        self.tip_tangent = tangent;
    }

    /// The tube's rings: the points kept by `simplify` at `tolerance`, less any within a step of
    /// the tip (`end_at`). The start ring is kept as it is, and the tip heads as the tube did; each
    /// ring between faces halfway between the chords meeting there, so the tube does not pinch,
    /// with its side carried along from the start's without twisting.
    fn finish(self, tolerance: f32) -> Vec<Ring> {
        let centers: Vec<Vec3> = self.points.iter().map(|(center, _)| *center).collect();
        let mut kept = simplify(&centers, tolerance);
        let tip = self.points.len() - 1;
        let tip_at = self.points[tip].1;
        kept.retain(|&i| i == 0 || i == tip || self.points[i].1 < tip_at - TRACE_STEP);
        let mut rings = vec![self.start];
        for (k, &i) in kept.iter().enumerate().skip(1) {
            let (center, at) = self.points[i];
            let previous = rings[rings.len() - 1];
            let into = (center - previous.center).normalize_or(previous.tangent);
            let tangent = match kept.get(k + 1) {
                Some(&next) => {
                    (into + (self.points[next].0 - center).normalize_or(into)).normalize_or(into)
                }
                None => self.tip_tangent,
            };
            let side = (Quat::from_rotation_arc(previous.tangent, tangent) * previous.side)
                .reject_from_normalized(tangent)
                .normalize_or(previous.side);
            rings.push(Ring {
                center,
                tangent,
                side,
                at,
            });
        }
        rings
    }
}

/// The indices of the points of a polyline that Ramer, Douglas and Peucker keep at `tolerance`:
/// both ends, and between two kept points the one furthest from the chord joining them, if it lies
/// further than `tolerance`, and so on within each half. What is left out lies within `tolerance`
/// of the polyline kept.
fn simplify(points: &[Vec3], tolerance: f32) -> Vec<usize> {
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut spans = vec![(0, points.len() - 1)];
    while let Some((a, b)) = spans.pop() {
        let furthest = (a + 1..b)
            .map(|i| (i, distance_to_segment(points[i], points[a], points[b])))
            .max_by(|x, y| x.1.total_cmp(&y.1));
        if let Some((i, distance)) = furthest
            && distance > tolerance
        {
            keep[i] = true;
            spans.push((a, i));
            spans.push((i, b));
        }
    }
    (0..points.len()).filter(|&i| keep[i]).collect()
}

/// How far `point` lies from the segment from `a` to `b`.
fn distance_to_segment(point: Vec3, a: Vec3, b: Vec3) -> f32 {
    let along = b - a;
    let t = if along.length_squared() > 0.0 {
        ((point - a).dot(along) / along.length_squared()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (a + along * t - point).length()
}

/// Ends a tube's rings at `tip`, a ring along it. Rings past it go, and so do rings less than a
/// trace step before it, but never the base: a tube turning between two rings that close folds
/// over itself where it is drawn thicker than the gap.
fn end_at(rings: &mut Vec<Ring>, tip: Ring) {
    rings.retain(|ring| ring.at == 0.0 || ring.at < tip.at - TRACE_STEP);
    if tip.at > rings[rings.len() - 1].at {
        rings.push(tip);
    }
}

/// Adds a tube through `rings`, each turned to its ring's side, so the tube rolls only as the
/// sides do. The radius is `radii.0` from the start to `narrows_from` along it, then narrows to
/// `radii.1` at the end, which a fan closes if `cap` is set. The start is left open.
fn tube(
    geometry: &mut Geometry,
    rings: &[Ring],
    radii: (f32, f32),
    narrows_from: f32,
    sides: u32,
    cap: bool,
) {
    let first = geometry.positions.len() as u32;
    let (base_radius, top_radius) = radii;
    let length = rings[rings.len() - 1].at;
    let narrowing = (length - narrows_from).max(1e-4);
    // A tapering tube's surface leans in toward the end, so its normals lean forward by the same
    // angle.
    let lean = ((base_radius - top_radius) / narrowing).atan();

    for ring in rings {
        let across = ring.side.cross(ring.tangent);
        let past = ring.at - narrows_from;
        let radius = base_radius + (top_radius - base_radius) * past.max(0.0) / narrowing;
        let (lean_sin, lean_cos) = if past < -1e-4 { 0.0 } else { lean }.sin_cos();
        for side in 0..sides {
            let (sin, cos) = (TAU * side as f32 / sides as f32).sin_cos();
            let outward = ring.side * cos + across * sin;
            geometry.positions.push(ring.center + outward * radius);
            geometry
                .normals
                .push((outward * lean_cos + ring.tangent * lean_sin).normalize());
        }
    }
    let segments = rings.len() as u32 - 1;
    for ring in 0..segments {
        for side in 0..sides {
            let a = first + ring * sides + side;
            let b = first + ring * sides + (side + 1) % sides;
            let (c, d) = (a + sides, b + sides);
            geometry.indices.extend([a, c, d, a, d, b]);
        }
    }
    if !cap {
        return;
    }

    let end = rings[rings.len() - 1];
    let tip = geometry.positions.len() as u32;
    geometry.positions.push(end.center);
    geometry.normals.push(end.tangent);
    let last = first + segments * sides;
    for side in 0..sides {
        geometry
            .indices
            .extend([last + side, tip, last + (side + 1) % sides]);
    }
}

/// The envelope's width at `t`, 0 at the crown base to 1 at the top, as a fraction of its radius.
pub fn profile_width(profile: Profile, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match profile {
        Profile::Cone => 1.0 - t,
        Profile::Ellipsoid => (1.0 - (2.0 * t - 1.0).powi(2)).max(0.0).sqrt(),
        Profile::Hemisphere => (1.0 - t * t).max(0.0).sqrt(),
        Profile::Column => 1.0,
        // Weber and Penn's flame.
        Profile::Flame if t <= 0.7 => t / 0.7,
        Profile::Flame => (1.0 - t) / 0.3,
    }
}

/// The envelope drawn as lines: meridians closed across the base and the top, and rings around
/// it. Pairs of points, a segment each.
pub fn envelope_outline(envelope: &Envelope) -> Vec<Vec3> {
    let Envelope { base, top, .. } = *envelope;
    let profile: Vec<(f32, f32)> = (0..=OUTLINE_STEPS)
        .map(|step| {
            let t = step as f32 / OUTLINE_STEPS as f32;
            let width = envelope.radius * profile_width(envelope.profile, t);
            (width, base + (top - base) * t)
        })
        .collect();
    let mut lines = Vec::new();
    let mut segment = |a: Vec3, b: Vec3| {
        // A profile that closes to a point leaves no spoke at that end.
        if a.distance_squared(b) > 1e-8 {
            lines.extend([a, b]);
        }
    };

    for meridian in 0..OUTLINE_MERIDIANS {
        let (sin, cos) = (TAU * meridian as f32 / OUTLINE_MERIDIANS as f32).sin_cos();
        let point = |(width, y): (f32, f32)| Vec3::new(cos * width, y, sin * width);
        segment(Vec3::Y * base, point(profile[0]));
        for pair in profile.windows(2) {
            segment(point(pair[0]), point(pair[1]));
        }
        segment(point(profile[profile.len() - 1]), Vec3::Y * top);
    }

    for &(width, y) in profile.iter().step_by(OUTLINE_RING_EVERY as usize) {
        for side in 0..OUTLINE_RING_SIDES {
            let around = |side: u32| {
                let (sin, cos) = (TAU * side as f32 / OUTLINE_RING_SIDES as f32).sin_cos();
                Vec3::new(cos * width, y, sin * width)
            };
            segment(around(side), around(side + 1));
        }
    }
    lines
}

/// The envelope's radius at height `y`: 0 below the crown base and above the top.
pub fn envelope_radius(envelope: &Envelope, y: f32) -> f32 {
    let Envelope { base, top, .. } = *envelope;
    if y < base || y > top || top <= base {
        return 0.0;
    }
    envelope.radius * profile_width(envelope.profile, (y - base) / (top - base))
}

/// How far a ray from `origin` along the unit `direction` runs before it leaves the envelope, or 0
/// if `origin` is outside it. Every profile makes a convex solid, so the ray leaves it once, and
/// bisection between the origin and a point beyond its bounds finds where.
pub fn distance_to_envelope(envelope: &Envelope, origin: Vec3, direction: Vec3) -> f32 {
    let inside = |p: Vec3| inside_envelope(envelope, p);
    if !inside(origin) {
        return 0.0;
    }
    // Further than any two points of the envelope are apart.
    let (mut near, mut far) = (0.0, 2.0 * (envelope.top.abs() + envelope.radius.abs()));
    for _ in 0..ENVELOPE_BISECTIONS {
        let middle = (near + far) * 0.5;
        if inside(origin + direction * middle) {
            near = middle;
        } else {
            far = middle;
        }
    }
    near
}

/// Whether `point` lies inside the envelope.
fn inside_envelope(envelope: &Envelope, point: Vec3) -> bool {
    point.x.hypot(point.z) < envelope_radius(envelope, point.y)
}

/// How far `point` lies in from the envelope's side and top, or `None` outside it. The crown's
/// base is not counted as surface, so a point just above it is as deep as the side makes it. The
/// distance is to the profile in the point's meridian plane, as a line through `DEPTH_STEPS` points
/// and across the top to the axis, which a flat top needs.
fn depth_in_envelope(envelope: &Envelope, point: Vec3) -> Option<f32> {
    if !inside_envelope(envelope, point) {
        return None;
    }
    let Envelope { base, top, .. } = *envelope;
    let at = Vec2::new(point.x.hypot(point.z), point.y);
    let profile = (0..=DEPTH_STEPS).map(|step| {
        let t = step as f32 / DEPTH_STEPS as f32;
        Vec2::new(
            envelope.radius * profile_width(envelope.profile, t),
            base + (top - base) * t,
        )
    });
    let outline: Vec<Vec2> = profile.chain([Vec2::new(0.0, top)]).collect();
    outline
        .windows(2)
        .map(|pair| {
            let (a, b) = (pair[0], pair[1]);
            let along = (at - a).dot(b - a) / (b - a).length_squared().max(1e-12);
            at.distance(a + (b - a) * along.clamp(0.0, 1.0))
        })
        .reduce(f32::min)
}

/// The point and unit tangent at `t` (0 to 1) along an arc of length `length` that starts at the
/// origin `start` radians from up toward +X and turns by `bend` radians further toward +X.
fn arc(length: f32, start: f32, bend: f32, t: f32) -> (Vec3, Vec3) {
    let angle = start + bend * t;
    let (sin, cos) = angle.sin_cos();
    let tangent = Vec3::new(sin, cos, 0.0);
    if bend.abs() < 1e-4 {
        return (tangent * length * t, tangent);
    }
    let radius = length / bend;
    let (start_sin, start_cos) = start.sin_cos();
    (
        Vec3::new(radius * (start_cos - cos), radius * (sin - start_sin), 0.0),
        tangent,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fuzzy::Distribution,
        random::Seed,
        species::{Bark, Trunk},
    };
    use bevy::math::Vec3Swizzles;

    /// A lone 5 m stem's rings, standing straight up and leaning toward +X, with no wander, kept
    /// to `tolerance`.
    fn lone_stem(lean_degrees: f32, tolerance: f32) -> Vec<Ring> {
        let kinks = Kinks::new(1.0, 0.0, Rng::stream(0, "test"));
        stem_rings(5.0, 0.0, lean_degrees.to_radians(), 0.0, kinks, tolerance)
    }

    /// `lone_stem`'s mesh, kept to 2 cm.
    fn stem_mesh(sides: u32, lean_degrees: f32) -> Geometry {
        let mut geometry = Geometry::default();
        tube(
            &mut geometry,
            &lone_stem(lean_degrees, 0.02),
            (0.2, 0.1),
            0.0,
            sides,
            true,
        );
        geometry
    }

    fn trunk_with(sides: i32, lean_degrees: f32) -> Trunk {
        Trunk {
            sides,
            lean: lean_degrees.to_radians(),
            wander: 0.0,
            ..Trunk::default()
        }
    }

    #[test]
    fn rings_follow_visible_error() {
        // Straight: one segment of 6 quads and a 6-triangle cap.
        assert_eq!(stem_mesh(6, 0.0).triangles(), 6 * 2 + 6);
        // Bent, more rings the finer the detail; none closer to the tip than a step.
        let rings = |tolerance: f32| lone_stem(90.0, tolerance);
        let (coarse, fine) = (rings(0.05), rings(0.002));
        assert!(coarse.len() > 2 && fine.len() > coarse.len() + 3);
        for rings in [&coarse, &fine] {
            let before_tip = rings[rings.len() - 2].at;
            assert!(before_tip < 5.0 - TRACE_STEP, "{before_tip}");
        }
    }

    #[test]
    fn simplified_lines_stay_within_the_tolerance() {
        // A zigzag 4 cm high and 10 cm apart: at 2 cm every corner is kept, at 5 cm none.
        let zigzag: Vec<Vec3> = (0..=20)
            .map(|i| Vec3::new(i as f32 * 0.05, (i % 2) as f32 * 0.04, 0.0))
            .collect();
        assert_eq!(simplify(&zigzag, 0.02).len(), zigzag.len());
        assert_eq!(simplify(&zigzag, 0.05), vec![0, 20]);
        // A bend: what is left out lies within the tolerance of what is kept.
        let bend: Vec<Vec3> = (0..=100)
            .map(|i| {
                let a = i as f32 * 0.015;
                Vec3::new(a.sin(), 1.0 - a.cos(), 0.0)
            })
            .collect();
        let kept = simplify(&bend, 0.01);
        assert!(kept.len() > 2 && kept.len() < 20, "{kept:?}");
        for point in &bend {
            let near = kept
                .windows(2)
                .map(|pair| distance_to_segment(*point, bend[pair[0]], bend[pair[1]]))
                .fold(f32::INFINITY, f32::min);
            assert!(near <= 0.01 + 1e-6, "{near}");
        }
    }

    #[test]
    fn vertices_are_shared() {
        let rings = lone_stem(30.0, 0.02);
        let geometry = stem_mesh(5, 30.0);
        // A ring of five for each ring, and the tip.
        assert_eq!(geometry.positions.len(), rings.len() * 5 + 1);
        assert_eq!(geometry.normals.len(), geometry.positions.len());
    }

    #[test]
    fn base_stands_on_the_origin() {
        let geometry = stem_mesh(6, 40.0);
        let base = &geometry.positions[..6];
        let center = base.iter().copied().sum::<Vec3>() / 6.0;
        assert!(center.length() < 1e-5);
        assert!(base.iter().all(|p| p.y.abs() < 1e-5));
    }

    #[test]
    fn a_straight_stem_reaches_its_height() {
        let mut species = regular();
        species.trunk.lean = 0.0;
        species.trunk.height = 3.0;
        let tip = stems(&species)[0].rings.last().unwrap().center;
        assert!((tip - Vec3::Y * 3.0).length() < 1e-5, "{tip}");
    }

    #[test]
    fn the_crown_moves_without_the_trunk() {
        let mut species = regular();
        species.trunk.lean = 0.0;
        let tip = |species: &Species| stems(species)[0].rings.last().unwrap().center;
        let before = (tip(&species), Envelope::of(&species));
        species.crown.top = 1.8;
        let after = (tip(&species), Envelope::of(&species));
        assert_eq!(before.0, after.0);
        assert_eq!(before.1.base, after.1.base);
        assert!((after.1.top - 1.8 * species.trunk.height).abs() < 1e-5);
    }

    #[test]
    fn stems_never_turn_past_level() {
        let mut species = regular();
        species.trunk.splay = 50.0_f32.to_radians();
        species.trunk.lean = 90.0_f32.to_radians();
        let tip = stems(&species)[0].rings.last().unwrap().tangent;
        assert!(tip.y > -1e-5, "{tip}");
    }

    #[test]
    fn slant_bends_a_stem_without_lengthening_it() {
        let length = |splay: f32, lean: f32| {
            let mut species = Species::default();
            species.trunk.splay = splay.to_radians();
            species.trunk.lean = lean.to_radians();
            stems(&species)[0].rings.last().unwrap().at
        };
        let upright = length(0.0, 0.0);
        for (splay, lean) in [(0.0, 45.0), (60.0, 0.0), (60.0, 90.0)] {
            assert!(
                (length(splay, lean) - upright).abs() < 1e-3,
                "{splay} {lean}"
            );
        }
    }

    #[test]
    fn stems_splay_evenly_around() {
        let mut species = Species::default();
        species.trunk.stems = 3;
        species.trunk.splay = 30.0_f32.to_radians();
        species.trunk.lean = 0.0;
        let stems = stems(&species);
        assert_eq!(stems.len(), 3);
        let directions: Vec<Vec3> = stems.iter().map(|stem| stem.rings[0].tangent).collect();
        for direction in &directions {
            assert!((direction.angle_between(Vec3::Y).to_degrees() - 30.0).abs() < 1e-3);
        }
        let heading = |v: Vec3| v.z.atan2(v.x);
        for pair in directions.windows(2) {
            let apart = (heading(pair[0]) - heading(pair[1])).rem_euclid(TAU);
            assert!((apart - TAU / 3.0).abs() < 1e-4, "{}", apart.to_degrees());
        }
    }

    #[test]
    fn the_seed_turns_the_lean() {
        let tip = |seed| {
            let species = Species {
                seed: Seed(seed),
                ..regular()
            };
            stems(&species)[0].rings.last().unwrap().center
        };
        // The same height and lean, in another compass direction.
        let (one, two) = (tip(1), tip(2));
        assert!((one.y - two.y).abs() < 1e-5);
        assert!((one.xz().length() - two.xz().length()).abs() < 1e-5);
        assert!(one.xz().angle_to(two.xz()).abs() > 0.1, "{one} {two}");
    }

    #[test]
    fn profiles_close_where_their_shapes_do() {
        let ends = |profile| (profile_width(profile, 0.0), profile_width(profile, 1.0));
        assert_eq!(ends(Profile::Cone), (1.0, 0.0));
        assert_eq!(ends(Profile::Ellipsoid), (0.0, 0.0));
        assert_eq!(ends(Profile::Hemisphere), (1.0, 0.0));
        assert_eq!(ends(Profile::Column), (1.0, 1.0));
        assert_eq!(ends(Profile::Flame), (0.0, 0.0));
        assert!((profile_width(Profile::Flame, 0.7) - 1.0).abs() < 1e-6);
        assert!((profile_width(Profile::Ellipsoid, 0.5) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn outline_skips_spokes_and_rings_of_no_width() {
        let segments = |profile| {
            let envelope = Envelope {
                profile,
                ..Envelope::default()
            };
            envelope_outline(&envelope).len() / 2
        };
        // Nine rings of 24, and 8 meridians of 16 steps. The ellipsoid closes at both ends, so it
        // loses the end rings and both spokes; the cone only at the top.
        assert_eq!(segments(Profile::Ellipsoid), 7 * 24 + 8 * 16);
        assert_eq!(segments(Profile::Cone), 8 * 24 + 8 * 17);
        assert_eq!(segments(Profile::Column), 9 * 24 + 8 * 18);
    }

    #[test]
    fn envelope_radius_is_zero_outside_the_crown() {
        let envelope = Envelope {
            profile: Profile::Column,
            base: 1.8,
            top: 6.0,
            ..Envelope::default()
        };
        assert_eq!(envelope_radius(&envelope, 1.7), 0.0);
        assert_eq!(envelope_radius(&envelope, 3.0), envelope.radius);
        assert_eq!(envelope_radius(&envelope, 6.1), 0.0);
    }

    #[test]
    fn rays_leave_the_envelope_at_its_surface() {
        let envelope = Envelope {
            profile: Profile::Column,
            base: 1.8,
            top: 6.0,
            ..Envelope::default()
        };
        let origin = Vec3::Y * 3.0;
        assert!((distance_to_envelope(&envelope, origin, Vec3::X) - envelope.radius).abs() < 1e-3);
        // Straight up, out through the top.
        assert!((distance_to_envelope(&envelope, origin, Vec3::Y) - 3.0).abs() < 1e-3);
        // From below the crown base, nothing.
        assert_eq!(distance_to_envelope(&envelope, Vec3::Y, Vec3::X), 0.0);
    }

    /// The default species with every level's limbs straight and nothing varied.
    fn regular() -> Species {
        let mut species = Species::default();
        species.trunk.wander = 0.0;
        for level in &mut species.limbs {
            level.bend = 0.0;
            level.wander = 0.0;
            level.skew = 0.0;
            for fuzzy in [&mut level.angle, &mut level.reach] {
                fuzzy.spread = 0.0;
            }
        }
        species
    }

    /// `regular`, with an upright stem in a column crown: the stem is inside the crown from the
    /// crown base at 1.25 m to its tip at 5.25 m, two crown radii of 2 m.
    fn straight() -> Species {
        let mut species = regular();
        species.trunk.lean = 0.0;
        species.trunk.height = 5.25;
        species.crown.profile = Profile::Column;
        species.crown.base = 1.25 / 5.25;
        species.crown.top = 6.25 / 5.25;
        species.crown.radius = 2.0;
        species
    }

    /// The limbs leaving their parents' sides, and those carrying their parents on, each starting
    /// on a parent's tip, turned as it is.
    fn sides_and_carriers(species: &Species) -> (Vec<Limb>, Vec<Limb>) {
        let (stems, limbs) = grow(species);
        let tips: Vec<Ring> = stems
            .iter()
            .filter(|stem| stem.carried_on)
            .map(|stem| &stem.rings)
            .chain(
                limbs
                    .iter()
                    .filter(|limb| limb.carried_on)
                    .map(|limb| &limb.rings),
            )
            .map(|rings| rings[rings.len() - 1])
            .collect();
        limbs.into_iter().partition(|limb| {
            !tips
                .iter()
                .any(|tip| tip.center == limb.origin() && tip.tangent == limb.direction())
        })
    }

    fn side_limbs(species: &Species) -> Vec<Limb> {
        sides_and_carriers(species).0
    }

    /// A stem's inside part, as heights.
    fn inside_heights(species: &Species) -> (f32, f32) {
        let stem = &stems(species)[0];
        let parent = Parent {
            rings: &stem.rings,
            radius: stem.radii.0,
            rng: &stem.rng,
        };
        let (start, end) = parent
            .inside_part(&Envelope::of(species))
            .expect("a stem inside the crown");
        (parent.frame(start).point.y, parent.frame(end).point.y)
    }

    #[test]
    fn the_straight_stem_is_inside_where_it_says() {
        let (start, end) = inside_heights(&straight());
        assert!(
            (start - 1.25).abs() < 1e-4 && (end - 5.25).abs() < 1e-4,
            "{start} {end}"
        );
    }

    #[test]
    fn limbs_reach_their_share_of_the_way_to_the_envelope() {
        let species = straight();
        let limbs = limbs(&species);
        // Sixteen, and the one carrying the stem on, which reaches the same way.
        assert_eq!(limbs.len(), 17);
        for limb in &limbs {
            assert_eq!(limb.rings.len(), 2, "a straight limb is one segment");
            let tip = limb.origin() + limb.direction() * limb.length();
            let rest = distance_to_envelope(&Envelope::of(&species), tip, limb.direction());
            let whole = limb.length() + rest;
            assert!((limb.length() - species.limbs[0].reach.center * whole).abs() < 1e-3);
        }
    }

    #[test]
    fn limbs_leave_the_trunk_at_their_angle() {
        // Upright, the trunk is one straight segment, base to tip.
        let mut species = regular();
        species.trunk.lean = 0.0;
        let tip = stems(&species)[0].rings.last().unwrap().center;
        for limb in side_limbs(&species) {
            let angle = limb.direction().angle_between(tip);
            assert!(
                (angle - species.limbs[0].angle.center).abs() < 1e-5,
                "{limb:?}"
            );
        }
    }

    #[test]
    fn varied_values_stay_within_their_spread_and_limits() {
        let mut species = regular();
        species.trunk.lean = 0.0;
        species.limbs[0].angle = Fuzzy::new(
            60.0_f32.to_radians(),
            10.0_f32.to_radians(),
            Distribution::Uniform,
        );
        // Reaching past the envelope is clamped to reaching it.
        species.limbs[0].reach = Fuzzy::new(0.95, 0.2, Distribution::Uniform);
        let tip = stems(&species)[0].rings.last().unwrap().center;
        let limbs = side_limbs(&species);
        let angles: Vec<f32> = limbs
            .iter()
            .map(|limb| limb.direction().angle_between(tip).to_degrees())
            .collect();
        assert!(
            angles.iter().all(|angle| (49.99..=70.01).contains(angle)),
            "{angles:?}"
        );
        assert!(
            angles.iter().any(|angle| (angle - angles[0]).abs() > 0.1),
            "{angles:?}"
        );
        for limb in &limbs {
            let whole =
                distance_to_envelope(&Envelope::of(&species), limb.origin(), limb.direction());
            assert!(limb.length() <= whole + 1e-3);
        }
    }

    #[test]
    fn deeper_levels_grow_on_the_level_before() {
        let mut species = regular();
        let first = limbs(&species);
        species.limb_levels = 2;
        let both = limbs(&species);
        // The first level is unchanged but for its tips, which its limbs narrow and cut short at
        // the last of them, and the second follows it.
        for (before, after) in first.iter().zip(&both) {
            assert_eq!(
                (before.rings[0], before.radii.0),
                (after.rings[0], after.radii.0)
            );
            assert!(after.length() <= before.length());
        }
        let second: Vec<Limb> = side_limbs(&species)
            .into_iter()
            .filter(|limb| limb.level == 1)
            .collect();
        assert!(!second.is_empty());
        for limb in &second {
            assert_eq!(limb.level, 1);
            // Starts on a first-level limb, at the level's angle from it.
            let parent = first
                .iter()
                .find(|parent| {
                    let from = limb.origin() - parent.origin();
                    from.cross(parent.direction()).length() < 1e-3
                        && from.dot(parent.direction()) >= 0.0
                        && from.length() <= parent.length()
                })
                .expect("a parent under the limb");
            let angle = limb.direction().angle_between(parent.direction());
            assert!((angle - species.limbs[1].angle.center).abs() < 1e-3);
        }
    }

    #[test]
    fn limbs_bend_toward_their_bend() {
        let mut species = regular();
        species.limbs[0].bend = 0.5;
        let target = species.limbs[0].bend_toward;
        let crown = species.crown.radius;
        for limb in limbs(&species) {
            let tip = limb.rings[limb.rings.len() - 1].tangent;
            // Half the angle left is closed over each crown radius of length.
            let left_at_base = target - limb.direction().angle_between(Vec3::Y);
            let left_at_tip = target - tip.angle_between(Vec3::Y);
            let expected = left_at_base * 0.5_f32.powf(limb.length() / crown);
            assert!(
                (left_at_tip - expected).abs() < 1e-3,
                "{left_at_tip} {expected}"
            );
            // The compass heading is kept.
            let heading = |v: Vec3| v.z.atan2(v.x);
            assert!((heading(tip) - heading(limb.direction())).abs() < 1e-3);
        }
    }

    #[test]
    fn every_bend_target_moves_the_tip() {
        let tips = |degrees: f32| {
            let mut species = regular();
            species.limbs[0].bend = 0.5;
            species.limbs[0].bend_toward = degrees.to_radians();
            limbs(&species)
                .iter()
                .map(|limb| limb.rings[limb.rings.len() - 1].tangent)
                .collect::<Vec<_>>()
        };
        let mut previous = tips(90.0);
        for degrees in [110.0, 130.0, 150.0, 170.0] {
            let next = tips(degrees);
            // Each limb's tip droops further, or the limb stops sooner at the envelope.
            assert_ne!(previous, next, "{degrees}°");
            previous = next;
        }
    }

    #[test]
    fn default_plants_stay_within_budget() {
        // Orders of magnitude, not counts: the low hundreds for one level; two, the usual tree,
        // under seedling's trees (about 2,500); three, the costly exception, a little over them,
        // every twig drawn.
        let triangles = |limb_levels| {
            let species = Species {
                limb_levels,
                ..Species::default()
            };
            plant(&species).bark.triangles()
        };
        assert!((100..1_000).contains(&triangles(1)), "{}", triangles(1));
        assert!(triangles(2) < 2_500, "{}", triangles(2));
        assert!(triangles(3) < 3_500, "{}", triangles(3));
    }

    #[test]
    fn rings_advance_along_every_limb() {
        let species = Species {
            limb_levels: 3,
            ..Species::default()
        };
        for limb in limbs(&species) {
            for pair in limb.rings.windows(2) {
                assert!(pair[1].at > pair[0].at);
            }
        }
    }

    #[test]
    fn limbs_kink_at_one_spacing_on_every_level() {
        // Kinks every 30 cm on average on both levels, whatever their densities, each stretch
        // 18 to 42 cm: with no bend, the rings between a limb's ends can only be kinks, so a limb
        // has no more than its length over the shortest stretch.
        let mut species = level_limbs(45.0);
        species.limb_levels = 2;
        species.limbs[1].wander = 45.0_f32.to_radians();
        let spacing = kink_spacing(&Envelope::of(&species));
        assert!((spacing - 0.3).abs() < 1e-6);
        let shortest = spacing * (1.0 - KINK_JITTER);
        let limbs = limbs(&species);
        for limb in &limbs {
            let kinks = (limb.length() / shortest).floor() as usize;
            assert!(
                limb.rings.len() - 2 <= kinks,
                "{} {}",
                limb.rings.len(),
                limb.length()
            );
        }
        assert!(
            limbs
                .iter()
                .any(|limb| limb.level == 0 && limb.rings.len() >= 6)
        );
        assert!(
            limbs
                .iter()
                .any(|limb| limb.level == 1 && limb.length() < shortest)
        );
        // Unevenly: the stretches between one limb's kinks differ.
        let stretches: Vec<f32> = limbs
            .iter()
            .filter(|limb| limb.level == 0)
            .flat_map(|limb| {
                limb.rings[..limb.rings.len() - 1]
                    .windows(2)
                    .map(|pair| pair[1].at - pair[0].at)
                    .collect::<Vec<_>>()
            })
            .collect();
        let (low, high) = stretches
            .iter()
            .fold((f32::INFINITY, 0.0_f32), |(low, high), s| {
                (low.min(*s), high.max(*s))
            });
        assert!(high - low > 0.1, "{low} {high}");
    }

    /// `straight`, its first level's limbs leaving level and running straight to the column's
    /// wall 2 m away, but for wandering by `wander` degrees.
    fn level_limbs(wander: f32) -> Species {
        let mut species = straight();
        species.limbs[0].angle = Fuzzy::new(FRAC_PI_2, 0.0, Distribution::Uniform);
        species.limbs[0].reach = Fuzzy::new(1.0, 0.0, Distribution::Uniform);
        species.limbs[0].wander = wander.to_radians();
        species
    }

    #[test]
    fn wander_zigzags_about_the_course() {
        // With no bend the course is the limb's start, and every internode stays within the
        // wander of it, however many kinks there are: the offsets do not add up.
        let species = level_limbs(45.0);
        let limbs = limbs(&species);
        for limb in &limbs {
            let course = limb.direction();
            for ring in &limb.rings {
                let off = ring.tangent.angle_between(course).to_degrees();
                assert!(off <= 45.0 + 1e-2, "{off}");
            }
        }
        assert!(limbs.iter().any(|limb| limb.rings.len() > 4));
    }

    #[test]
    fn limbs_stay_inside_the_envelope() {
        let species = Species {
            limb_levels: 3,
            ..Species::default()
        };
        for limb in limbs(&species) {
            for ring in &limb.rings[1..] {
                assert!(
                    inside_envelope(&Envelope::of(&species), ring.center),
                    "{ring:?}"
                );
            }
        }
    }

    #[test]
    fn the_seed_sets_the_wander() {
        let positions = |seed| {
            let species = Species {
                seed: Seed(seed),
                limb_levels: 2,
                ..Species::default()
            };
            plant(&species).bark.positions
        };
        assert_eq!(positions(1), positions(1));
        assert_ne!(positions(1), positions(2));
    }

    #[test]
    fn hidden_levels_do_not_grow() {
        let mut species = Species::default();
        species.limbs[2].density = 40.0;
        let one = limbs(&species).len();
        species.limb_levels = 2;
        let two = limbs(&species);
        assert!(two.len() > one);
        assert!(two.iter().all(|limb| limb.level < 2));
        species.limb_levels = 3;
        assert!(limbs(&species).iter().any(|limb| limb.level == 2));
    }

    #[test]
    fn azimuth_zero_is_the_upper_side() {
        // A level limb's side is world up; one pointing straight up keeps its parent's.
        assert!((upper_side(Vec3::X, Vec3::Z) - Vec3::Y).length() < 1e-6);
        assert!((upper_side(Vec3::Y, Vec3::Z) - Vec3::Z).length() < 1e-6);
    }

    #[test]
    fn limbs_start_on_the_trunk_as_meshed() {
        // Upright, the trunk is one straight segment, base to tip.
        let mut species = regular();
        species.trunk.lean = 0.0;
        let tip = stems(&species)[0].rings.last().unwrap().center;
        for limb in limbs(&species) {
            assert!(limb.origin().cross(tip).length() < 1e-5, "{limb:?}");
        }
    }

    #[test]
    fn nodes_hold_their_limbs_evenly_around() {
        // Straight, so azimuth about the trunk is azimuth about world up.
        let mut species = straight();
        // Two nodes on the stem's two crown radii.
        species.limbs[0].density = 1.0;
        species.limbs[0].per_node = 3.0;
        let limbs = side_limbs(&species);
        assert_eq!(limbs.len(), 6);
        // A node's limbs share an origin and stand a third of a turn apart.
        assert_eq!(limbs[0].origin(), limbs[2].origin());
        let azimuth = |limb: &Limb| limb.direction().z.atan2(limb.direction().x);
        let apart = (azimuth(&limbs[1]) - azimuth(&limbs[0])).rem_euclid(TAU);
        assert!((apart - TAU / 3.0).abs() < 1e-5);
    }

    /// The turns between successive nodes on `straight`'s upright stem, in degrees, modulo the
    /// angle between a node's limbs, since they are alike. The first few are left out, while the
    /// pattern settles.
    fn node_turns(per_node: f32) -> Vec<f32> {
        let mut species = straight();
        species.limbs[0].density = 8.0;
        species.limbs[0].per_node = per_node;
        let limbs = side_limbs(&species);
        let per_node = per_node as usize;
        assert_eq!(limbs.len(), 16 * per_node);
        let azimuth = |limb: &Limb| limb.direction().z.atan2(limb.direction().x).to_degrees();
        let between = 360.0 / per_node as f32;
        limbs
            .chunks(per_node)
            .map(|node| azimuth(&node[0]))
            .collect::<Vec<_>>()
            .windows(2)
            .skip(3)
            .map(|pair| (pair[1] - pair[0]).rem_euclid(between))
            .collect()
    }

    fn mean(values: &[f32]) -> f32 {
        values.iter().sum::<f32>() / values.len() as f32
    }

    #[test]
    fn competition_gives_the_classic_patterns() {
        // One limb a node spirals by the golden angle, pairs cross, and whorls of three alternate.
        // A turn either way reads the same. With nodes unevenly spaced, a spiral's turns swing
        // about the golden angle, and are judged by their mean.
        let spiral: Vec<f32> = node_turns(1.0)
            .iter()
            .map(|turn| turn.min(360.0 - turn))
            .collect();
        assert!((mean(&spiral) - 137.5).abs() < 8.0, "{spiral:?}");
        // Pairs cross at 90° and whorls of three alternate at 60°, the most a turn can be from
        // the last node's limbs, so they are judged by how far each turn is from it.
        for (per_node, expected) in [(2.0, 90.0), (3.0, 60.0)] {
            let off: Vec<f32> = node_turns(per_node)
                .iter()
                .map(|turn| (turn - expected).abs())
                .collect();
            assert!(mean(&off) < 12.0, "{per_node}: {off:?}");
        }
    }

    /// The vertical part of each second-level limb's direction, on level first-level limbs, with
    /// the second level facing `facing`.
    fn rises(facing: Facing) -> Vec<f32> {
        let mut species = straight();
        species.limb_levels = 2;
        species.limbs[0].angle = Fuzzy::new(FRAC_PI_2, 0.0, Distribution::Uniform);
        species.limbs[1].facing = facing;
        species.limbs[1].per_node = 1.0;
        let limbs = limbs(&species);
        let rises: Vec<f32> = limbs
            .iter()
            .filter(|limb| limb.level == 1)
            .map(|limb| limb.direction().y)
            .collect();
        assert!(rises.len() > 20, "{}", rises.len());
        rises
    }

    #[test]
    fn limbs_on_a_level_parent_face_as_asked() {
        let level = |rises: &[f32]| mean(&rises.iter().map(|y| y.abs()).collect::<Vec<_>>());
        let around = rises(Facing::Around);
        assert!(level(&rises(Facing::Sides)) < 0.5 * level(&around));
        assert!(mean(&rises(Facing::Up)) > 0.4);
        assert!(mean(&rises(Facing::Down)) < -0.4);
        assert!(mean(&around).abs() < 0.2);
    }

    #[test]
    fn skew_turns_each_limb_on_its_own() {
        // How far each pair's limbs are from opposite, about the upright stem, in degrees.
        let off_opposite = |skew: f32| -> Vec<f32> {
            let mut species = straight();
            species.limbs[0].per_node = 2.0;
            species.limbs[0].skew = skew.to_radians();
            let azimuth = |limb: &Limb| limb.direction().z.atan2(limb.direction().x).to_degrees();
            side_limbs(&species)
                .chunks(2)
                .map(|pair| {
                    ((azimuth(&pair[1]) - azimuth(&pair[0])).rem_euclid(360.0) - 180.0).abs()
                })
                .collect()
        };
        assert!(off_opposite(0.0).iter().all(|off| *off < 1e-3));
        // Each limb turns up to 30°, so a pair is at most 60° from opposite.
        let skewed = off_opposite(30.0);
        assert!(skewed.iter().all(|off| *off <= 60.0 + 1e-3), "{skewed:?}");
        assert!(mean(&skewed) > 10.0, "{skewed:?}");
    }

    #[test]
    fn splayed_stems_send_their_limbs_outward() {
        let mut species = regular();
        species.trunk.stems = 3;
        species.trunk.splay = 30.0_f32.to_radians();
        species.trunk.lean = 0.0;
        // How far each limb points out from the axis, horizontally.
        let outward: Vec<f32> = limbs(&species)
            .iter()
            .map(|limb| {
                let out = Vec3::new(limb.origin().x, 0.0, limb.origin().z).normalize();
                limb.direction().dot(out)
            })
            .collect();
        assert!(mean(&outward) > 0.2, "{outward:?}");
    }

    #[test]
    fn nodes_lie_near_their_even_places_inside_the_crown() {
        let mut species = straight();
        // Five nodes along the stem's two crown radii inside the crown, each within 30% of the
        // 0.8 m gap of 1.65, 2.45, 3.25, 4.05 and 4.85 m.
        species.limbs[0].density = 2.5;
        let heights: Vec<f32> = side_limbs(&species)
            .iter()
            .map(|limb| limb.origin().y)
            .collect();
        assert_eq!(heights.len(), 5);
        for (height, expected) in heights.iter().zip([1.65, 2.45, 3.25, 4.05, 4.85]) {
            assert!((height - expected).abs() <= 0.24 + 1e-3, "{heights:?}");
        }
        assert!(heights.iter().any(|height| {
            [1.65, 2.45, 3.25, 4.05, 4.85]
                .iter()
                .all(|expected| (height - expected).abs() > 1e-3)
        }));
    }

    #[test]
    fn nodes_spread_from_their_start_to_the_tip() {
        let mut species = straight();
        species.limbs[0].density = 2.5;
        species.limbs[0].start = 0.5;
        // The stem inside the crown is 1.25 to 5.25 m, so nodes start at 3.25 m: five in its
        // upper 2 m, each within 30% of the 0.4 m gap of 3.45, 3.85, 4.25, 4.65 and 5.05 m.
        let heights: Vec<f32> = side_limbs(&species)
            .iter()
            .map(|limb| limb.origin().y)
            .collect();
        assert_eq!(heights.len(), 5);
        for (height, expected) in heights.iter().zip([3.45, 3.85, 4.25, 4.65, 5.05]) {
            assert!((height - expected).abs() <= 0.12 + 1e-3, "{heights:?}");
        }
    }

    #[test]
    fn node_places_keep_their_ends_and_order() {
        for start in [0.0, 0.3, 0.9] {
            // Pushed as far as jitter goes, the first node down and the last up.
            let first = node_place(0, 4, start, 0.0);
            let last = node_place(3, 4, start, 1.0);
            assert!(first > start && last < 1.0, "{start}: {first} {last}");
            // Neighbours pushed toward each other stay in order.
            for node in 0..3 {
                assert!(node_place(node, 4, start, 1.0) < node_place(node + 1, 4, start, 0.0));
            }
        }
    }

    #[test]
    fn limbs_share_their_parent_section() {
        let mut species = straight();
        species.limbs[0].density = 1.0;
        species.limbs[0].radius = 0.6;
        species.bark.pipe_exponent = 2.0;
        let trunk = species.trunk.radius;
        let (stems, limbs) = grow(&species);
        let top = stems[0].radii.1;
        // Two, and the one carrying the trunk on.
        assert_eq!(limbs.len(), 3);
        // Each limb is 0.6 of the trunk past it, the two sharing the trunk's section before it:
        // past each limb the trunk is 1 / √(1 + 0.6²) of what it was.
        let narrowing = (1.0_f32 + 0.36).sqrt().recip();
        let past_first = trunk * narrowing;
        let left = past_first * narrowing;
        assert!((limbs[0].radii.0 - 0.6 * past_first).abs() < 1e-6);
        assert!((limbs[1].radii.0 - 0.6 * left).abs() < 1e-6);
        // The trunk's top is what is left past both, unless the taper narrows it further.
        let tapered = trunk * (1.0 - species.trunk.taper);
        assert!(
            (top - left.min(tapered)).abs() < 1e-6,
            "{top} {left} {tapered}"
        );
        // A higher exponent narrows the trunk less.
        species.bark.pipe_exponent = 3.0;
        species.trunk.taper = 0.0;
        let higher = grow(&species).0[0].radii.1;
        assert!(higher > left, "{higher} {left}");
    }

    #[test]
    fn an_even_fork_never_uses_up_its_parent() {
        // At 100% each limb is as thick as the trunk past it, √½ of the trunk before it.
        let mut species = straight();
        species.limbs[0].density = 1.0;
        species.limbs[0].radius = 1.0;
        species.trunk.taper = 0.0;
        let trunk = species.trunk.radius;
        let (stems, limbs) = grow(&species);
        let half = std::f32::consts::FRAC_1_SQRT_2;
        assert!((limbs[0].radii.0 - trunk * half).abs() < 1e-6);
        assert!((limbs[1].radii.0 - trunk * half * half).abs() < 1e-6);
        assert!((stems[0].radii.1 - trunk * half * half).abs() < 1e-6);
    }

    #[test]
    fn a_stem_keeps_its_radius_to_its_first_limb() {
        // The upright trunk's limbs start inside the crown, 1.25 m up: no wood leaves it below, so
        // it is as thick there as at the ground, and narrows from there.
        let species = straight();
        let (stems, limbs) = grow(&species);
        let trunk = &stems[0];
        let first = limbs[0].origin();
        assert_eq!(trunk.rings.len(), 3);
        assert!((trunk.rings[1].center - first).length() < 1e-5);
        assert_eq!(trunk.narrows_from, trunk.rings[1].at);
        let mut geometry = Geometry::default();
        let sides = 6;
        tube(
            &mut geometry,
            &trunk.rings,
            trunk.radii,
            trunk.narrows_from,
            sides,
            false,
        );
        for vertex in &geometry.positions[sides as usize..2 * sides as usize] {
            assert!(((*vertex - first).length() - trunk.radii.0).abs() < 1e-5);
        }
    }

    #[test]
    fn a_limb_carries_its_parent_on_from_its_last_node() {
        let mut species = straight();
        species.limb_levels = 2;
        let (stems, limbs) = grow(&species);
        let (sides, carriers) = sides_and_carriers(&species);
        // The trunk ends where its last limb leaves it.
        let trunk = &stems[0];
        let tip = trunk.rings[trunk.rings.len() - 1];
        let last = sides.iter().rfind(|limb| limb.level == 0).unwrap();
        assert!(trunk.carried_on);
        assert_eq!(tip.center, last.origin());
        // One limb carries on each parent bearing limbs: the trunk, and each first-level limb
        // that does.
        let bearing = limbs.iter().filter(|limb| limb.children).count();
        assert_eq!(carriers.len(), 1 + bearing);
        // The trunk's starts on its tip, along it, at its top radius.
        let carrier = &carriers[0];
        assert_eq!((carrier.level, carrier.rings[0].side), (0, tip.side));
        assert_eq!(carrier.radii.0, trunk.radii.1);
    }

    #[test]
    fn a_tube_carried_on_is_left_open() {
        let mut species = straight();
        species.limbs[0].density = 1.0;
        // The trunk's two segments of six quads, to its first limb and on to its top, open there,
        // and three straight limbs of four quads and a cap.
        assert_eq!(
            plant(&species).bark.triangles(),
            2 * 6 * 2 + 3 * (4 * 2 + 4)
        );
    }

    #[test]
    fn thin_limbs_are_drawn_as_wide_as_the_detail_threshold() {
        let mut species = Species {
            limb_levels: 3,
            ..regular()
        };
        species.trunk.lean = 0.0;
        let thick = Species {
            bark: Bark {
                detail: 0.1,
                ..species.bark.clone()
            },
            ..species.clone()
        };
        let (thin, thick) = (plant(&species), plant(&thick));
        // Every stem straight, so no bend is kept for the threshold: the same limbs and leaves,
        // drawn thicker.
        assert_eq!(thick.level_limbs, thin.level_limbs);
        assert_ne!(thick.bark.positions, thin.bark.positions);
        assert_eq!(thick.leaves.positions, thin.leaves.positions);
        // The thinnest tube is drawn 10 cm wide: some ring vertex lies 5 cm off its tip.
        let tip = |limb: &Limb| limb.rings[limb.rings.len() - 1].center;
        let thinnest = limbs(&species)
            .into_iter()
            .min_by(|a, b| a.radii.1.total_cmp(&b.radii.1))
            .unwrap();
        assert!(thinnest.radii.1 < 0.05);
        assert!(
            thick
                .bark
                .positions
                .iter()
                .any(|p| ((*p - tip(&thinnest)).length() - 0.05).abs() < 1e-4)
        );
    }

    #[test]
    fn nodes_are_counted_in_crown_radii() {
        let mut species = straight();
        // The stem inside the crown is 4 m, two crown radii, then four.
        species.limbs[0].density = 1.0;
        assert_eq!(side_limbs(&species).len(), 2);
        species.crown.radius = 1.0;
        assert_eq!(side_limbs(&species).len(), 4);
    }

    #[test]
    fn fractional_counts_average_out_over_seeds() {
        // Limbs on the trunk for each of many seeds, with `density` and `per_node` as given.
        let counts = |density: f32, per_node: f32| -> Vec<usize> {
            (0..400)
                .map(|seed| {
                    let mut species = straight();
                    species.seed = Seed(seed);
                    species.limbs[0].density = density;
                    species.limbs[0].per_node = per_node;
                    side_limbs(&species).len()
                })
                .collect()
        };
        let mean = |counts: &[usize]| counts.iter().sum::<usize>() as f32 / counts.len() as f32;
        // 2.5 nodes on the stem's two crown radii: two or three.
        let nodes = counts(1.25, 1.0);
        assert!(
            nodes.iter().all(|count| (2..=3).contains(count)),
            "{nodes:?}"
        );
        assert!((mean(&nodes) - 2.5).abs() < 0.1, "{}", mean(&nodes));
        // Eight nodes of one or two limbs.
        let limbs = counts(4.0, 1.5);
        assert!(
            limbs.iter().all(|count| (8..=16).contains(count)),
            "{limbs:?}"
        );
        assert!((mean(&limbs) - 12.0).abs() < 0.3, "{}", mean(&limbs));
    }

    #[test]
    fn winding_faces_the_normals() {
        for lean in [0.0, 45.0, 90.0] {
            let species = Species {
                trunk: trunk_with(5, lean),
                limb_levels: 3,
                ..Species::default()
            };
            let grown = plant(&species);
            assert!(grown.leaves.triangles() > 0);
            for geometry in [&grown.bark, &grown.leaves] {
                for triangle in geometry.indices.chunks(3) {
                    let [a, b, c] = [0, 1, 2].map(|i| triangle[i] as usize);
                    let p = &geometry.positions;
                    let facing = (p[b] - p[a]).cross(p[c] - p[a]);
                    let n = &geometry.normals;
                    assert!(
                        facing.dot(n[a] + n[b] + n[c]) > 0.0,
                        "lean {lean}: {triangle:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn depth_is_measured_from_the_side_and_top() {
        // A column 2.5 m wide from 1.2 m to 6 m.
        let envelope = Envelope {
            profile: Profile::Column,
            base: 1.2,
            top: 6.0,
            ..Envelope::default()
        };
        let depth = |x: f32, y: f32| depth_in_envelope(&envelope, Vec3::new(x, y, 0.0));
        assert!((depth(2.0, 3.0).unwrap() - 0.5).abs() < 1e-4);
        assert!((depth(0.0, 5.5).unwrap() - 0.5).abs() < 1e-4);
        // Just above the base, as deep as the side makes it.
        assert!((depth(0.0, 1.3).unwrap() - 2.5).abs() < 1e-4);
        assert_eq!(depth(3.0, 3.0), None);
        assert_eq!(depth(0.0, 1.0), None);
    }

    /// Where the default plant's cards meet their twigs, with `limb_levels` levels and the foliage
    /// as given: the middle of each card's base.
    fn card_bases(limb_levels: i32, foliage: Foliage) -> Vec<Vec3> {
        let species = Species {
            limb_levels,
            foliage,
            ..Species::default()
        };
        plant(&species)
            .leaves
            .positions
            .chunks(4)
            .map(|corners| (corners[0] + corners[1]) * 0.5)
            .collect()
    }

    #[test]
    fn clusters_grow_in_the_shell() {
        let foliage = Foliage::default();
        let envelope = Envelope::default();
        let bases = card_bases(1, foliage.clone());
        // Around 40 for the default one-level tree, not hundreds.
        assert!((20..150).contains(&bases.len()), "{}", bases.len());
        // One at each limb's tip, wherever it ends; the rest in the shell.
        let tips: Vec<Vec3> = limbs(&Species::default())
            .iter()
            .map(|limb| limb.rings[limb.rings.len() - 1].center)
            .collect();
        for base in &bases {
            if tips.iter().any(|tip| (*tip - *base).length() < 1e-4) {
                continue;
            }
            let depth = depth_in_envelope(&envelope, *base).expect("inside the envelope");
            assert!(depth < foliage.shell * envelope.radius + 1e-3, "{depth}");
        }
        assert!(
            tips.iter()
                .all(|tip| bases.iter().any(|base| (*tip - *base).length() < 1e-4))
        );
    }

    #[test]
    fn clusters_follow_the_density() {
        let with = |density: f32| {
            card_bases(
                2,
                Foliage {
                    density,
                    ..Foliage::default()
                },
            )
            .len()
        };
        assert_eq!(with(0.0), 0);
        // Beyond the one at each tip, which any density grows.
        let tips = with(1e-6);
        let (some, more) = (with(5.0) - tips, with(20.0) - tips);
        assert!(some > 0 && more > 3 * some, "{tips} {some} {more}");
    }

    #[test]
    fn cards_lie_along_their_twigs_facing_up() {
        let foliage = Foliage::default();
        let side = foliage.size * Envelope::default().radius;
        let species = Species {
            foliage,
            limb_levels: 2,
            ..regular()
        };
        let limbs = limbs(&species);
        let twigs: Vec<&Limb> = limbs.iter().filter(|limb| !limb.children).collect();
        let leaves = plant(&species).leaves;
        assert!(!leaves.positions.is_empty());
        assert_eq!(leaves.uvs.len(), leaves.positions.len());
        for (corners, normals) in leaves.positions.chunks(4).zip(leaves.normals.chunks(4)) {
            let base = (corners[0] + corners[1]) * 0.5;
            let axis = (corners[3] - corners[0]) / side;
            let across = (corners[1] - corners[0]) / side;
            let normal = normals[0];
            assert!((across.length() - 1.0).abs() < 1e-4 && (axis.length() - 1.0).abs() < 1e-4);
            // Level across, facing as near up as the axis allows.
            assert!(across.y.abs() < 1e-4, "{across}");
            assert!(normal.dot(axis).abs() < 1e-4 && normal.y >= 0.0, "{normal}");
            assert!((normal - Vec3::Y.reject_from(axis).normalize()).length() < 1e-3);
            // The base on a twig, which the axis follows: straight twigs, so their direction.
            let twig = twigs
                .iter()
                .find(|twig| {
                    let from = base - twig.origin();
                    from.cross(twig.direction()).length() < 1e-3
                })
                .expect("a twig under the card");
            assert!(axis.angle_between(twig.direction()) < 1e-3);
        }
    }

    #[test]
    fn leaves_grow_on_every_limb_bearing_none() {
        // A second level with no nodes leaves the first bearing the leaves, as if it were the last.
        let mut species = Species {
            limb_levels: 2,
            ..Species::default()
        };
        species.limbs[1].density = 0.0;
        let leaves = |species: &Species| plant(species).leaves.positions;
        let bare_second = leaves(&species);
        species.limb_levels = 1;
        assert!(!bare_second.is_empty());
        assert_eq!(bare_second, leaves(&species));
    }
}
