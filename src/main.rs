use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
//use photon_rs::channels::alter_red_channel;
use axum::{Router, extract::Path, extract::State, routing::get};
use base64::{
    display::Base64Display, engine::general_purpose::STANDARD, engine::general_purpose::URL_SAFE,
};
use hmac::{Hmac, Mac};
use photon_rs::PhotonImage;
use photon_rs::native::{open_image, save_image};
use photon_rs::transform::{SamplingFilter, compress, resize};
use sha1::Sha1;
use std::sync::Arc;

/// defines root commands
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

//defines subcommands
#[derive(Subcommand)]
enum Commands {
    Process(ProcessArgs),
    Serve(ServeArgs),
    PathSignature(PathSigArgs),
}

/// defines args/options for `process` subcommand
#[derive(Args)]
struct ProcessArgs {
    #[arg(short, long, default_value = "./input.jpg")]
    src_file: PathBuf,
    #[arg(short, long, default_value = "./output.jpg")]
    dst_file: PathBuf,
    /// resize dimensions in form of `<width>x<height>`
    #[arg(short, long)]
    resize: Option<String>,
    /// compression ratio (1-100?)
    #[arg(short, long)]
    compress: Option<u8>,
}

/// resizes image based on specified `<w>x<h>` string, if either `w` or `h` are 0, that
/// dimension will be computed based on the ratio of defined dimension, might not be exact due
/// to floating point rounding
fn resize_image(mut img: &PhotonImage, size: String) -> PhotonImage {
    let dims: Vec<&str> = size.split("x").collect();
    let w_orig: f32 = img.get_width() as f32;
    let h_orig: f32 = img.get_height() as f32;
    let mut width: f32 = dims[0].parse().unwrap();
    let mut height: f32 = dims[1].parse().unwrap();

    if width == 0.0 {
        width = w_orig * (height / h_orig)
    }
    if height == 0.0 {
        height = h_orig * (width / w_orig)
    }
    println!(
        "width: {}, height: {} (orig {}x{})",
        width.round(),
        height.round(),
        w_orig,
        h_orig
    );
    resize(
        &mut img,
        width.round() as u32,
        height.round() as u32,
        SamplingFilter::Nearest,
    )
}

/// subcommand for `process`
/// opens image, transforms image based on operations and then saves the transformed image to
/// disk. output format is inferred from file extension (photon-rs' behavior)
fn process(args: &ProcessArgs) {
    println!("source: {}", args.src_file.display());
    println!("destination: {}", args.dst_file.display());

    let img = open_image(args.src_file.clone()).expect("File should open");
    let mut output: PhotonImage = img.clone();
    //alter_red_channel(&mut img, 25_i16);
    match &args.resize {
        Some(dimensions) => output = resize_image(&mut output, dimensions.to_string()),
        None => (),
    }
    match &args.compress {
        Some(quality) => output = compress(&mut output, *quality),
        None => (),
    }
    save_image(output, args.dst_file.clone()).expect("output file should be written");
}

/// defines CLI args/options for `serve` subcommand
#[derive(Args)]
struct ServeArgs {
    /// listen address in form of `<ip>:<port>`; SSL is not supported currently
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    address: String,
    /// fs path that contains the 'raw' images
    #[arg(short, long, default_value = "./src_images")]
    src_path: PathBuf,
    /// fs path used to store processed images
    #[arg(short, long, default_value = "./cache")]
    cache_path: PathBuf,
    /// sensitive key used to generate HMAC signature; environment variable value will not be
    /// displayed in `--help` output
    // SECRET_KEY env-var
    #[arg(long, env, hide_env_values = true, default_value = "secret")]
    secret_key: String,
}

/// defines AppState that is passed to handlers of `serve`
struct AppState {
    src_path: PathBuf,
    cache_path: PathBuf,
    signature_secret: String,
}

/// simple health check
async fn health_handler(State(state): State<Arc<AppState>>) -> String {
    format!(
        "stuff is responding, src: {}, cache: {}",
        state.src_path.display(),
        state.cache_path.display()
    )
}

/// IN PROGRESS; will eventually respond with processed image
async fn pic_handler(
    Path((hmac, part)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> String {
    // TODO: check signature
    let computed_hmac = hmac_signature(state.signature_secret.clone(), part.clone(), &URL_SAFE);

    // check url after signature in cache and file (last item in path) in src
    format!(
        "hmac: {}, url part: {}, src: {}, cache: {}\n computed hmac: {}",
        //"hmac: {}, url part: {}, src: {}, cache: {}",
        hmac,
        part,
        state.src_path.display(),
        state.cache_path.display(),
        computed_hmac,
    )
}

/// starts axum webserver
#[tokio::main]
async fn serve(args: &ServeArgs) {
    let shared_state = Arc::new(AppState {
        src_path: args.src_path.clone(),
        cache_path: args.cache_path.clone(),
        signature_secret: args.secret_key.clone(),
    });

    println!(
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

/// defines args/options for `path-signature`
#[derive(Args)]
struct PathSigArgs {
    /// sensitive key used to generate HMAC signature; environment variable value will not be
    /// displayed in `--help` output

    // SECRET_KEY env-var, as well as '--secret-key'
    #[arg(long, env, hide_env_values = true, default_value = "secret")]
    secret_key: String,
    /// the part of the thumbor URL that contains the image transformation params and image path;
    /// i.e. `210x138/filters:no_upscale():max_bytes(150000):strip_icc():format(webp)/someimage.jpg`
    #[arg(short, long)]
    part: String,
}

/// generates HMAC signature
fn hmac_signature(key: String, msg: String, alphabet: &base64::engine::GeneralPurpose) -> String {
    /// from: https://stackoverflow.com/questions/71632964/rust-equivalent-of-hmac-digest-in-python/71633139#71633139
    /// generates an HMAC signature from key and msg (the image part of the thumbor URL)
    /// alphabet is the charset that base64 uses to output the signature, currently can be
    /// &STANDARD (standard base64 charset) or &URL_SAFE (charset that allows the signature to be
    /// used in a URL path)
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(key.as_bytes()).unwrap();
    mac.update(&msg.as_bytes());
    let code_bytes = mac.finalize().into_bytes();

    return Base64Display::new(&code_bytes.to_vec(), alphabet).to_string();
}

/// subcommand for `path-signature`
fn thumbor_path_signature(args: &PathSigArgs) {
    let signature_urlsafe = hmac_signature(args.secret_key.clone(), args.part.clone(), &URL_SAFE);
    let signature = hmac_signature(args.secret_key.clone(), args.part.clone(), &STANDARD);
    println!(
        "signature (urlsafe):     {}\nsignature (non-urlsafe): {}",
        signature_urlsafe, signature
    );
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Process(args) => process(args),
        Commands::Serve(args) => serve(args),
        Commands::PathSignature(args) => thumbor_path_signature(args),
    }
}
