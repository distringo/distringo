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

	let db = Box::leak(Box::new(db));

	Router::new()
		.route("/", get(|| db.index()))
		.route("/:id", get(|id| db.show(id)))
}
