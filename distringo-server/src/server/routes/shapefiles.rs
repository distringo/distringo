use std::collections::HashMap;

use axum::{extract::Path, response::IntoResponse, routing::get, Json, Router};

use serde::Deserialize;

type ShapefileId = String;

struct ShapefileDatabase {
	entries: HashMap<ShapefileId, LazyFileContents>,
}

struct LazyFileContents {
	file_handle: std::fs::File,
	contents: Option<std::borrow::Cow<'static, str>>,
}

impl LazyFileContents {
	fn try_from_file_name<P: AsRef<std::path::Path>>(file: P) -> std::io::Result<Self> {
		let file_handle = std::fs::File::open(file)?;

		Ok(Self {
			file_handle,
			contents: None,
		})
	}

	async fn get_contents(&self) -> String {
		if let Some(str) = &self.contents {
			str.to_string()
		} else {
			todo!()
		}
	}
}

impl From<&crate::settings::ShapefilesConfig> for ShapefileDatabase {
	#[tracing::instrument(skip_all)]
	fn from(config: &crate::settings::ShapefilesConfig) -> Self {
		tracing::trace!("initializing shapefile database");

		let mut entries = HashMap::new();

		let shapefiles: &HashMap<String, crate::settings::ShapefileConfig> = config.as_ref();

		for (key, shapefile_config) in shapefiles {
			// let config: ShapefileConfig = shapefile_config.parse();

			tracing::debug!(id = key, "loading shapefile");

			// entries.insert(key, todo!());
		}

		Self { entries }
	}
}

impl ShapefileDatabase {
	#[tracing::instrument(skip(self))]
	async fn index(&self) -> impl IntoResponse + '_ {
		Json(self.entries.keys().collect::<Vec<_>>())
	}

	// #[tracing::instrument(skip(self))]
	async fn show(&self, Path(id): Path<ShapefileId>) -> impl IntoResponse + '_ {
		if let Some(shapefile) = self.entries.get(&id) {
			tracing::debug!(?id, "shapefile found");

			let contents: String = shapefile.get_contents().await;

			Ok(contents)
		} else {
			tracing::debug!(?id, "shapefile not found");
			Err(http::StatusCode::NOT_FOUND)
		}
	}
}

pub(crate) fn router(config: &crate::settings::ShapefilesConfig) -> Router {
	let db: ShapefileDatabase = config.into();

	let db = Box::leak(Box::new(db));

	Router::new()
		.route("/", get(|| db.index()))
		.route("/:id", get(|id| db.show(id)))
}
