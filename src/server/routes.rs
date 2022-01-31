use axum::{error_handling::HandleErrorLayer, routing::get, BoxError};
use http::StatusCode;
use tower_http::trace::TraceLayer;

use crate::RuntimeError;

#[tracing::instrument]
async fn ping_handler() -> Result<String, StatusCode> {
	Ok("pong".to_string())
}

pub(super) fn app_router(config: &config::Config) -> Result<axum::Router, RuntimeError> {
	let error_handler = tower::ServiceBuilder::new()
		.layer(HandleErrorLayer::new(|error: BoxError| async move {
			if error.is::<tower::timeout::error::Elapsed>() {
				Ok(StatusCode::REQUEST_TIMEOUT)
			} else {
				Err((
					StatusCode::INTERNAL_SERVER_ERROR,
					format!("Unhandled internal error: {}", error),
				))
			}
		}))
		.timeout(core::time::Duration::from_secs(10))
		.layer(TraceLayer::new_for_http())
		.into_inner();

	let router = axum::Router::new()
		.route("/ping", get(ping_handler))
		.layer(error_handler);

	Ok(router)
}
