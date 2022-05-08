mod feature_id;
use feature_id::feature_id;

mod point;

mod geometry_interner;
pub use geometry_interner::GeometryInterner;

mod geoid;
pub use geoid::GeoId;
