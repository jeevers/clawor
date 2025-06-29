/// for `process` subcommand
use clap::Args;
use image::{DynamicImage, ImageFormat, ImageReader};
use std::path::PathBuf;
//use photon_rs::channels::alter_red_channel;
use crate::common::parse_thumbor_path;
use crate::image_processing::resize_image_imagers;
use std::io::Cursor;

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

    let image_file: Vec<u8> =
        std::fs::read(&args.src_file).expect("file should be opened and read");
    let imagebuf = Cursor::new(image_file);

    let img = ImageReader::new(imagebuf)
        .with_guessed_format()
        .expect("File should open");
    let orig_format = img.format();
    println!("orig - format:{:?}", orig_format);
    let mut output: DynamicImage = img.decode().expect("image should be decoded");
    println!("orig - color:{:?}", output.color());

    if args.url != "" {
        let _ =
            parse_thumbor_path(args.url.clone(), args.no_hash.clone()).expect("url is parsable");
    } else {
        match &args.resize {
            Some(dimensions) => {
                let dimstr = dimensions.to_string();
                let dims: Vec<&str> = dimstr.split("x").collect();
                let width: u32 = dims[0].parse().unwrap();
                let height: u32 = dims[1].parse().unwrap();
                output = resize_image_imagers(&output, width, height);
            }
            None => (),
        }
    }
    println!("output - color:{:?}", output.color());
    let dest_format = match ImageFormat::from_path(&args.dst_file) {
        Ok(val) => val,
        Err(err) => {
            println!("error determining destination format from path: {}", err);
            ImageFormat::Jpeg
        }
    };

    match dest_format {
        ImageFormat::Jpeg => output.into_rgb8().save(&args.dst_file).unwrap(),
        _ => output.save(&args.dst_file).unwrap(),
    }
}
