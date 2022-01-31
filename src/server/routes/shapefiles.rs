use axum::{routing::get, Json, Router};

#[derive(serde::Serialize)]
struct ShapefileList {
	shapefiles: Vec<String>,
}

async fn index() -> Json<ShapefileList> {
	// TODO(rye): Get this ShapefileList out of some kind of cache (and generate that on startup)
	Json(ShapefileList {
		shapefiles: vec!["a".to_string(), "b".to_string()],
	})
}

async fn show() -> ShapefileList {
	todo!()
}

pub(crate) fn router(config: &config::Config) -> Router {
	Router::new().route("/", get(index))
}
