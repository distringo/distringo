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
}

async fn show(Path(id): Path<ShapefileId>) -> impl IntoResponse {
	id
}

pub(crate) fn router(_config: &config::Config) -> Router {
	let db = ShapefileDatabase {
		entries: HashMap::new(),
	};

	let db = Box::leak(Box::new(db));

	Router::new()
		.route("/", get(|| db.index()))
		.route("/:id", get(show(id)))
}
