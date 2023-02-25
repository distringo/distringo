use axum::{response::IntoResponse, routing::*};
use http::{StatusCode, Uri};
use tower_http::{services::ServeDir, trace::TraceLayer};

use crate::RuntimeError;

mod shapefiles;

#[tracing::instrument]
async fn ping() -> Result<String, StatusCode> {
	Ok("pong".to_string())
}

#[tracing::instrument]
async fn router_fallback(uri: Uri) -> impl IntoResponse {
	(StatusCode::NOT_FOUND, format!("uri {uri} not found"))
}

/// Constructs a Service that can serve as a fallback.
fn static_files_handler() -> MethodRouter {
	let serve_dir = ServeDir::new("dist");

	get_service(serve_dir)
}

async fn handle_error(error: tower::BoxError) -> Result<impl IntoResponse, impl IntoResponse> {
	if error.is::<tower::timeout::error::Elapsed>() {
		Ok(StatusCode::REQUEST_TIMEOUT)
	} else {
		Err((
			StatusCode::INTERNAL_SERVER_ERROR,
			format!("Unhandled internal error: {error}"),
		))
	}
}

pub(super) fn app_router(
	config: &crate::settings::AppConfig,
) -> Result<axum::Router, RuntimeError> {
	use axum::error_handling::HandleErrorLayer;

	let error_handler = tower::ServiceBuilder::new()
		.layer(HandleErrorLayer::new(handle_error))
		.timeout(core::time::Duration::from_secs(10))
		.into_inner();

	let static_files_fallback = static_files_handler();

	let router = axum::Router::new()
		.route("/ping", get(ping))
		.nest("/shapefiles", shapefiles::router(config.shapefiles()))
		.layer(TraceLayer::new_for_http())
		.layer(error_handler)
		.fallback_service(static_files_fallback);

	Ok(router)
}
