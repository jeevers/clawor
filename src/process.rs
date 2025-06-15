/// for `process` subcommand
use clap::Args;
use std::path::PathBuf;
//use photon_rs::channels::alter_red_channel;
use crate::common::parse_thumbor_path;
use crate::image_processing::resize_image;
use photon_rs::native::{open_image, save_image};
use photon_rs::transform::compress;
use photon_rs::PhotonImage;

/// opens image, transforms image based on supplied options and then saves the transformed image to
/// disk. output format is inferred from file extension (photon-rs' behavior)
#[derive(Args, Debug)]
pub struct ProcessArgs {
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
    /// thumbor style url to parse for processing operations
    #[arg(short, long, default_value = "")]
    url: String,
    /// supplied url (--url/-u) will be evaluated without hash element in the url path
    #[arg(short, long)]
    no_hash: bool,
}

/// subcommand for `process`
#[tracing::instrument]
pub fn process(args: &ProcessArgs) {
    println!("source: {}", args.src_file.display());
    println!("destination: {}", args.dst_file.display());

    let img = open_image(args.src_file.clone()).expect("File should open");
    let mut output: PhotonImage = img.clone();

    if args.url != "" {
        let _ =
            parse_thumbor_path(args.url.clone(), args.no_hash.clone()).expect("url is parsable");
    } else {
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
}
