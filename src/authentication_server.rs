use anyhow::{Context, Result};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, routing::get, Router};
use std::collections::HashMap;
use tokio::sync::mpsc::{channel, Sender};
use tracing::info;

/// State for the axum webserver that allows the handler to send a code back
/// to the main thread and shutdown the webserver.
#[derive(Debug, Clone)]
struct AppState {
    code_sender: Sender<String>,
    shutdown_sender: Sender<()>,
}

/// Starts a webserver on port 7007 to listen for an authentication callback.
/// Returns the received authentication code when the callback is called.
pub async fn get_code() -> Result<String> {
    let port = 7007;
    let (code_sender, mut code_receiver) = channel::<String>(1);
    let (shutdown_sender, mut shutdown_reciever) = channel::<()>(1);
    let state = AppState {
        code_sender,
        shutdown_sender,
    };
    info!(
        "Starting webserver to listen for authentication callback on port {}",
        port
    );
    let addr = ([127, 0, 0, 1], port).into();
    let router = Router::new()
        .route("/callback", get(handler))
        .with_state(state);
    let server = axum::Server::bind(&addr).serve(router.into_make_service());
    server
        .with_graceful_shutdown(async {
            shutdown_reciever.recv().await;
        })
        .await
        .context("Error running webserver")?;
    code_receiver
        .recv()
        .await
        .context("Error receiving auth code from webserver")
}

/// Handles the `/callback` route for the webserver.
/// It extracts the authentication code from the query string and sends it to the main thread.
/// After that, it sends a shutdown signal to the webserver.
async fn handler(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> axum::response::Result<&'static str, AppError> {
    let code = params.get("code").context("No code in query string")?;
    state
        .code_sender
        .send(code.to_string())
        .await
        .context("Error sending code to main thread")?;
    state
        .shutdown_sender
        .send(())
        .await
        .context("Error sending shutdown signal to webserver")?;
    Ok("Authentication successful! You can close this window now.")
}

/// helper type to convert `anyhow::Error`s into responses
struct AppError(anyhow::Error);

/// Implements `IntoResponse` for `AppError`, converting it into a response with status code 500.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

/// Implements the `From` trait for `AppError`, allowing it to be converted
/// from any type implementing `Into<anyhow::Error>`.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}
