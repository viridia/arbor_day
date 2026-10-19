//! The generator: a species in, a plant's geometry out. Plain functions on Bevy's math types, with
//! no ECS, so they can move to a library of their own (ledger X1).

mod envelope;
mod foliage;
mod fork;
mod hint;
mod leaf;
mod mesh;
mod placement;
mod skeleton;
mod trace;
mod wander;

#[cfg(test)]
mod tests;

pub use envelope::{Envelope, envelope_helper};
pub use leaf::LeafTexture;
pub use mesh::{CARD_CORNERS, Geometry, plant};
