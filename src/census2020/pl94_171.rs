use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Table {
	P1,
	P2,
	P3,
	P4,
	H1,
	P5,
}

pub use Table::{H1, P1, P2, P3, P4, P5};

#[cfg(test)]
const C2020_STYLE_EXAMPLES: [&str; 1] = [
	"PLST|RI|750|00|00|000|00|0019326|7500000US440070185003030|440070185003030|1|1|44|01219835|007|H4|01219781|80780|C5|01220079|||||||99999|99|99999999|80780|C5|01220079|018500|3|3030|9999|9|99999|99|99999999|999|99999|99|99999999|999999|9|99999|99|99999999|39300|1|148|99999|77200|1|715|99999|N|N||||01|||||020|||||051|||||443909|A||99999|99999|01200||1625|0|3030|Block 3030|S||0|0|+41.9866626|-071.4802535|BK||99999",
	// "PLST|WI|040|00|00|000|00|0000001|0400000US55|55|2|3|55|01779806|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||140292246684|29343721650|Wisconsin|Wisconsin|A||5893718|2727726|+44.6309071|-089.7093916|00||",
	// "PLST|WI|150|00|00|000|00|0004174|1500000US550250026024|550250026024|2|3|55|01779806|025|H1|01581072||||||||||||||||002602|4||||||||||||||||31540|1|357|99999||||||||||||||||||||||||||||||||926332|0|4|Block Group 4|S||1664|945|+43.1407442|-089.3031046|BG||",
	// "PLST|WI|970|00|00|000|00|0346752|9700000US5599999|5599999|2|3|55|01779806|||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||99999||3593567423|372310966|Wisconsin|Remainder of Wisconsin|F||222465|110434|+43.4603631|-088.5091312|00||",
];

macro_rules! generate_field_getter {
	($container_type:ty, $container_data_field:ident, $name:ident, $width:literal, [ $vis:vis , $getter_name:ident #> $pty:ty ]) => {
		#[allow(dead_code)]
		impl $container_type {
			#[must_use]
			$vis fn $getter_name(&self) -> $pty {
				debug_assert!(self.$container_data_field[$name].len() <= $width);
				self.$container_data_field[$name].parse::<$pty>().unwrap()
			}
		}
	};

	($container_type:ty, $container_data_field:ident, $name:ident, $width:literal, [ $vis:vis , $getter_name:ident #> $pty:ty | e.g. $expected:tt ]) => {
		generate_field_getter!($container_type, $container_data_field, $name, $width, [ $vis , $getter_name #> $pty ]);

		#[test]
		fn $getter_name() {
			for example in C2020_STYLE_EXAMPLES {
				let geo_header = <$container_type>::new(example);
				let expected = $expected;
				assert_eq!(geo_header.$getter_name(), expected);
			}
		}
	};



	($container_type:ty, $container_data_field:ident, $name:ident, $width:literal, [ $vis:vis , $getter_name:ident ]) => {
		#[allow(dead_code)]
		impl $container_type {
			#[must_use]
			$vis fn $getter_name(&self) -> &str {
				debug_assert!(self.$container_data_field[$name].len() <= $width);
				&self.$container_data_field[$name]
			}
		}
	};

	($container_type:ty, $container_data_field:ident, $name:ident, $width:literal, [ $vis:vis , $getter_name:ident e.g. $expected:literal ]) => {
		generate_field_getter!($container_type, $container_data_field, $name, $width, [ $vis , $getter_name ]);

		#[test]
		fn $getter_name() {
			for example in C2020_STYLE_EXAMPLES {
				let geo_header = <$container_type>::new(example);
				assert_eq!(geo_header.$getter_name(), $expected);
			}
		}
	};


	($container_type:ty, $container_data_field:ident, $name:ident, $width:literal, [ $vis:vis , | $getter_name:ident | ]) => {
		#[allow(dead_code)]
		impl $container_type {
			#[must_use]
			$vis fn $getter_name(&self) -> &str {
				debug_assert!(self.$container_data_field[$name].len() <= $width);
				&self.$container_data_field[$name].trim()
			}
		}
	};

	($container_type:ty, $container_data_field:ident, $name:ident, $width:literal, [ $vis:vis , | $getter_name:ident | e.g. $expected:literal ]) => {
		generate_field_getter!($container_type, $container_data_field, $name, $width, [ $vis , |$getter_name| ]);

		#[test]
		fn $getter_name() {
			for example in C2020_STYLE_EXAMPLES {
				let geo_header = <$container_type>::new(example);
				assert_eq!(geo_header.$getter_name(), $expected);
			}
		}
	};
}

macro_rules! generate_fields_inner {
	($container_type:ident, $container_data_field:ident, $name:ident, {}) => {};

	($container_type:ident, $container_data_field:ident, $name:ident, { @ + $loc:literal w $width:literal - $rest:tt }) => {
		#[allow(dead_code)]
		const $name: usize = $loc;

		generate_field_getter!($container_type, $container_data_field, $name, $width, $rest);
	};
}

macro_rules! generate_field {
	($container_type:ident, $container_data_field:ident, $name:ident $rest:tt) => {
		generate_fields_inner!($container_type, $container_data_field, $name, $rest);
	};
}

// Record codes
generate_field!(GeographicalHeader, data, FILEID { @ + 0 w 6 - [pub, fileid e.g. "PLST"] });
generate_field!(GeographicalHeader, data, STUSAB { @ + 1 w 2 - [pub, stusab e.g. "RI"] });
generate_field!(GeographicalHeader, data, SUMLEV { @ + 2 w 3 - [pub, sumlev e.g. "750"] });
// TODO(rye) 2020 +field: GEOVAR)
generate_field!(GeographicalHeader, data, GEOVAR { @ + 3 w 2 - [pub, geovar e.g. "00"] });
generate_field!(GeographicalHeader, data, GEOCOMP { @ + 4 w 2 - [pub, geocomp e.g. "00"] });
generate_field!(GeographicalHeader, data, CHARITER { @ + 5 w 3 - [pub, chariter e.g. "000"] });
generate_field!(GeographicalHeader, data, CIFSN { @ + 6 w 2 - [pub, cifsn e.g. "00"] });
generate_field!(GeographicalHeader, data, LOGRECNO { @ + 7 w 7 - [pub, logrecno #> crate::LogicalRecordNumber | e.g. 19326_u64] });

// Geographic Area Code)
generate_field!(GeographicalHeader, data, GEOID { @ + 8 w 60 - [ pub, geoid e.g. "7500000US440070185003030" ] });
generate_field!(GeographicalHeader, data, GEOCODE { @ + 9 w 51 - [ pub, geocode e.g. "440070185003030" ] });
generate_field!(GeographicalHeader, data, REGION { @ + 10 w 1 - [ pub, region e.g. "1" ] });
generate_field!(GeographicalHeader, data, DIVISION { @ + 11 w 1 - [ pub, division e.g. "1" ] });
generate_field!(GeographicalHeader, data, STATE { @ + 12 w 2 - [ pub, state e.g. "44" ] });
generate_field!(GeographicalHeader, data, STATENS { @ + 13 w 8 - [ pub, statens e.g. "01219835" ] });
generate_field!(GeographicalHeader, data, COUNTY { @ + 14 w 3 - [ pub, county e.g. "007" ] });
generate_field!(GeographicalHeader, data, COUNTYCC { @ + 15 w 2 - [ pub, countycc e.g. "H4" ] });
generate_field!(GeographicalHeader, data, COUNTYNS { @ + 16 w 8 - [ pub, countyns e.g. "01219781" ] });
generate_field!(GeographicalHeader, data, COUSUB { @ + 17 w 5 - [ pub, cousub e.g. "80780" ] });
generate_field!(GeographicalHeader, data, COUSUBCC { @ + 18 w 2 - [ pub, cousubcc e.g. "C5" ] });
generate_field!(GeographicalHeader, data, COUSUBNS { @ + 19 w 8 - [ pub, cousubns e.g. "01220079" ] });
generate_field!(GeographicalHeader, data, SUBMCD { @ + 20 w 5 - [ pub, submcd e.g. "" ] });
generate_field!(GeographicalHeader, data, SUBMCDCC { @ + 21 w 2 - [ pub, submcdcc e.g. "" ] });
generate_field!(GeographicalHeader, data, SUBMCDNS { @ + 22 w 8 - [ pub, submcdns e.g. "" ] });
generate_field!(GeographicalHeader, data, ESTATE { @ + 23 w 5 - [ pub, estate e.g. "" ] });
generate_field!(GeographicalHeader, data, ESTATECC { @ + 24 w 2 - [ pub, estatecc e.g. "" ] });
generate_field!(GeographicalHeader, data, ESTATENS { @ + 25 w 8 - [ pub, estatens e.g. "" ] });
generate_field!(GeographicalHeader, data, CONCIT { @ + 26 w 5 - [ pub, concit e.g. "99999" ] });
generate_field!(GeographicalHeader, data, CONCITCC { @ + 27 w 2 - [ pub, concitcc e.g. "99" ] });
generate_field!(GeographicalHeader, data, CONCITNS { @ + 28 w 8 - [ pub, concitns e.g. "99999999" ] });
generate_field!(GeographicalHeader, data, PLACE { @ + 29 w 5 - [ pub, place e.g. "80780" ] });
generate_field!(GeographicalHeader, data, PLACECC { @ + 30 w 2 - [ pub, placecc e.g. "C5" ] });
generate_field!(GeographicalHeader, data, PLACENS { @ + 31 w 8 - [ pub, placens e.g. "01220079" ] });
generate_field!(GeographicalHeader, data, TRACT { @ + 32 w 6 - [ pub, tract e.g. "018500" ] });
generate_field!(GeographicalHeader, data, BLKGRP { @ + 33 w 1 - [ pub, blkgrp e.g. "3" ] });
generate_field!(GeographicalHeader, data, BLOCK { @ + 34 w 4 - [ pub, block e.g. "3030" ] });

generate_field!(GeographicalHeader, data, AIANHH { @ + 35 w 4 - [ pub, aianhh e.g. "9999" ] });
generate_field!(GeographicalHeader, data, AIHHTLI { @ + 36 w 1 - [ pub, aihhtli e.g. "9" ] });
generate_field!(GeographicalHeader, data, AIANHHFP { @ + 37 w 5 - [ pub, aianhhfp e.g. "99999" ] });
generate_field!(GeographicalHeader, data, AIANHHCC { @ + 38 w 2 - [ pub, aianhhcc e.g. "99" ] });
generate_field!(GeographicalHeader, data, AIANHHNS { @ + 39 w 8 - [ pub, aianhhns e.g. "99999999" ] });
generate_field!(GeographicalHeader, data, AITS { @ + 40 w 3 - [ pub, aits e.g. "999" ] });
generate_field!(GeographicalHeader, data, AITSFP { @ + 41 w 5 - [ pub, aitsfp e.g. "99999" ] });
generate_field!(GeographicalHeader, data, AITSCC { @ + 42 w 2 - [ pub, aitscc e.g. "99" ] });
generate_field!(GeographicalHeader, data, AITSNS { @ + 43 w 8 - [ pub, aitsns e.g. "99999999" ] });
generate_field!(GeographicalHeader, data, TTRACT { @ + 44 w 6 - [ pub, ttract e.g. "999999" ] });
generate_field!(GeographicalHeader, data, TBLKGRP { @ + 45 w 1 - [ pub, tblkgrp e.g. "9" ] });
generate_field!(GeographicalHeader, data, ANRC { @ + 46 w 5 - [ pub, anrc e.g. "99999" ] });
generate_field!(GeographicalHeader, data, ANRCCC { @ + 47 w 2 - [ pub, anrccc e.g. "99" ] });
generate_field!(GeographicalHeader, data, ANRCNS { @ + 48 w 8 - [ pub, anrcns e.g. "99999999" ] });
generate_field!(GeographicalHeader, data, CBSA { @ + 49 w 5 - [ pub, cbsa e.g. "39300" ] });
generate_field!(GeographicalHeader, data, MEMI { @ + 50 w 1 - [ pub, memi e.g. "1" ] });
generate_field!(GeographicalHeader, data, CSA { @ + 51 w 3 - [ pub, csa e.g. "148" ] });
generate_field!(GeographicalHeader, data, METDIV { @ + 52 w 5 - [ pub, metdiv e.g. "99999" ] });
generate_field!(GeographicalHeader, data, NECTA { @ + 53 w 5 - [ pub, necta e.g. "77200" ] });
generate_field!(GeographicalHeader, data, NMEMI { @ + 54 w 1 - [ pub, nmemi e.g. "1" ] });
generate_field!(GeographicalHeader, data, CNECTA { @ + 55 w 3 - [ pub, cnecta e.g. "715" ] });
generate_field!(GeographicalHeader, data, NECTADIV { @ + 56 w 5 - [ pub, nectadiv e.g. "99999" ] });
generate_field!(GeographicalHeader, data, CBSAPCI { @ + 57 w 1 - [ pub, cbsapci e.g. "N" ] });
generate_field!(GeographicalHeader, data, NECTAPCI { @ + 58 w 1 - [ pub, nectapci e.g. "N" ] });
generate_field!(GeographicalHeader, data, UA { @ + 59 w 5 - [ pub, ua e.g. "" ] });
generate_field!(GeographicalHeader, data, UATYPE { @ + 60 w 1 - [ pub, uatype e.g. "" ] });
generate_field!(GeographicalHeader, data, UR { @ + 61 w 1 - [ pub, ur e.g. "" ] });
generate_field!(GeographicalHeader, data, CD116 { @ + 62 w 2 - [ pub, cd116 e.g. "01" ] });
generate_field!(GeographicalHeader, data, CD118 { @ + 63 w 2 - [ pub, cd118 e.g. "" ] });
generate_field!(GeographicalHeader, data, CD119 { @ + 64 w 2 - [ pub, cd119 e.g. "" ] });
generate_field!(GeographicalHeader, data, CD120 { @ + 65 w 2 - [ pub, cd120 e.g. "" ] });
generate_field!(GeographicalHeader, data, CD121 { @ + 66 w 2 - [ pub, cd121 e.g. "" ] });
generate_field!(GeographicalHeader, data, SLDU18 { @ + 67 w 3 - [ pub, sldu18 e.g. "020" ] });
generate_field!(GeographicalHeader, data, SLDU22 { @ + 68 w 3 - [ pub, sldu22 e.g. "" ] });
generate_field!(GeographicalHeader, data, SLDU24 { @ + 69 w 3 - [ pub, sldu24 e.g. "" ] });
generate_field!(GeographicalHeader, data, SLDU26 { @ + 70 w 3 - [ pub, sldu26 e.g. "" ] });
generate_field!(GeographicalHeader, data, SLDU28 { @ + 71 w 3 - [ pub, sldu28 e.g. "" ] });
generate_field!(GeographicalHeader, data, SLDL18 { @ + 72 w 3 - [ pub, sldl18 e.g. "051" ] });
generate_field!(GeographicalHeader, data, SLDL22 { @ + 73 w 3 - [ pub, sldl22 e.g. "" ] });
generate_field!(GeographicalHeader, data, SLDL24 { @ + 74 w 3 - [ pub, sldl24 e.g. "" ] });
generate_field!(GeographicalHeader, data, SLDL26 { @ + 75 w 3 - [ pub, sldl26 e.g. "" ] });
generate_field!(GeographicalHeader, data, SLDL28 { @ + 76 w 3 - [ pub, sldl28 e.g. "" ] });
generate_field!(GeographicalHeader, data, VTD { @ + 77 w 6 - [ pub, vtd e.g. "443909" ] });
generate_field!(GeographicalHeader, data, VTDI { @ + 78 w 1 - [ pub, vtdi e.g. "A" ] });
generate_field!(GeographicalHeader, data, ZCTA { @ + 79 w 5 - [ pub, zcta e.g. "" ] });
generate_field!(GeographicalHeader, data, SDELM { @ + 80 w 5 - [ pub, sdelm e.g. "99999" ] });
generate_field!(GeographicalHeader, data, SDSEC { @ + 81 w 5 - [ pub, sdsec e.g. "99999" ] });
generate_field!(GeographicalHeader, data, SDUNI { @ + 82 w 5 - [ pub, sduni e.g. "01200" ] });
generate_field!(GeographicalHeader, data, PUMA { @ + 83 w 5 - [ pub, puma e.g. "" ] });

// Area Characteristic)
generate_field!(GeographicalHeader, data, AREALAND { @ + 84 w 14 - [ pub, arealand e.g. "1625" ] });
generate_field!(GeographicalHeader, data, AREAWATR { @ + 85 w 14 - [ pub, areawatr e.g. "0" ] });
generate_field!(GeographicalHeader, data, BASENAME { @ + 86 w 100 - [ pub, basename e.g. "3030" ] });
generate_field!(GeographicalHeader, data, NAME { @ + 87 w 125 - [ pub, name e.g. "Block 3030" ] });
generate_field!(GeographicalHeader, data, FUNCSTAT { @ + 88 w 1 - [ pub, funcstat e.g. "S" ] });
generate_field!(GeographicalHeader, data, GCUNI { @ + 89 w 1 - [ pub, gcuni e.g. "" ] });
generate_field!(GeographicalHeader, data, POP100 { @ + 90 w 9 - [ pub, pop100 e.g. "0" ] });
generate_field!(GeographicalHeader, data, HU100 { @ + 91 w 9 - [ pub, hu100 e.g. "0" ] });
generate_field!(GeographicalHeader, data, INTPTLAT { @ + 92 w 11 - [ pub, intptlat e.g. "+41.9866626" ] });
generate_field!(GeographicalHeader, data, INTPTLON { @ + 93 w 12 - [ pub, intptlon e.g. "-071.4802535" ] });
generate_field!(GeographicalHeader, data, LSADC { @ + 94 w 2 - [ pub, lsadc e.g. "BK" ] });
generate_field!(GeographicalHeader, data, PARTFLAG { @ + 95 w 1 - [ pub, partflag e.g. "" ] });

// Special Area Code)
generate_field!(GeographicalHeader, data, UGA { @ + 96 w 5 - [ pub, uga e.g. "99999" ] });

pub struct GeographicalHeader {
	data: Vec<String>,
}

impl GeographicalHeader {
	pub fn new(data: &str) -> Self {
		Self {
			data: data.split('|').map(str::to_owned).collect(),
		}
	}
}

impl crate::GeographicalHeader for GeographicalHeader {
	fn name(&self) -> &str {
		self.name()
	}

	fn logrecno(&self) -> crate::LogicalRecordNumber {
		self.logrecno()
	}
}
