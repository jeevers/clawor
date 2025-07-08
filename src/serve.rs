/// for `serve` subcommand
use crate::common::{hmac_signature, parse_thumbor_path};
use anyhow::{anyhow, bail};
use axum::{
    Router,
    body::Body,
    extract::Path,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use base64::engine::general_purpose::URL_SAFE;
use clap::Args;
use core::error;
use image::{DynamicImage, ImageFormat, ImageReader};
use photon_rs::PhotonImage;
use secrecy::SecretString;
use std::sync::Arc;
use std::{error::Error, io::Cursor, path::PathBuf};
use tokio::fs::read;
use tracing::{debug, error, info};

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

#[derive(Debug)]
struct UrlError {
    message: String,
}

#[derive(Debug)]
struct ImageError {
    image: String,
    message: String,
}

// the error ahndling for pic_handler was getting.... unweildy
// adapted axum's example here to simply that:
// https://github.com/tokio-rs/axum/blob/main/examples/error-handling/src/main.rs
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::NOT_FOUND,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
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
) -> Result<Response, AppError> {
    info!("THUM ON MY BOR TILL IT RESIZE");

    let computed_hmac = hmac_signature(state.signature_secret.clone(), part.clone(), &URL_SAFE);

    let parsed_url = parse_thumbor_path(format!("{}/{}", hmac, part), false)?;
    if parsed_url.hash.unwrap_or("NOHASH".to_string()) != computed_hmac {
        error!("hash validation/verification failed");
        return Err(anyhow!("hash validation/verification failed").into());
    }
    let image_path = state.src_path.join(parsed_url.image);
    let image_data = read(&image_path).await?;
    let imgbuf = Cursor::new(image_data);
    let img = ImageReader::new(imgbuf).with_guessed_format()?.decode()?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::SERVER, "axum-clawor")
        .header("x-clawor-hmac", &hmac)
        .header("x-clawor-computed-hmac", &computed_hmac)
        .header("x-clawor-part", &part)
        .header(
            "x-clawor-src-path",
            state.src_path.clone().to_str().unwrap(),
        )
        .header(
            "x-clawor-cache-path",
            state.cache_path.clone().to_str().unwrap(),
        )
        .body(Body::from(img.into_bytes()))?)
}

/// starts axum webserver
#[tokio::main]
pub async fn serve_app(args: &ServeArgs) {
    debug!("creating directories if they don't exist");
    match args.src_path.try_exists() {
        Ok(true) => debug!("src images directory exists"),
        Ok(false) => panic!(
            "src image directory '{}' does not exist",
            args.src_path.to_str().unwrap()
        ),
        Err(err) => panic!("error checking src directory: {}", err),
    }
    match args.cache_path.try_exists() {
        Ok(true) => debug!(
            "src images directory '{}' exists",
            args.cache_path.to_str().unwrap()
        ),
        Ok(false) => {
            std::fs::create_dir(args.cache_path.clone())
                .expect("cache directory should be created");
            debug!(
                "created cache directory '{}'",
                args.cache_path.to_str().unwrap()
            );
        }
        Err(err) => panic!("error checking cache directory: {}", err),
    }
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
