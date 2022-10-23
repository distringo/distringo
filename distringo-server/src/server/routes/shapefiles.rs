use std::collections::HashMap;

use axum::{extract::Path, response::IntoResponse, routing::get, Json, Router};

type ShapefileId = String;

struct ShapefileDatabase {
	entries: HashMap<ShapefileId, ()>,
}

impl ShapefileDatabase {
	#[tracing::instrument(skip(self))]
	async fn index(&self) -> impl IntoResponse + '_ {
		Json(self.entries.keys().collect::<Vec<_>>())
	}

	#[tracing::instrument(skip(self))]
	async fn show(&self, Path(id): Path<ShapefileId>) -> impl IntoResponse + '_ {
		if let Some(shapefile) = self.entries.get(&id) {
			tracing::debug!(?id, "Retrieved shapefile");
			Ok("found shapefile {}")
		} else {
			Err("no such shapefile")
		}
	}
}

pub(crate) fn router(config: &crate::settings::ShapefilesConfig) -> Router {
	println!("{:?}", config);

	let db = ShapefileDatabase {
		entries: HashMap::new(),
	};

	tracing::trace!("initializing shapefile database");

	let db = Box::leak(Box::new(db));

	let shapefiles = config.get_table("shapefiles");

	match shapefiles {
		Ok(shapefiles) => {
			for (key, shapefile_config) in shapefiles {
				tracing::debug!(id = key, "loading shapefile from {:?}", shapefile_config);

				db.entries.insert(key, ());
			}
		}
		Err(err) => tracing::warn!(error = %err, "error getting shapefiles from configuration"),
	};

	Router::new()
		.route("/", get(|| db.index()))
		.route("/:id", get(|id| db.show(id)))
}
