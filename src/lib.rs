mod error;
pub use crate::error::*;

mod config;
pub use crate::config::*;

// to get from a feature in a shapefile to its name (header only)
// use geoid## => geographical header (look up geoid## in GEOCODE column) -> name
//
// to get from a feature in a shapefile to its data (across # files)
// use geoid## => geographical header (geoid## in GEOCODE column) -> LOGRECNO => [for all files: offset for LOGRECNO]
//   (in debug mode, verify LOGRECNO match)
//   ()
