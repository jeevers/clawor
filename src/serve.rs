/// for `serve` subcommand
use crate::common::hmac_signature;
use axum::{extract::Path, extract::State, routing::get, Router};
use base64::engine::general_purpose::URL_SAFE;
use clap::Args;
use secrecy::SecretString;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};

/// Starts a webserver that serves transformed images based on thumbor's url path scheme
#[derive(Args)]
pub struct ServeArgs {
    /// listen address in form of `<ip>:<port>`; SSL is not supported currently
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    address: String,
    /// fs path that contains the 'raw' images
    #[arg(short, long, default_value = "./src_images")]
    src_path: PathBuf,
    /// fs path used to store processed images
    #[arg(short, long, default_value = "./cache")]
    cache_path: PathBuf,
    /// sensitive key used to generate HMAC signature
    // environment variable value will not be displayed in `--help` output
    // SECRET_KEY env-var
    #[arg(long, env, hide_env_values = true, default_value = "secret")]
    secret_key: SecretString,
}

/// defines AppState that is passed to handlers of `serve`
#[derive(Debug)]
struct AppState {
    src_path: PathBuf,
    cache_path: PathBuf,
    signature_secret: SecretString,
}

/// simple health check
#[tracing::instrument]
async fn health_handler(State(state): State<Arc<AppState>>) -> String {
    info!("HEALTH CHECK");
    format!(
        "stuff is responding, src: {}, cache: {}",
        state.src_path.display(),
        state.cache_path.display()
    )
}

/// IN PROGRESS; will eventually respond with processed image
#[tracing::instrument]
async fn pic_handler(
    Path((hmac, part)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> String {
    info!("THUMBOR");
    // TODO: check signature
    let computed_hmac = hmac_signature(state.signature_secret.clone(), part.clone(), &URL_SAFE);

    // check url after signature in cache and file (last item in path) in src
    format!(
        "hmac: {}, url part: {}, src: {}, cache: {}\n computed hmac: {}",
        hmac,
        part,
        state.src_path.display(),
        state.cache_path.display(),
        computed_hmac,
    )
}

/// starts axum webserver
#[tokio::main]
pub async fn serve_app(args: &ServeArgs) {
    debug!("starting web-server");
    let shared_state = Arc::new(AppState {
        src_path: args.src_path.clone(),
        cache_path: args.cache_path.clone(),
        signature_secret: args.secret_key.clone(),
    });

    debug!(
        "address: {}, src: {}, cache: {}",
        args.address,
        args.src_path.display(),
        args.cache_path.display()
    );
    let app = Router::new()
        .route("/", get(|| async { "hello world!" }))
        .route("/health", get(health_handler))
        .with_state(shared_state.clone())
        .route("/thmb/{hmac}/{*part}", get(pic_handler))
        .with_state(shared_state);
    let listener = tokio::net::TcpListener::bind(args.address.clone())
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
