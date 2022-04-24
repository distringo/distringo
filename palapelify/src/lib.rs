#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeoScalar(i32);

impl From<f64> for GeoScalar {
	fn from(f64: f64) -> Self {
		debug_assert!(f64 < 180.00 && f64 > -180.0);

		Self((f64 * 1E6).trunc() as i32)
	}
}

impl From<GeoScalar> for f64 {
	fn from(geo_scalar: GeoScalar) -> f64 {
		(geo_scalar.0 as f64) / 1E6
	}
}

#[cfg(test)]
mod geoscalar {
	use super::GeoScalar;

	#[cfg(test)]
	mod from_f64 {
		use super::GeoScalar;

		impl core::fmt::Debug for GeoScalar {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				f.debug_tuple("GeoScalar").field(&self.0).finish()
			}
		}

		#[test]
		fn converts_positive_in_range() {
			let degrees: f64 = 87.31275;
			assert_eq!(GeoScalar::from(degrees), GeoScalar(87312750));
		}

		#[test]
		fn converts_negative_in_range() {
			let degrees: f64 = -37.172718;
			assert_eq!(GeoScalar::from(degrees), GeoScalar(-37172718));
		}

		#[test]
		#[should_panic]
		fn asserts_in_range_positive() {
			let degrees: f64 = 180.0;
			let _ = GeoScalar::from(degrees);
		}

		#[test]
		#[should_panic]
		fn asserts_in_range_negative() {
			let degrees: f64 = -180.0;
			let _ = GeoScalar::from(degrees);
		}
	}
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct GeoId(String);

impl From<String> for GeoId {
	fn from(string: String) -> Self {
		Self(string)
	}
}

impl core::fmt::Display for GeoId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[cfg(test)]
mod geoid {
	use super::GeoId;
	#[test]
	fn clone() {
		let string: String = "I am a string!".to_string();
		assert!(core::ptr::eq(&string, &string));

		let geoid: GeoId = GeoId(string.clone());
		assert!(!core::ptr::eq(&string, &geoid.0));

		let clone = geoid.clone();
		assert!(!core::ptr::eq(&geoid, &clone));

		assert!(!core::ptr::eq(&geoid.0, &clone.0));

		assert_eq!(&geoid.0, &string);
		assert_eq!(&clone.0, &string);
	}

	#[test]
	fn from_string() {
		let string: String = "I am a string!".to_string();

		let geoid: GeoId = string.clone().into();
		assert_eq!(geoid.0, string);
	}
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeometryPoint([GeoScalar; 2]);

impl From<geo::Coordinate<f64>> for GeometryPoint {
	fn from(coordinate: geo::Coordinate<f64>) -> Self {
		GeometryPoint([coordinate.y.into(), coordinate.x.into()])
	}
}
