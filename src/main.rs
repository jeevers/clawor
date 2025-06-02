use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
//use photon_rs::channels::alter_red_channel;
use axum::{routing::get, Router};
use photon_rs::native::{open_image, save_image};
use photon_rs::transform::{compress, resize, SamplingFilter};
use photon_rs::PhotonImage;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Process(ProcessArgs),
    Serve(ServeArgs),
}

#[derive(Args)]
struct ProcessArgs {
    #[arg(short, long, default_value = "./input.jpg")]
    src_file: PathBuf,
    #[arg(short, long, default_value = "./output.jpg")]
    dst_file: PathBuf,
    #[arg(short, long)]
    resize: Option<String>,
    #[arg(short, long)]
    compress: Option<u8>,
}

#[derive(Args)]
struct ServeArgs {
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    address: String,
    #[arg(short, long, default_value = "./src_images")]
    src_path: PathBuf,
    #[arg(short, long, default_value = "./cache")]
    cache_path: PathBuf,
}

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

#[tokio::main]
async fn serve(args: &ServeArgs) {
    println!(
        "address: {}, src: {}, cache: {}",
        args.address,
        args.src_path.display(),
        args.cache_path.display()
    );
    let app = Router::new().route("/", get(|| async { "hello world!" }));
    let listener = tokio::net::TcpListener::bind(args.address.clone())
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Process(args) => process(args),
        Commands::Serve(args) => serve(args),
    }
}
