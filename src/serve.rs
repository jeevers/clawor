/// for `serve` subcommand
use crate::common::{hmac_signature, parse_thumbor_path};
use axum::{
    Router,
    body::Body,
    extract::Path,
    extract::State,
    http::{StatusCode, header},
    response::Response,
    routing::get,
};
use base64::engine::general_purpose::URL_SAFE;
use clap::Args;
use photon_rs::PhotonImage;
use secrecy::SecretString;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::read;
use tracing::{Level, debug, error, info, span};

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
) -> Response {
    info!("THUM ON MY BOR TILL IT RESIZE");

    let computed_hmac = hmac_signature(state.signature_secret.clone(), part.clone(), &URL_SAFE);

    let parsed_url = parse_thumbor_path(format!("{}/{}", hmac, part), false);
    // TODO: check signature
    match parsed_url {
        Ok(url) => {
            if url.hash.unwrap() != computed_hmac {
                error!("hash validation/verification failed");
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(header::SERVER, "axum-clawor")
                    .header(header::CONTENT_TYPE, "text/plain")
                    .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
                    .body(Body::from("invalid url"))
                    .unwrap();
            } else {
                let image_path = state.src_path.join(url.image);
                let image_data = read(&image_path).await;
                match image_data {
                    Err(err) => {
                        debug!(
                            "failed to open '{}': {}",
                            &image_path.to_str().unwrap(),
                            err
                        );
                        return Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .header(header::SERVER, "axum-clawor")
                            .header(header::CONTENT_TYPE, "text/plain")
                            .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
                            .body(Body::from("shrugmoji"))
                            .unwrap();
                    }
                    Ok(file) => {
                        let phimage = PhotonImage::new_from_byteslice(file);
                        return Response::builder()
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
                            .body(Body::from(phimage.get_bytes_jpeg(50))) // get_bytes returns png
                            .unwrap();
                    }
                }
            }
        }
        Err(_err) => {
            error!("regex parsing failed");
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header(header::SERVER, "axum-clawor")
                .header(header::CONTENT_TYPE, "text/plain")
                .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
                .body(Body::from("invalid url"))
                .unwrap();
        }
    }
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
