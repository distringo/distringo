#![deny(clippy::all)]
#![warn(clippy::cargo)]
#![warn(clippy::pedantic)]
#![warn(clippy::cognitive_complexity)]

mod feature_id;
use feature_id::feature_id;

mod point;

mod geometry_interner;
pub use geometry_interner::GeometryInterner;
