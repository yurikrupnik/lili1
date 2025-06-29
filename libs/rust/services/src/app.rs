// use crate::handlers::{health_check, not_found};
// use axum::routing::get;
use axum::Router;
use utoipa::OpenApi;
use tokio::signal;

pub async fn shutdown_signal() {
  let ctrl_c = async {
    signal::ctrl_c()
      .await
      .expect("failed to install Ctrl+C handler");
  };

  #[cfg(unix)]
  let terminate = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
      .expect("failed to install signal handler")
      .recv()
      .await;
  };

  #[cfg(not(unix))]
  let terminate = std::future::pending::<()>();

  tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

pub fn app<T, S: 'static + Clone + Send + Sync>(state: S, apis: Router<S>) -> Router
where
  T: OpenApi + 'static,
{
  Router::new()
    // .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", T::openapi()))
    // .merge(Redoc::with_url("/redoc", T::openapi()))
    // .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
    // .merge(Scalar::with_url("/scalar", T::openapi()))
    // .route("/health", get(health_check))
    // .layer(CookieManagerLayer::new())
    .nest("/api", apis)
    .with_state(state)
    // .fallback(not_found)
}
