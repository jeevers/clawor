use image::{DynamicImage, imageops::FilterType};
use photon_rs::PhotonImage;
/// collects all image processing/transformation functions
//use photon_rs::channels::alter_red_channel;
use photon_rs::transform::{SamplingFilter, resize};
use tracing::debug;

/// resizes image based on specified `<w>x<h>` string, if either `w` or `h` are 0, that
/// dimension will be computed based on the ratio of defined dimension, might not be exact due
/// to floating point rounding
#[tracing::instrument]
pub fn resize_image(mut img: &PhotonImage, size: String) -> PhotonImage {
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

#[tracing::instrument]
pub fn resize_image_imagers(img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    let w_orig: f32 = img.width() as f32;
    let h_orig: f32 = img.height() as f32;
    let mut width_f32: f32 = width as f32;
    let mut height_f32: f32 = height as f32;

    if width_f32 == 0.0 {
        width_f32 = w_orig * (height_f32 / h_orig)
    }
    if height_f32 == 0.0 {
        height_f32 = h_orig * (width_f32 / w_orig)
    }
    debug!(
        "width: {}, height: {} (orig {}x{})",
        width_f32.round(),
        height_f32.round(),
        w_orig,
        h_orig
    );
    img.resize(
        width_f32.round() as u32,
        height_f32.round() as u32,
        FilterType::Nearest,
    )
}
