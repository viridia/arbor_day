use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::math::{Vec3, Vec3Swizzles};

use super::{
    envelope::*, foliage::*, hint::*, mesh::*, placement::*, skeleton::*, trace::*, wander::*,
};
use crate::{
    fuzzy::{Distribution, Fuzzy},
    random::Seed,
    species::{
        Bias, Facing, Flare, Lean, Ornaments, Profile, Sag, Species, TipLeaves, Trunk, Upright, Wander,
    },
    toggle::{OperatorState, Toggle},
};

/// A lone 5 m stem's rings, standing straight up and leaning toward +X, with no wander, hinted
/// to `tolerance`.
fn lone_stem(lean_degrees: f32, tolerance: f32) -> Vec<Ring> {
    hint_rings(
        &stem_rings(5.0, 0.0, lean_degrees.to_radians(), 0.0),
        tolerance,
    )
}

/// `lone_stem`'s mesh, hinted to 2 cm, narrowing from 20 cm to 10 cm.
fn stem_mesh(sides: u32, lean_degrees: f32) -> Geometry {
    let mut rings = lone_stem(lean_degrees, 0.02);
    taper_rings(&mut rings, 0.2, 0.5);
    let mut geometry = Geometry::default();
    tube(&mut geometry, &rings, &vec![sides; rings.len()], 0.0);
    geometry
}

/// `species`'s trunk forking `at` its length, a share, into `count` stems, each `splay` degrees
/// from its direction: at 0, a clump from the ground.
fn forked_trunk(mut species: Species, count: i32, splay: f32, at: f32) -> Species {
    let fork = &mut species.trunk.fork;
    fork.state = OperatorState::On;
    fork.settings.chance = 1.0;
    fork.settings.at = Fuzzy::new(at, 0.0, Distribution::Bell);
    fork.settings.copies = count;
    fork.settings.angle = Fuzzy::new((2.0 * splay).to_radians(), 0.0, Distribution::Bell);
    species
}

fn trunk_with(sides: i32, lean_degrees: f32) -> Trunk {
    Trunk {
        sides,
        lean: Toggle::on(Lean::degrees(lean_degrees)),
        wander: Toggle::on(Wander::degrees(0.0)),
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
    // Traced, a stem keeps a ring at every step; grown, it keeps those hinting does.
    let mut species = Species::default();
    species.trunk.wander.settings.angle = 45.0_f32.to_radians();
    let traced = &stems(&species)[0];
    assert!(traced.rings.len() as f32 > species.trunk.height / TRACE_STEP);
    let grown = &grow(&species).axes[0];
    assert!(grown.rings.len() < traced.rings.len() / 4);
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
    species.trunk.height = 3.0;
    let tip = stems(&species)[0].rings.last().unwrap().center;
    assert!((tip - Vec3::Y * 3.0).length() < 1e-5, "{tip}");
}

#[test]
fn the_crown_moves_without_the_trunk() {
    let mut species = regular();
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
    // Forked at the ground, at partway up past level, and partway up from a leaning trunk.
    for (splay, at) in [(50.0, 0.0), (120.0, 0.5)] {
        let mut species = forked_trunk(regular(), 2, splay, at);
        species.trunk.lean = Toggle::on(Lean::degrees(90.0));
        for stem in stems(&species) {
            let tip = stem.rings.last().unwrap().tangent;
            assert!(tip.y > -1e-5, "{splay} {at}: {tip}");
        }
    }
}

#[test]
fn slant_bends_a_stem_without_lengthening_it() {
    let length = |splay: f32, lean: f32| {
        let mut species = forked_trunk(regular(), 2, splay, 0.0);
        species.trunk.lean = Toggle::on(Lean::degrees(lean));
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
fn a_trunk_forked_at_the_ground_is_a_clump() {
    let species = forked_trunk(Species::default(), 3, 30.0, 0.0);
    let stems = stems(&species);
    assert_eq!(stems.len(), 3);
    assert!(
        stems
            .iter()
            .all(|stem| stem.kind == AxisKind::Stem && stem.parent.is_none())
    );
    assert!(
        stems
            .iter()
            .all(|stem| stem.radii().0 == species.trunk.radius)
    );
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
fn a_trunk_forked_partway_splits_into_stems() {
    let mut species = forked_trunk(regular(), 2, 20.0, 0.4);
    species.trunk.lean = Toggle::on(Lean::degrees(30.0));
    let stems = stems(&species);
    let (trunk, copies) = (&stems[0], &stems[1..]);
    assert_eq!(copies.len(), 2);
    assert_eq!(trunk.scions, [1, 2]);
    // The trunk ends at 40% of its height, where the copies start at its radius.
    let height = species.trunk.height;
    let tip = trunk.rings[trunk.rings.len() - 1];
    assert!((trunk.length() - 0.4 * height).abs() < 1e-4);
    for copy in copies {
        assert_eq!(
            (copy.kind, copy.level, copy.parent),
            (AxisKind::ForkCopy, 0, Some((0, tip.at)))
        );
        assert!((copy.origin() - tip.center).length() < 1e-5);
        assert_eq!(copy.radii().0, tip.radius);
        // 20° from the trunk, and the rest of the height long.
        let angle = copy.direction().angle_between(tip.tangent).to_degrees();
        assert!((angle - 20.0).abs() < 1e-2, "{angle}");
        assert!((tip.at + copy.length() - height).abs() < 1e-4);
        // Leaning on, away from the middle: further from up at the tip than at the fork.
        let from_up = |direction: Vec3| direction.angle_between(Vec3::Y);
        assert!(from_up(copy.rings[copy.rings.len() - 1].tangent) > from_up(copy.direction()));
    }
    // Grown, the limbs grow on the copies.
    let axes = grow(&species).axes;
    assert!(
        axes.iter()
            .any(|limb| limb.parent.is_some_and(|(parent, _)| parent == 1))
    );
}

#[test]
fn a_limb_forked_at_its_base_is_several_limbs() {
    let species = forking(3, 0.0);
    let mut at_base = species.clone();
    at_base.limbs.levels[0].fork.settings.at.center = 0.0;
    let axes = grow(&at_base).axes;
    // No forked limb and no copy: each forked limb is up to three from where it left, a copy too
    // short to trace being left out.
    assert!(forks(&axes).is_empty());
    assert!(axes.iter().all(|axis| axis.kind != AxisKind::ForkCopy));
    let forked = grow(&species).axes;
    let limbs = (forked.iter()).filter(|axis| axis.level == 1 && axis.kind != AxisKind::ForkCopy);
    let (limbs, after) = (limbs.count(), axes.len() - 1);
    assert!(
        (2 * limbs + 1..=3 * limbs).contains(&after),
        "{limbs} {after}"
    );
    // Each listed once among its parent's scions.
    let mut scions: Vec<usize> = axes.iter().flat_map(|axis| axis.scions.clone()).collect();
    scions.sort();
    assert_eq!(scions, (1..axes.len()).collect::<Vec<_>>());
}

#[test]
fn the_seed_turns_the_lean() {
    let tip = |seed| {
        let mut species = Species {
            seed: Seed(seed),
            ..regular()
        };
        species.trunk.lean = Toggle::on(Lean::default());
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
fn helper_skips_spokes_and_rings_of_no_width() {
    let segments = |profile| {
        let envelope = Envelope {
            profile,
            ..Envelope::default()
        };
        envelope_helper(&envelope).len() / 2
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
    species.trunk.wander.settings.angle = 0.0;
    for level in &mut species.limbs.levels {
        level.sag.settings.amount = 0.0;
        level.wander.settings.angle = 0.0;
        level.skew = 0.0;
        for fuzzy in [
            &mut level.angle,
            &mut level.reach,
            &mut level.fork.settings.at,
            &mut level.fork.settings.angle,
        ] {
            fuzzy.spread = 0.0;
        }
    }
    species
}

/// `species` with its first `count` limb levels growing.
fn shown(count: i32, mut species: Species) -> Species {
    species.limbs.shown = count;
    species
}

/// `regular`, with an upright stem in a column crown: the stem is inside the crown from the
/// crown base at 1.25 m to its tip at 5.25 m, two crown radii of 2 m.
fn straight() -> Species {
    let mut species = regular();
    species.trunk.height = 5.25;
    species.crown.profile = Profile::Column;
    species.crown.base = 1.25 / 5.25;
    species.crown.top = 6.25 / 5.25;
    species.crown.radius = 2.0;
    species
}

/// The limbs: every axis but the stems.
fn limbs(species: &Species) -> Vec<Axis> {
    let axes = grow(species).axes;
    axes.into_iter().filter(|axis| axis.level > 0).collect()
}

/// The limbs of `kind`.
fn limbs_of(species: &Species, kind: AxisKind) -> Vec<Axis> {
    let limbs = limbs(species);
    limbs.into_iter().filter(|limb| limb.kind == kind).collect()
}

/// The limbs leaving their parents' sides.
fn side_limbs(species: &Species) -> Vec<Axis> {
    limbs_of(species, AxisKind::Lateral)
}

/// A stem's inside part, as heights.
fn inside_heights(species: &Species) -> (f32, f32) {
    let stem = &stems(species)[0];
    let (start, end) = stem
        .inside_part(&Envelope::of(species))
        .expect("a stem inside the crown");
    (stem.frame(start).point.y, stem.frame(end).point.y)
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
    assert_eq!(limbs.len(), 16);
    for limb in &limbs {
        assert_eq!(limb.rings.len(), 2, "a straight limb is one segment");
        let tip = limb.origin() + limb.direction() * limb.length();
        let rest = distance_to_envelope(&Envelope::of(&species), tip, limb.direction());
        let whole = limb.length() + rest;
        assert!((limb.length() - species.limbs.levels[0].reach.center * whole).abs() < 1e-3);
    }
}

#[test]
fn bias_size_changes_laterals_along_their_parent() {
    // Half way to the envelope, so 150% of it stays inside.
    let mut species = straight();
    species.limbs.levels[0].reach.center = 0.5;
    let reach = species.limbs.levels[0].reach.center;
    species.limbs.levels[0].bias = Toggle::on(Bias {
        size: 0.5,
        aim: 0.0,
        leaves: 0.0,
    });
    let laterals = side_limbs(&species);
    let envelope = Envelope::of(&species);
    // One a node, from the first at 150% of the reach to the last at 50%, the middle as the
    // level's.
    assert_eq!(laterals.len(), 16);
    for (node, limb) in laterals.iter().enumerate() {
        let place = 2.0 * node as f32 / 15.0 - 1.0;
        let tip = limb.origin() + limb.direction() * limb.length();
        let whole = limb.length() + distance_to_envelope(&envelope, tip, limb.direction());
        let share = 1.0 - 0.5 * place;
        assert!(
            (limb.length() - share * reach * whole).abs() < 1e-3,
            "{node}"
        );
    }
}

#[test]
fn bias_leaves_scale_cards_along_their_parent() {
    let mut species = shown(2, straight());
    species.limbs.levels[0].bias = Toggle::on(Bias {
        size: 0.6,
        aim: 0.0,
        leaves: 0.5,
    });
    let skeleton = grow(&species);
    let axes = &skeleton.axes;
    let full = card_size(&species);
    // The trunk's laterals by node, one a node, from 160% of the reach at the first to 40% at the
    // last: their leaves from 100% down by half of the way to 40/160.
    let mut laterals: Vec<usize> = (0..axes.len())
        .filter(|&index| axes[index].level == 1 && axes[index].parent.is_some_and(|p| p.0 == 0))
        .collect();
    laterals.sort_by(|&a, &b| axes[a].parent.unwrap().1.total_cmp(&axes[b].parent.unwrap().1));
    assert_eq!(laterals.len(), 16);
    let scale_of = |node: usize| {
        let reach = 1.0 - 0.6 * (2.0 * node as f32 / 15.0 - 1.0);
        1.0 - 0.5 * (1.0 - reach / 1.6)
    };
    let cards_of = |axis: usize| skeleton.cards.iter().filter(move |card| card.axis == axis);
    for (node, &lateral) in laterals.iter().enumerate() {
        let scale = scale_of(node);
        assert!((axes[lateral].leaf_scale - scale).abs() < 1e-4, "{node}");
        // The second level, its Bias off, keeps its parent's.
        let expected = full * scale;
        let twigs = axes[lateral].scions.iter().copied();
        for card in cards_of(lateral).chain(twigs.flat_map(cards_of)) {
            assert!((card.size - expected).abs() < 1e-4, "{node}");
        }
    }
    // The trunk's tip as its last node's.
    let tip = full * scale_of(15);
    assert!(cards_of(0).all(|card| (card.size - tip).abs() < 1e-4));
    assert!(cards_of(0).count() > 0);
    // Halved at the least.
    species.limbs.levels[0].bias.settings.leaves = 1.0;
    species.limbs.levels[0].bias.settings.size = 0.8;
    let skeleton = grow(&species);
    let least = full * LEAST_LEAF_SCALE;
    assert!(skeleton.cards.iter().all(|card| card.size >= least - 1e-4));
    assert!(skeleton.cards.iter().any(|card| (card.size - least).abs() < 1e-4));
    // Longest at the last node, so full there.
    let level = &mut species.limbs.levels[0];
    level.bias.settings.size = -0.6;
    assert_eq!(bias_leaves(level, 15, 16), 1.0);
    assert!((bias_leaves(level, 0, 16) - 0.4 / 1.6).abs() < 1e-6);
}

#[test]
fn aim_turns_laterals_toward_the_outline() {
    // The column's outline runs up its wall from the crown base at 1.25 m, then across its top at
    // 6.25 m to the axis.
    let mut species = straight();
    let drawn = side_limbs(&species);
    species.limbs.levels[0].bias = Toggle::on(Bias {
        size: 0.0,
        aim: 1.0,
        leaves: 0.0,
    });
    let aimed = side_limbs(&species);
    assert_eq!(aimed.len(), drawn.len());
    let heading = |limb: &Axis| limb.direction().xz().normalize();
    let from_up = |limb: &Axis| limb.direction().angle_between(Vec3::Y);
    // The last points straight up, with no heading.
    for (aimed, drawn) in aimed.iter().zip(&drawn).rev().skip(1) {
        let (a, b) = (heading(aimed), heading(drawn));
        assert!(a.distance(b) < 1e-3, "aim keeps each limb's heading");
    }
    // The first toward the wall at the crown base, below its node; each steeper than the one
    // before; the last toward the top on the axis.
    assert!(aimed[0].direction().y <= 0.0);
    for pair in aimed.windows(2) {
        assert!(from_up(&pair[1]) < from_up(&pair[0]));
    }
    let last = aimed.last().unwrap();
    let toward_top = (Vec3::Y * 6.25 - last.origin()).normalize();
    assert!(last.direction().angle_between(toward_top) < 1e-3);
}

#[test]
fn limbs_leave_the_trunk_at_their_angle() {
    // Upright, the trunk is one straight segment, base to tip.
    let species = regular();
    let tip = stems(&species)[0].rings.last().unwrap().center;
    for limb in side_limbs(&species) {
        let angle = limb.direction().angle_between(tip);
        assert!(
            (angle - species.limbs.levels[0].angle.center).abs() < 1e-5,
            "{limb:?}"
        );
    }
}

#[test]
fn varied_values_stay_within_their_spread_and_limits() {
    let mut species = regular();
    species.limbs.levels[0].angle = Fuzzy::new(
        60.0_f32.to_radians(),
        10.0_f32.to_radians(),
        Distribution::Uniform,
    );
    // Reaching past the envelope is clamped to reaching it.
    species.limbs.levels[0].reach = Fuzzy::new(0.95, 0.2, Distribution::Uniform);
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
        let whole = distance_to_envelope(&Envelope::of(&species), limb.origin(), limb.direction());
        assert!(limb.length() <= whole + 1e-3);
    }
}

#[test]
fn deeper_levels_grow_on_the_level_before() {
    let mut species = regular();
    let first = limbs(&species);
    species.limbs.shown = 2;
    let both = limbs(&species);
    // The first level is unchanged but for its tips, which its limbs narrow and cut short at
    // the last of them, and the second follows it.
    for (before, after) in first.iter().zip(&both) {
        assert_eq!(
            (before.rings[0], before.radii().0),
            (after.rings[0], after.radii().0)
        );
        assert!(after.length() <= before.length());
    }
    let second: Vec<Axis> = side_limbs(&species)
        .into_iter()
        .filter(|limb| limb.level == 2)
        .collect();
    assert!(!second.is_empty());
    for limb in &second {
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
        assert!((angle - species.limbs.levels[1].angle.center).abs() < 1e-3);
    }
}

/// Checks each of `species`' first-level limbs: its angle from up at the tip is `expected` of its
/// angle at the base and its length in crown radii, and its compass heading is kept.
fn check_tips(species: &Species, expected: impl Fn(f32, f32) -> f32) {
    let crown = species.crown.radius;
    for limb in limbs(species).iter().filter(|limb| limb.level == 1) {
        let tip = limb.rings[limb.rings.len() - 1].tangent;
        let at_base = limb.direction().angle_between(Vec3::Y);
        let at_tip = tip.angle_between(Vec3::Y);
        let expected = expected(at_base, limb.length() / crown);
        assert!((at_tip - expected).abs() < 1e-3, "{at_tip} {expected}");
        let heading = |v: Vec3| v.z.atan2(v.x);
        assert!((heading(tip) - heading(limb.direction())).abs() < 1e-3);
    }
}

#[test]
fn limbs_sag_by_the_sine_of_their_angle() {
    // The turn per crown radius scaled by the sine of the angle from up: the tangent of the half
    // angle grows by e to the turn.
    let mut species = regular();
    species.limbs.levels[0].sag = Toggle::on(Sag { amount: 0.5 });
    check_tips(&species, |at_base, crowns| {
        2.0 * ((at_base / 2.0).tan() * (0.5 * crowns).exp()).atan()
    });
    // Exactly, whatever the step: half a turn and half again is one turn.
    let halves = sag(sag(1.0, 0.25), 0.25);
    assert!((halves - sag(1.0, 0.5)).abs() < 1e-5);
    // Neither an upright course nor a hanging one sags.
    assert_eq!(sag(0.0, 1.0), 0.0);
    assert!((sag(PI, 1.0) - PI).abs() < 1e-5);
}

#[test]
fn limbs_rise_toward_up() {
    // Half the angle from up is closed over each crown radius of length.
    let mut species = regular();
    species.limbs.levels[0].sag.state = OperatorState::Off;
    species.limbs.levels[0].rise.state = OperatorState::On;
    species.limbs.levels[0].rise.settings.amount = 0.5;
    check_tips(&species, |at_base, crowns| at_base * 0.5_f32.powf(crowns));
}

#[test]
fn default_plants_stay_within_budget() {
    // Orders of magnitude, not counts: the low hundreds for one level; two, the usual tree,
    // under seedling's trees (about 2,500); three, the costly exception, a little over them,
    // every twig drawn.
    let triangles = |limb_levels| {
        let species = shown(limb_levels, Species::default());
        plant(&species).bark.triangles()
    };
    assert!((100..1_000).contains(&triangles(1)), "{}", triangles(1));
    assert!(triangles(2) < 2_500, "{}", triangles(2));
    assert!(triangles(3) < 3_500, "{}", triangles(3));
}

#[test]
fn rings_advance_along_every_limb() {
    let species = shown(3, Species::default());
    for limb in limbs(&species) {
        for pair in limb.rings.windows(2) {
            assert!(pair[1].at > pair[0].at);
        }
    }
}

#[test]
fn limbs_kink_at_one_spacing_on_every_level() {
    // Kinks about every 30 cm on both levels, whatever their densities: segments are split while
    // longer than 45 cm, each half at least 30% of it, so no stretch is under 13.5 cm. With no
    // bend, the rings between a limb's ends can only be kinks, so a limb has no more than its
    // length over the shortest stretch.
    let mut species = level_limbs(45.0);
    species.limbs.shown = 2;
    species.limbs.levels[1].wander = Toggle::on(Wander::degrees(45.0));
    let spacing = kink_spacing(&Envelope::of(&species));
    assert!((spacing - 0.3).abs() < 1e-6);
    let shortest = spacing * SPLIT_LENGTH * (0.5 - KINK_JITTER);
    let limbs = limbs(&species);
    for limb in &limbs {
        let kinks = (limb.length() / shortest).floor() as usize;
        let rings = limb.rings.len();
        assert!(rings - 2 <= kinks, "{rings} {}", limb.length());
    }
    assert!(
        limbs
            .iter()
            .any(|limb| limb.level == 1 && limb.rings.len() >= 6)
    );
    assert!(
        limbs
            .iter()
            .any(|limb| limb.level == 2 && limb.length() < shortest)
    );
    // Unevenly: the stretches between one limb's kinks differ.
    let stretches: Vec<f32> = limbs
        .iter()
        .filter(|limb| limb.level == 1)
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
    species.limbs.levels[0].angle = Fuzzy::new(FRAC_PI_2, 0.0, Distribution::Uniform);
    species.limbs.levels[0].reach = Fuzzy::new(1.0, 0.0, Distribution::Uniform);
    species.limbs.levels[0].wander = Toggle::on(Wander::degrees(wander));
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
fn operators_not_on_grow_as_at_zero() {
    // Disabled or off, an operator grows the plant as its value that does nothing would.
    let rings = |species: &Species| -> Vec<Vec3> {
        let axes = grow(species).axes;
        axes.iter()
            .flat_map(|axis| axis.rings.iter().map(|ring| ring.center))
            .collect()
    };
    let mut off = level_limbs(45.0);
    off.limbs.levels[0].wander.state = OperatorState::Disabled;
    assert_eq!(rings(&off), rings(&level_limbs(0.0)));

    let mut sagged = Species::default();
    sagged.limbs.levels[0].sag.settings.amount = 0.8;
    let mut straight = sagged.clone();
    straight.limbs.levels[0].sag.settings.amount = 0.0;
    sagged.limbs.levels[0].sag.state = OperatorState::Off;
    assert_eq!(rings(&sagged), rings(&straight));

    let mut risen = Species::default();
    risen.limbs.levels[0].rise.settings.amount = 0.8;
    let mut level = risen.clone();
    level.limbs.levels[0].rise.state = OperatorState::On;
    level.limbs.levels[0].rise.settings.amount = 0.0;
    risen.limbs.levels[0].rise.state = OperatorState::Disabled;
    assert_eq!(rings(&risen), rings(&level));

    let mut leaning = Species::default();
    leaning.trunk.lean = Toggle::on(Lean::degrees(30.0));
    let mut upright = leaning.clone();
    upright.trunk.lean.settings.angle = 0.0;
    leaning.trunk.lean.state = OperatorState::Disabled;
    assert_eq!(rings(&leaning), rings(&upright));

    let mut biased = Species::default();
    let mut even = biased.clone();
    even.limbs.levels[0].bias = Toggle::on(Bias {
        size: 0.0,
        aim: 0.0,
        leaves: 0.0,
    });
    biased.limbs.levels[0].bias.state = OperatorState::Disabled;
    assert_eq!(rings(&biased), rings(&even));

    let mut unforked = forking(2, 0.0);
    unforked.limbs.levels[0].fork.state = OperatorState::Disabled;
    let mut never = forking(2, 0.0);
    never.limbs.levels[0].fork.settings.chance = 0.0;
    assert_eq!(rings(&unforked), rings(&never));
}

#[test]
fn wander_leaves_tips_where_their_course_ends() {
    // The trunk, and the limbs on it, end where they would with no wander.
    let tips = |species: &Species| {
        let axes = grow(species).axes;
        let stem = stems(species)[0].rings.last().unwrap().center;
        let limbs: Vec<Vec3> = (axes.iter().filter(|axis| axis.level == 1))
            .map(|limb| limb.rings.last().unwrap().center)
            .collect();
        (stem, limbs)
    };
    let mut species = level_limbs(0.0);
    let (stem, limbs) = tips(&species);
    species.trunk.wander.settings.angle = 45.0_f32.to_radians();
    assert!((tips(&species).0 - stem).length() < 1e-4);
    species.trunk.wander.settings.angle = 0.0;
    species.limbs.levels[0].wander.settings.angle = 45.0_f32.to_radians();
    let (_, wandering) = tips(&species);
    assert_eq!(wandering.len(), limbs.len());
    for (one, two) in wandering.iter().zip(&limbs) {
        assert!((one - two).length() < 1e-4, "{one} {two}");
    }
    // A stem leaves the ground along its course, before it is hinted, which may drop the corner
    // ending its straight base.
    species.trunk.wander.settings.angle = 45.0_f32.to_radians();
    let stem = &stems(&species)[0];
    let base = stem.rings[0];
    let run = stem.rings[1..]
        .iter()
        .take_while(|ring| ring.at < 0.5 * species.trunk.height);
    let straight: Vec<&Ring> = run
        .take_while(|ring| (ring.center - base.center).angle_between(base.tangent) < 1e-3)
        .collect();
    let along = straight.last().unwrap().at;
    let spacing = kink_spacing(&Envelope::of(&species));
    assert!(along >= 0.6 * spacing - TRACE_STEP, "{along} {spacing}");
    // Every turn between stretches under twice the wander.
    let mut species = shown(2, Species::default());
    species.trunk.wander = Toggle::on(Wander::degrees(45.0));
    for level in &mut species.limbs.levels {
        level.wander = Toggle::on(Wander::degrees(45.0));
    }
    let mut sharpest = 0.0_f32;
    for axis in grow(&species).axes {
        for three in axis.rings.windows(3) {
            let into = three[1].center - three[0].center;
            let out = three[2].center - three[1].center;
            sharpest = sharpest.max(into.angle_between(out).to_degrees());
        }
    }
    assert!(sharpest < 90.0 + 1.0, "{sharpest}");
}

#[test]
fn a_short_axis_wanders_up_to_its_tip_run() {
    // Three spacings long, a spacing of it the tip's straight run: the stretch between the base
    // run and the tip run is under the 1.5 spacings a split needs, but with the tip run it is
    // over them, so it kinks, never inside the tip run, its tip where its course ends.
    let (spacing, straight) = (1.0, 1.0);
    let course = stem_rings(3.0, 0.0, 0.0, 0.0);
    let tip = course[course.len() - 1].center;
    let kinked = (0..8).filter(|&seed| {
        let mut rings = course.clone();
        let rng = Seed(seed).stream("test");
        displace(
            &mut rings,
            rng,
            30.0_f32.to_radians(),
            spacing,
            straight,
            None,
        );
        let last = rings[rings.len() - 1];
        assert!((last.center - tip).length() < 1e-4);
        // Its last corner at least the tip run back from the tip.
        let mut corners = rings.windows(3).filter(|three| {
            let into = three[1].center - three[0].center;
            into.angle_between(three[2].center - three[1].center) > 1e-3
        });
        if let Some(corner) = corners.next_back() {
            assert!(
                corner[1].at <= last.at - straight + 1e-3,
                "{}",
                corner[1].at
            );
        }
        rings.iter().any(|ring| ring.center.xz().length() > 1e-3)
    });
    assert!(kinked.count() > 0);
}

#[test]
fn branch_ends_run_straight_under_their_cards() {
    // With no bend, a stretch with no kinks keeps no rings, so a branch end's last ring
    // before its tip lies before the stretch its card covers.
    let species = level_limbs(45.0);
    let straight = axial_fold(&species)
        * species.ornaments.tip_leaves.settings.size
        * Envelope::of(&species).radius;
    let limbs = limbs(&species);
    let ends: Vec<&Axis> = limbs.iter().filter(|limb| limb.scions.is_empty()).collect();
    assert!(ends.iter().any(|limb| limb.rings.len() > 2));
    for limb in ends {
        let [.., before, tip] = limb.rings[..] else {
            unreachable!()
        };
        assert!(
            before.at <= tip.at - straight + 1e-3,
            "{} {}",
            before.at,
            tip.at
        );
    }
}

#[test]
fn limbs_stay_inside_the_envelope() {
    let species = shown(3, Species::default());
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
            ..shown(2, Species::default())
        };
        plant(&species).bark.positions
    };
    assert_eq!(positions(1), positions(1));
    assert_ne!(positions(1), positions(2));
}

#[test]
fn hidden_levels_do_not_grow() {
    let mut species = Species::default();
    species.limbs.levels[2].density = 40.0;
    let one = limbs(&species).len();
    species.limbs.shown = 2;
    let two = limbs(&species);
    assert!(two.len() > one);
    assert!(two.iter().all(|limb| limb.level < 3));
    species.limbs.shown = 3;
    assert!(limbs(&species).iter().any(|limb| limb.level == 3));
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
    let species = regular();
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
    species.limbs.levels[0].density = 1.0;
    species.limbs.levels[0].per_node = 3.0;
    let limbs = side_limbs(&species);
    assert_eq!(limbs.len(), 6);
    // A node's limbs share an origin and stand a third of a turn apart.
    assert_eq!(limbs[0].origin(), limbs[2].origin());
    let azimuth = |limb: &Axis| limb.direction().z.atan2(limb.direction().x);
    let apart = (azimuth(&limbs[1]) - azimuth(&limbs[0])).rem_euclid(TAU);
    assert!((apart - TAU / 3.0).abs() < 1e-5);
}

/// The turns between successive nodes on `straight`'s upright stem, in degrees, modulo the
/// angle between a node's limbs, since they are alike. The first is included, so the pattern
/// holds from the first node.
fn node_turns(per_node: f32) -> Vec<f32> {
    let mut species = straight();
    species.limbs.levels[0].density = 8.0;
    species.limbs.levels[0].per_node = per_node;
    let limbs = side_limbs(&species);
    let per_node = per_node as usize;
    assert_eq!(limbs.len(), 16 * per_node);
    let azimuth = |limb: &Axis| limb.direction().z.atan2(limb.direction().x).to_degrees();
    let between = 360.0 / per_node as f32;
    limbs
        .chunks(per_node)
        .map(|node| azimuth(&node[0]))
        .collect::<Vec<_>>()
        .windows(2)
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
    // From the first node, rather than opposite, then near square, for several nodes.
    let first = &spiral[..4];
    assert!(first.iter().all(|turn| (turn - 137.5).abs() < 20.0), "{spiral:?}");
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
    species.limbs.shown = 2;
    species.limbs.levels[0].angle = Fuzzy::new(FRAC_PI_2, 0.0, Distribution::Uniform);
    species.limbs.levels[1].facing = facing;
    species.limbs.levels[1].per_node = 1.0;
    let limbs = limbs(&species);
    let rises: Vec<f32> = limbs
        .iter()
        .filter(|limb| limb.level == 2)
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
        species.limbs.levels[0].per_node = 2.0;
        species.limbs.levels[0].skew = skew.to_radians();
        let azimuth = |limb: &Axis| limb.direction().z.atan2(limb.direction().x).to_degrees();
        side_limbs(&species)
            .chunks(2)
            .map(|pair| ((azimuth(&pair[1]) - azimuth(&pair[0])).rem_euclid(360.0) - 180.0).abs())
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
    let species = forked_trunk(regular(), 3, 30.0, 0.0);
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
    // 0.8 m gap of 1.65, 2.45, 3.25, 4.05 and 4.85 m, ending at the tip. Pairs, opposite, so
    // balance leaves the places alone.
    species.limbs.levels[0].density = 2.5;
    species.limbs.levels[0].end = 1.0;
    species.limbs.levels[0].per_node = 2.0;
    let heights: Vec<f32> = (side_limbs(&species).iter().step_by(2))
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
fn nodes_spread_from_their_start_to_their_end() {
    let mut species = straight();
    species.limbs.levels[0].density = 2.5;
    species.limbs.levels[0].start = 0.5;
    species.limbs.levels[0].end = 0.9;
    species.limbs.levels[0].per_node = 2.0;
    // The stem inside the crown is 1.25 to 5.25 m, so nodes run from 3.25 to 4.85 m: five in
    // those 1.6 m, each within 30% of the 0.32 m gap of 3.41, 3.73, 4.05, 4.37 and 4.69 m. Pairs,
    // as above.
    let heights: Vec<f32> = (side_limbs(&species).iter().step_by(2))
        .map(|limb| limb.origin().y)
        .collect();
    assert_eq!(heights.len(), 5);
    for (height, expected) in heights.iter().zip([3.41, 3.73, 4.05, 4.37, 4.69]) {
        assert!((height - expected).abs() <= 0.096 + 1e-3, "{heights:?}");
    }
}

#[test]
fn node_places_keep_their_ends_and_order() {
    for (start, end) in [(0.0, 1.0), (0.3, 0.85), (0.9, 1.0)] {
        // Pushed as far as jitter goes, the first node down and the last up.
        let first = node_place(0, 4, start, end, 0.0);
        let last = node_place(3, 4, start, end, 1.0);
        assert!(first > start && last < end, "{start}: {first} {last}");
        // Neighbours pushed toward each other stay in order.
        for node in 0..3 {
            let (this, next) = (node_place(node, 4, start, end, 1.0), node);
            assert!(this < node_place(next + 1, 4, start, end, 0.0));
        }
    }
    // An end below the start puts every node at the start.
    assert_eq!(node_place(2, 4, 0.5, 0.2, 0.7), 0.5);
}

#[test]
fn a_lone_limb_low_on_the_parent_is_followed_sooner() {
    let places: Vec<f32> = (0..10).map(|node| (node as f32 + 0.5) / 10.0).collect();
    // One limb a node, spiralling by the golden angle.
    let spiral: Vec<Vec<f32>> = (0..10)
        .map(|node| vec![(node as f32 * 137.5).to_radians()])
        .collect();
    let balanced = balanced_places(&places, &spiral);
    let gaps: Vec<f32> = balanced.windows(2).map(|pair| pair[1] - pair[0]).collect();
    assert_eq!(balanced[0], places[0]);
    assert!((balanced[9] - places[9]).abs() < 1e-5);
    assert!(gaps.iter().all(|gap| *gap > 0.0), "{gaps:?}");
    assert!(gaps[0] < 0.6 * mean(&gaps), "{gaps:?}");
    // Once the spiral has turned about, its gaps are alike.
    let settled = &gaps[3..];
    assert!(settled.iter().all(|gap| (gap - mean(settled)).abs() < 0.1 * mean(settled)));
    // Pairs, opposite, are balanced from the first.
    let pairs: Vec<Vec<f32>> = (0..10)
        .map(|node| {
            let turn = (node as f32 * 90.0).to_radians();
            vec![turn, turn + PI]
        })
        .collect();
    let paired = balanced_places(&places, &pairs);
    assert!(paired.iter().zip(&places).all(|(a, b)| (a - b).abs() < 1e-5));
}

#[test]
fn limbs_take_their_share_of_the_parent_where_they_leave() {
    let mut species = straight();
    species.limbs.levels[0].density = 1.0;
    species.limbs.levels[0].radius = 0.6;
    let (trunk, taper) = (species.trunk.radius, species.taper);
    let height = species.trunk.height;
    let axes = grow(&species).axes;
    let (stems, limbs) = axes.split_at(1);
    assert_eq!(limbs.len(), 2);
    // The upright trunk narrows evenly over its height, so its radius at a height is its base
    // radius less that share of the taper.
    let trunk_at = |y: f32| trunk * (1.0 - taper * y / height);
    for limb in limbs {
        let expected = 0.6 * trunk_at(limb.origin().y);
        assert!((limb.radii().0 - expected).abs() < 1e-5, "{limb:?}");
    }
    // A level's radius thickens that level and nothing else.
    species.limbs.levels[0].radius = 1.0;
    assert_eq!(grow(&species).axes[0].rings, stems[0].rings);
}

#[test]
fn a_tube_is_drawn_at_its_rings_radii() {
    let species = straight();
    let trunk = &grow(&species).axes[0];
    let mut geometry = Geometry::default();
    let sides = 6;
    tube(
        &mut geometry,
        &trunk.rings,
        &vec![sides; trunk.rings.len()],
        0.0,
    );
    // The upright trunk narrows evenly over its height, and keeps its base radius below the ground.
    let per_metre = species.trunk.radius * species.taper / species.trunk.height;
    for (i, ring) in trunk.rings.iter().enumerate() {
        let expected = species.trunk.radius - per_metre * ring.at.max(0.0);
        assert!((ring.radius - expected).abs() < 1e-5, "{ring:?}");
        let vertex = geometry.positions[i * sides as usize];
        assert!(((vertex - ring.center).length() - ring.radius).abs() < 1e-5);
    }
}

#[test]
fn stems_reach_below_the_ground_by_their_base_radius() {
    let mut species = straight();
    species.trunk.lean = Toggle::on(Lean::degrees(30.0));
    let trunk = &grow(&species).axes[0];
    let (below, ground) = (trunk.rings[0], trunk.rings[1]);
    assert_eq!(ground.center, Vec3::ZERO);
    assert!((below.center - Vec3::new(0.0, -species.trunk.radius, 0.0)).length() < 1e-5);
    assert_eq!(below.radius, ground.radius);
}

#[test]
fn a_flare_widens_the_base_and_meets_the_taper() {
    let mut species = straight();
    let bare = grow(&species).axes[0].clone();
    species.trunk.flare = Toggle::on(Flare { width: 1.0 });
    let flared = &grow(&species).axes[0];
    let radius = species.trunk.radius;
    // Twice as wide at the ground, and below it.
    assert!((flared.frame_at(0.0).radius - 2.0 * radius).abs() < 1e-5);
    assert!((flared.rings[0].radius - 2.0 * radius).abs() < 1e-5);
    // Rings added along the curve, narrowing all the way, and unchanged past three base radii.
    assert!(flared.rings.len() >= bare.rings.len() + 2);
    for pair in flared.rings.windows(2) {
        assert!(pair[1].radius <= pair[0].radius + 1e-6, "{pair:?}");
    }
    let above = |axis: &Axis| -> Vec<Ring> {
        (axis.rings.iter().filter(|ring| ring.at > 3.0 * radius + 1e-4)).copied().collect()
    };
    assert_eq!(above(flared), above(&bare));
}

#[test]
fn a_parent_runs_on_past_its_last_node_to_its_tip() {
    let mut species = straight();
    species.limbs.shown = 2;
    let axes = grow(&species).axes;
    // The trunk keeps its height, its last limb leaving below the tip.
    let trunk = &axes[0];
    assert!((trunk.length() - species.trunk.height).abs() < 1e-4);
    let last = (axes.iter().filter(|limb| limb.level == 1))
        .map(|limb| limb.parent.unwrap().1)
        .fold(0.0, f32::max);
    assert!(last < trunk.length());
    // Every tip bears leaves, the trunk's and those of limbs bearing limbs too.
    let bearing = (axes[1..].iter()).filter(|limb| limb.bears(&axes, AxisKind::Lateral));
    assert!(bearing.count() > 0);
    let mut leafy: Vec<usize> = grow(&species).cards.iter().map(|card| card.axis).collect();
    leafy.dedup();
    assert_eq!(leafy, (0..axes.len()).collect::<Vec<_>>());
}

/// `straight`'s first level, every limb forking halfway into `copies` at 60° across, the Y's
/// plane turned by `roll` in degrees.
fn forking(copies: i32, roll: f32) -> Species {
    let mut species = straight();
    let level = &mut species.limbs.levels[0];
    level.density = 1.0;
    level.fork.state = OperatorState::On;
    level.fork.settings.chance = 1.0;
    level.fork.settings.at.center = 0.5;
    level.fork.settings.copies = copies;
    level.fork.settings.angle.center = 60.0_f32.to_radians();
    level.fork.settings.roll = Fuzzy::new(roll.to_radians(), 0.0, Distribution::Bell);
    species
}

/// Each forked axis among `axes`, and its fork's copies.
fn forks(axes: &[Axis]) -> Vec<(&Axis, Vec<&Axis>)> {
    (axes.iter())
        .filter_map(|axis| {
            let copies: Vec<&Axis> = (axis.scions.iter())
                .map(|&scion| &axes[scion])
                .filter(|scion| scion.kind == AxisKind::ForkCopy)
                .collect();
            (!copies.is_empty()).then_some((axis, copies))
        })
        .collect()
}

#[test]
fn a_fork_spreads_its_copies_from_its_tip() {
    for (copies, roll) in [(2, 0.0), (2, 90.0), (3, 0.0)] {
        let axes = grow(&forking(copies, roll)).axes;
        let forks = forks(&axes);
        assert!(!forks.is_empty());
        for (limb, copies) in forks {
            let tip = limb.rings[limb.rings.len() - 1];
            let upper = upper_side(tip.tangent, tip.side);
            let mut spread = Vec3::ZERO;
            for copy in &copies {
                // On the tip, at its radius, turned half the fork angle from it.
                assert!((copy.origin() - tip.center).length() < 1e-5);
                assert_eq!(copy.radii().0, limb.radii().1);
                let angle = copy.direction().angle_between(tip.tangent).to_degrees();
                assert!((angle - 30.0).abs() < 1e-2, "{angle}");
                spread += copy.direction().reject_from_normalized(tip.tangent);
            }
            // Spread evenly around it.
            assert!(spread.length() < 1e-4, "{spread}");
            if copies.len() == 2 {
                // Two lie level side by side at no roll, and one above the other at 90°.
                let apart = (copies[0].direction() - copies[1].direction()).normalize();
                let along_upper = apart.dot(upper).abs();
                let expected = if roll == 0.0 { 0.0 } else { 1.0 };
                assert!(
                    (along_upper - expected).abs() < 1e-3,
                    "{roll}: {along_upper}"
                );
            }
        }
    }
}

#[test]
fn a_forked_limb_bears_no_leaves_at_its_fork() {
    // One card at each tip, forks left out, with one level and with two.
    let mut species = forking(2, 0.0);
    for shown in [1, 2] {
        species.limbs.shown = shown;
        let skeleton = grow(&species);
        let axes = &skeleton.axes;
        assert!(!forks(axes).is_empty());
        let tips: Vec<usize> = (0..axes.len())
            .filter(|&index| !axes[index].bears(axes, AxisKind::ForkCopy))
            .collect();
        let mut leafy: Vec<usize> = skeleton.cards.iter().map(|card| card.axis).collect();
        leafy.dedup();
        assert_eq!(leafy, tips, "{shown}");
    }
}

#[test]
fn a_short_limb_forks_short() {
    // Each copy reaches no further than its limb had left to grow past the fork.
    let mut species = forking(2, 0.0);
    species.limbs.levels[0].reach.center = 0.3;
    let axes = grow(&species).axes;
    let forks = forks(&axes);
    assert!(!forks.is_empty());
    for (limb, copies) in forks {
        let left = limb.reach - limb.length();
        for copy in copies {
            assert!(copy.length() <= left + 1e-3, "{} {left}", copy.length());
        }
    }
}

#[test]
fn a_forked_limb_is_one_tube_with_its_first_copy() {
    let species = forking(2, 0.0);
    let axes = grow(&species).axes;
    let sides_of = |axis: &Axis| match axis.level {
        0 => species.trunk.sides as u32,
        _ => species.limbs.levels[0].sides as u32,
    };
    let forks = forks(&axes);
    assert!(!forks.is_empty());
    for (limb, copies) in &forks {
        // The limb's rings, then the first copy's past the one they share, all with its sides.
        let (rings, sides) = chain(&axes, limb, sides_of);
        let joint = limb.rings.len() - 1;
        assert_eq!(rings.len(), joint + copies[0].rings.len());
        let tip = |rings: &[Ring]| rings[rings.len() - 1].center;
        assert_eq!(tip(&rings), tip(&copies[0].rings));
        assert!(sides.iter().all(|&count| count == sides[0]));
        // The shared ring faces halfway round the turn.
        let into = (rings[joint].center - rings[joint - 1].center).normalize();
        let out = (rings[joint + 1].center - rings[joint].center).normalize();
        assert!((rings[joint].tangent - (into + out).normalize()).length() < 1e-4);
        // The other copy is a tube of its own.
        assert_eq!(chain(&axes, copies[1], sides_of).0, copies[1].rings);
    }
    // No axis drawn twice: a tube from each axis but the first copies, each with a ring of
    // vertices per ring and its cap's center.
    let first_copies: Vec<&Axis> = forks.iter().map(|(_, copies)| copies[0]).collect();
    let vertices: usize = (axes.iter())
        .filter(|&axis| !first_copies.contains(&axis))
        .map(|axis| chain(&axes, axis, sides_of).1.iter().sum::<u32>() as usize + 1)
        .sum();
    assert_eq!(plant(&species).bark.positions.len(), vertices);
}

#[test]
fn a_forked_trunk_does_not_twist_into_its_first_copy() {
    // Each copy's own heading sets its rings' roll; the first carries on the trunk's tube from
    // the trunk's side, so it must start from it.
    for seed in 0..16 {
        let mut species = forked_trunk(regular(), 2, 20.0, 0.4);
        species.seed = Seed(seed);
        species.trunk.lean = Toggle::on(Lean::degrees(30.0));
        let axes = grow(&species).axes;
        let (rings, _) = chain(&axes, &axes[0], |_| 6);
        let joint = axes[0].rings.len() - 1;
        let (at, next) = (rings[joint], rings[joint + 1]);
        let twist = at
            .side
            .reject_from_normalized(next.tangent)
            .angle_between(next.side)
            .to_degrees();
        assert!(twist < 30.0, "seed {seed}: {twist}°");
    }
}

#[test]
fn a_chain_shares_its_joins() {
    // A trunk of six sides bent 90° at its tip, carried on level by a copy of three.
    let ring = |center: Vec3, tangent: Vec3, at: f32| Ring {
        center,
        tangent,
        side: Vec3::Z,
        at,
        radius: 0.1,
    };
    let stream = Seed(0).stream("test");
    let trunk = Axis::stem(
        vec![ring(Vec3::ZERO, Vec3::Y, 0.0), ring(Vec3::Y, Vec3::Y, 1.0)],
        stream.clone(),
    );
    let mut carrier = Axis::placed(
        1,
        AxisKind::ForkCopy,
        (0, 1.0),
        ring(Vec3::Y, Vec3::Y, 0.0),
        1.0,
        stream,
    );
    carrier
        .rings
        .push(ring(Vec3::new(1.0, 1.0, 0.0), Vec3::X, 1.0));
    let mut axes = vec![trunk, carrier];
    axes[0].scions.push(1);
    let (rings, sides) = chain(&axes, &axes[0], |axis| if axis.level == 0 { 6 } else { 3 });
    assert_eq!(sides, [6, 3, 3]);
    assert_eq!(
        rings.iter().map(|ring| ring.at).collect::<Vec<_>>(),
        [0.0, 1.0, 2.0]
    );
    // The join faces halfway, its side square to it.
    let halfway = Vec3::new(1.0, 1.0, 0.0).normalize();
    assert!((rings[1].tangent - halfway).length() < 1e-5);
    assert!(rings[1].side.dot(rings[1].tangent).abs() < 1e-5);
    // The stitch covers the segment: every vertex of both rings in a triangle, n + m of them.
    let mut geometry = Geometry::default();
    tube(&mut geometry, &rings[..2], &sides[..2], 0.0);
    assert_eq!(geometry.triangles(), (6 + 3) + 3);
    let stitch = &geometry.indices[..(6 + 3) * 3];
    assert!((0..6 + 3).all(|vertex| stitch.contains(&vertex)));
}

#[test]
fn thin_limbs_are_drawn_as_wide_as_the_detail_threshold() {
    let species = shown(3, regular());
    let thick = Species {
        detail: 0.1,
        ..species.clone()
    };
    let (thin, thick) = (plant(&species), plant(&thick));
    // Every stem straight, so no bend is kept for the threshold: the same limbs and leaves,
    // drawn thicker.
    assert_eq!(thick.level_limbs, thin.level_limbs);
    assert_ne!(thick.bark.positions, thin.bark.positions);
    assert_eq!(thick.leaves.positions, thin.leaves.positions);
    // The thinnest tube is drawn 10 cm wide: some ring vertex lies 5 cm off its tip.
    let tip = |limb: &Axis| limb.rings[limb.rings.len() - 1].center;
    let thinnest = limbs(&species)
        .into_iter()
        .min_by(|a, b| a.radii().1.total_cmp(&b.radii().1))
        .unwrap();
    assert!(thinnest.radii().1 < 0.05);
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
    species.limbs.levels[0].density = 1.0;
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
                species.limbs.levels[0].density = density;
                species.limbs.levels[0].per_node = per_node;
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
            ..shown(3, Species::default())
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

/// In a card's corners: the middle of its base, its base's left and right ends, the middle of
/// its axial fold, and the middle of its tip.
const BASE: usize = 1;
const FOLD: usize = 2;
const BASE_LEFT: usize = 0;
const BASE_RIGHT: usize = 5;
const TIP: usize = 10;

#[test]
fn a_card_at_every_branch_end() {
    // Two levels, whose twigs kink near their tips, the second's rising from their parents' upper
    // sides, so their tips are steep and the first's are not.
    let mut species = shown(2, Species::default());
    species.limbs.levels[1].facing = Facing::Up;
    let leaves = plant(&species).leaves;
    let cards: Vec<&[Vec3]> = leaves.positions.chunks(CARD_CORNERS).collect();
    // Every tip, the trunk's and those of limbs bearing limbs too.
    let ends = grow(&species).axes;
    let tips: Vec<Vec3> = (ends.iter())
        .map(|limb| limb.rings[limb.rings.len() - 1].center)
        .collect();
    // As many as the tips' steepness asks for, some tips bearing more than one.
    let per_tip: Vec<usize> = (ends.iter())
        .map(|limb| {
            let elevation = limb.frame_at(limb.length()).tangent.y.asin();
            cards_at(elevation, species.ornaments.tip_leaves.settings.upright) as usize
        })
        .collect();
    assert_eq!(cards.len(), per_tip.iter().sum::<usize>());
    assert!(per_tip.contains(&1) && per_tip.iter().any(|&count| count > 1));
    // Each tip between a card's base and fold: at its fold, or short of it on a short twig, or
    // at the base of a rosette's card.
    let spans = |card: &[Vec3], tip: Vec3| distance_to_segment(tip, card[BASE], card[FOLD]) < 1e-4;
    assert!(
        tips.iter()
            .all(|tip| cards.iter().any(|card| spans(card, *tip)))
    );
    for state in [OperatorState::Disabled, OperatorState::Off] {
        species.ornaments.tip_leaves.state = state;
        assert!(plant(&species).leaves.positions.is_empty());
    }
}

#[test]
fn cards_lie_along_their_twigs_facing_up() {
    let tip_leaves = TipLeaves {
        lateral_droop: 0.0,
        axial_droop: 0.0,
        ..TipLeaves::default()
    };
    let side = tip_leaves.size * Envelope::default().radius;
    let species = Species {
        ornaments: Ornaments {
            tip_leaves: Toggle::on(tip_leaves),
        },
        ..shown(2, regular())
    };
    let twigs = limbs(&species);
    let skeleton = grow(&species);
    let leaves = plant(&species).leaves;
    assert!(!leaves.positions.is_empty());
    assert_eq!(leaves.uvs.len(), leaves.positions.len());
    let cards = leaves.positions.chunks(CARD_CORNERS);
    let cards = cards.zip(leaves.normals.chunks(CARD_CORNERS));
    // Rosettes' cards turn from their twigs (`steep_tips_bear_rosettes`).
    let mut along = 0;
    for (cluster, (corners, normals)) in skeleton.cards.iter().zip(cards) {
        let twig = &skeleton.axes[cluster.axis];
        let elevation = twig.frame_at(twig.length()).tangent.y.asin();
        if cards_at(elevation, species.ornaments.tip_leaves.settings.upright) > 1 {
            continue;
        }
        along += 1;
        let base = corners[BASE];
        let axis = (corners[TIP] - base) / side;
        let across = (corners[BASE_RIGHT] - corners[BASE_LEFT]) / side;
        // Flat, so every quad faces the same way.
        let normal = normals[0];
        assert!(normals.iter().all(|n| (*n - normal).length() < 1e-4));
        assert!((across.length() - 1.0).abs() < 1e-4 && (axis.length() - 1.0).abs() < 1e-4);
        // Level across, facing as near up as the axis allows.
        assert!(across.y.abs() < 1e-4, "{across}");
        assert!(normal.dot(axis).abs() < 1e-4 && normal.y >= 0.0, "{normal}");
        assert!((normal - Vec3::Y.reject_from(axis).normalize()).length() < 1e-3);
        // The base on a twig, which the axis follows: straight twigs, so their direction. A
        // short twig's tip card starts at its base, where its parent and siblings meet too.
        assert!(twigs.iter().any(|twig| {
            let from = base - twig.origin();
            from.cross(twig.direction()).length() < 1e-3
                && axis.angle_between(twig.direction()) < 1e-3
        }));
    }
    assert!(along > 0);
}

#[test]
fn steep_tips_bear_rosettes() {
    // A lone straight twig a metre long, rising at `elevation` toward +X, its tip as `upright`
    // has it.
    let cards_as = |elevation: f32, upright: Upright| {
        let (rise, run) = elevation.to_radians().sin_cos();
        let tangent = Vec3::new(run, rise, 0.0);
        let ring = |at: f32| Ring {
            center: tangent * at,
            tangent,
            side: Vec3::Z,
            at,
            radius: 0.01,
        };
        let stream = Seed(0).stream("test");
        let mut twig = Axis::placed(1, AxisKind::Lateral, (0, 0.0), ring(0.0), 1.0, stream);
        twig.rings.push(ring(1.0));
        let mut species = Species::default();
        species.ornaments.tip_leaves.settings.upright = upright;
        let cards = clusters(&species, &[twig]);
        assert!(cards.iter().all(|card| card.direction.is_normalized()));
        (cards, tangent)
    };
    let cards = |elevation: f32| cards_as(elevation, Upright::Triple);
    let heading = |direction: Vec3| direction.xz().to_angle().to_degrees();
    let rise = |direction: Vec3| direction.y.asin().to_degrees();
    // Below 45°, one card along the twig, short of its tip.
    let (one, tangent) = cards(40.0);
    assert_eq!(one.len(), 1);
    assert!(one[0].at < 1.0 && one[0].direction.angle_between(tangent) < 1e-4);
    // Steep, the option's count from 45°.
    assert_eq!(cards(45.0).0.len(), 3);
    // At 60°, two at the tip, each turned ¾ of the way to level: one ahead, down from 60° to
    // 15°, and one over the top, through 120° to 30° behind.
    let (two, _) = cards_as(60.0, Upright::Double);
    assert_eq!(two.len(), 2);
    assert!(two.iter().all(|card| card.at == 1.0));
    assert!(heading(two[0].direction).abs() < 1e-2);
    assert!((rise(two[0].direction) - 15.0).abs() < 1e-2);
    assert!((heading(two[1].direction).abs() - 180.0).abs() < 1e-2);
    assert!((rise(two[1].direction) - 30.0).abs() < 1e-2);
    // Straight up, three, 120° apart, each 22.5° above level.
    let (three, _) = cards(90.0);
    assert_eq!(three.len(), 3);
    for (card, next) in three.iter().zip(three.iter().cycle().skip(1)) {
        let apart = (heading(card.direction) - heading(next.direction)).rem_euclid(360.0);
        assert!((apart - 120.0).abs() < 1e-2, "{apart}");
        assert!((rise(card.direction) - 22.5).abs() < 1e-2);
    }
    // Single keeps one card along the twig; Bare, none. Below 45° every choice bears the one
    // card.
    let (single, tangent) = cards_as(90.0, Upright::Single);
    assert_eq!(single.len(), 1);
    assert!(single[0].direction.angle_between(tangent) < 1e-4);
    assert!(cards_as(60.0, Upright::Bare).0.is_empty());
    for upright in [Upright::Bare, Upright::Single, Upright::Double] {
        assert_eq!(cards_as(40.0, upright).0.len(), 1);
    }
}

#[test]
fn cards_fold_at_the_midrib_and_across() {
    let frame = Frame {
        point: Vec3::ZERO,
        tangent: Vec3::X,
        side: Vec3::Z,
        radius: 0.0,
    };
    let folded = |lateral: f32, axial: f32| {
        let mut geometry = Geometry::default();
        let leaves = TipLeaves {
            lateral_droop: lateral.to_radians(),
            axial_droop: axial.to_radians(),
            ..TipLeaves::default()
        };
        card(&mut geometry, &frame, 2.0, 0.5, &leaves);
        geometry
    };
    // A level twig along x, so the card faces up and its left edge lies toward −z.
    let flat = folded(0.0, 0.0);
    assert!((flat.positions[TIP] - Vec3::new(2.0, 0.0, 0.0)).length() < 1e-5);
    // Drooped 30°, each edge drops half a width times sin 30°, and draws in; negative cups.
    let drooped = folded(30.0, 0.0);
    let edge = Vec3::new(0.0, -0.5, -30_f32.to_radians().cos());
    assert!((drooped.positions[BASE_LEFT] - edge).length() < 1e-5);
    assert!((folded(-30.0, 0.0).positions[BASE_LEFT].y - 0.5).abs() < 1e-5);
    // The midrib stays on the twig to the fold, and the far half turns down 30° from it.
    let bent = folded(0.0, 30.0);
    assert_eq!(bent.positions[BASE], Vec3::ZERO);
    let (sin, cos) = 30_f32.to_radians().sin_cos();
    assert!((bent.positions[TIP] - Vec3::new(1.0 + cos, -sin, 0.0)).length() < 1e-5);
    // Every quad keeps the flat card's normal, and its winding faces the same side.
    for geometry in [&drooped, &bent, &folded(-30.0, -30.0)] {
        assert!(geometry.normals.iter().all(|n| *n == Vec3::Y));
        for quad in geometry.positions.chunks(4) {
            assert!((quad[1] - quad[0]).cross(quad[3] - quad[0]).y > 0.0);
        }
    }
}

#[test]
fn leaves_grow_on_every_limb_bearing_none() {
    // A second level with no nodes leaves the first bearing the leaves, as if it were the last.
    let mut species = shown(2, Species::default());
    species.limbs.levels[1].density = 0.0;
    let leaves = |species: &Species| plant(species).leaves.positions;
    let bare_second = leaves(&species);
    species.limbs.shown = 1;
    assert!(!bare_second.is_empty());
    assert_eq!(bare_second, leaves(&species));
}
