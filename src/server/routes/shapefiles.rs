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

async fn index() -> impl IntoResponse {
	// TODO(rye): Get this ShapefileList out of some kind of cache (and generate that on startup)
	Json(ShapefileList {
		shapefiles: vec!["a".to_string(), "b".to_string()],
	})
}

async fn show(Path(id): Path<ShapefileId>) -> impl IntoResponse {
	id
}

pub(crate) fn router(_config: &config::Config) -> Router {
	Router::new()
		.route("/", get(index))
		.route("/:id", get(show(id)))
}
