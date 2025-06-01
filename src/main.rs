use clap::Parser;
use std::path::PathBuf;
//extern crate photon_rs;

use photon_rs::channels::alter_red_channel;
use photon_rs::native::{open_image, save_image};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./input.jpg")]
    src_file: PathBuf,
    #[arg(short, long, default_value = "./output.jpg")]
    dst_file: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("source: {}", args.src_file.display());
    println!("destination: {}", args.dst_file.display());

    let mut img = open_image(args.src_file).expect("File should open");
    alter_red_channel(&mut img, 25_i16);
    save_image(img, args.dst_file).expect("output file should be written")
}
