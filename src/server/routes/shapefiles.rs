use std::collections::HashMap;

use axum::{extract::Path, response::IntoResponse, routing::get, Json, Router};

#[derive(serde::Serialize)]
struct ShapefileList {
	shapefiles: Vec<String>,
}

type ShapefileId = String;

struct ShapefileDatabase {
	entries: HashMap<ShapefileId, ()>,
}

impl ShapefileDatabase {
	async fn index(&self) -> impl IntoResponse + '_ {
		Json(self.entries.keys().collect::<Vec<_>>())
		// "shapefiles index"
	}

	async fn show(&self, Path(id): Path<ShapefileId>) -> impl IntoResponse + '_ {
		if let Some(shapefile) = self.entries.get(&id) {
			Ok("found shapefile {}")
		} else {
			Err("no such shapefile")
		}
	}
}

pub(crate) fn router(_config: &config::Config) -> Router {
	let db = ShapefileDatabase {
		entries: HashMap::new(),
	};

	let db = Box::leak(Box::new(db));

	Router::new()
		.route("/", get(|| db.index()))
		.route("/:id", get(|id| db.show(id)))
}
