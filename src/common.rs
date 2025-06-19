/// for common utility functions used by multiple subcommands
use base64::display::Base64Display;
use hmac::{Hmac, Mac};
use regex::Regex;
use secrecy::{ExposeSecret, SecretString};
use sha1::Sha1;

/// generates HMAC signature
#[tracing::instrument]
pub fn hmac_signature(
    key: SecretString,
    msg: String,
    alphabet: &base64::engine::GeneralPurpose,
) -> String {
    /// from: https://stackoverflow.com/questions/71632964/rust-equivalent-of-hmac-digest-in-python/71633139#71633139
    /// generates an HMAC signature from key and msg (the image part of the thumbor URL)
    /// alphabet is the charset that base64 uses to output the signature, currently can be
    /// &STANDARD (standard base64 charset) or &URL_SAFE (charset that allows the signature to be
    /// used in a URL path)
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(key.expose_secret().as_bytes()).unwrap();
    mac.update(&msg.as_bytes());
    let code_bytes = mac.finalize().into_bytes();

    return Base64Display::new(&code_bytes.to_vec(), alphabet).to_string();
}

// struct defining operations passed as thumbor url
#[derive(Default, Debug)]
pub struct ThumborUrlParams {
    // hash is either a string or None
    pub unsafe_hash: bool,
    pub hash: Option<String>,
    /*debug: bool,
    meta: bool,
    trim: String,
    crop_left: u32,
    crop_top: u32,
    crop_right: u32,
    crop_bottom: u32,
    adaptive: String,
    full: String,
    fit_in: String,
    horizontal_flip: bool,
    vertical_flip: bool,*/
    pub width: u32,
    pub height: u32,
    /*smart: bool,
    filters: Vec<String>,*/
    pub image: String,
}

/// parse thumbor style url (without hmac signature)
pub fn parse_thumbor_path(path: String, no_hash: bool) -> Result<ThumborUrlParams, &'static str> {
    // adapted from libthumbor:
    // https://github.com/thumbor/libthumbor/blob/master/libthumbor/url.py
    let mut thumbor_req: ThumborUrlParams = Default::default();

    let regex_patt_parts = [
        r"/?",
        r"(?:(?:(?P<unsafe>unsafe)|(?P<hash>.+?))/)?", // unsafe_or_hash
        r"(?:(?P<debug>debug)/)?",                     // debug
        r"(?:(?P<meta>meta)/)?",                       // meta
        r"(?:(?P<trim>trim(?::(?:top-left|bottom-right))?(?::\d+)?)/)?", // trim
        r"(?:(?P<crop_left>\d+)x(?P<crop_top>\d+):(?P<crop_right>\d+)x(?P<crop_bottom>\d+)/)?", //crop
        r"(?:(?P<adaptive>adaptive-)?(?P<full>full-)?(?P<fit_in>fit-in)/)?", // fit_in
        r"(?:(?P<horizontal_flip>-)?(?P<width>(?:\d+|orig))?x(?P<vertical_flip>-)?(?P<height>(?:\d+|orig))?/)?", // dimensions
        r"(?:(?P<halign>left|right|center)/)?", // halign
        r"(?:(?P<valign>top|bottom|middle)/)?", // valign
        r"(?:(?P<smart>smart)/)?",              // smart
        r"(?:filters:(?P<filters>.+?\))/)?",    // filters
        r"(?P<image>.+)",                       // image
    ];
    let regex_patt_parts_nohash = [
        r"/?",
        r"(?:(?P<debug>debug)/)?", // debug
        r"(?:(?P<meta>meta)/)?",   // meta
        r"(?:(?P<trim>trim(?::(?:top-left|bottom-right))?(?::\d+)?)/)?", // trim
        r"(?:(?P<crop_left>\d+)x(?P<crop_top>\d+):(?P<crop_right>\d+)x(?P<crop_bottom>\d+)/)?", //crop
        r"(?:(?P<adaptive>adaptive-)?(?P<full>full-)?(?P<fit_in>fit-in)/)?", // fit_in
        r"(?:(?P<horizontal_flip>-)?(?P<width>(?:\d+|orig))?x(?P<vertical_flip>-)?(?P<height>(?:\d+|orig))?/)?", // dimensions
        r"(?:(?P<halign>left|right|center)/)?", // halign
        r"(?:(?P<valign>top|bottom|middle)/)?", // valign
        r"(?:(?P<smart>smart)/)?",              // smart
        r"(?:filters:(?P<filters>.+?\))/)?",    // filters
        r"(?P<image>.+)",                       // image
    ];

    let full_regex = match &no_hash {
        false => regex_patt_parts.join(""),
        true => regex_patt_parts_nohash.join(""),
    };
    let re = Regex::new(&full_regex).unwrap();

    let Some(caps) = re.captures(&path) else {
        return Err("regex failed");
    };

    println!("{:?}", &caps);

    if no_hash {
        thumbor_req.hash = None;
    } else {
        match &caps.name("unsafe") {
            Some(_value) => {
                println!("'unsafe' hash");
                thumbor_req.unsafe_hash = true;
            }
            None => {
                thumbor_req.unsafe_hash = false;
            }
        }
        match &caps.name("hash") {
            Some(value) => {
                println!("hash: {}", value.as_str());
                thumbor_req.hash = Some(value.as_str().to_string());
            }
            None => {}
        }
    }

    match &caps.name("image") {
        Some(value) => {
            println!("image: {}", value.as_str());
            thumbor_req.image = value.as_str().to_string();
        }
        None => {}
    }

    match &caps.name("width") {
        Some(value) => {
            println!("width: {}", value.as_str());
            thumbor_req.width = value.as_str().parse::<u32>().unwrap();
        }
        None => {}
    }
    match &caps.name("height") {
        Some(value) => {
            println!("height: {}", value.as_str());
            thumbor_req.height = value.as_str().parse::<u32>().unwrap();
        }
        None => {}
    }

    println!("{:?}", thumbor_req);

    Ok(thumbor_req)

    /*tracing::info!("hash:{}", &caps["hash"]);
    tracing::info!("resize:{}x{}", &caps["width"], &caps["height"]);
    tracing::info!(
        "hflip:{}, vflip:{}",
        &caps["horizontal_flip"],
        &caps["vertical_flip"]
    );*/
    /*tracing::info!(
        "meta:{}, debug:{}, smart:{}, adaptive:{}",
        &caps["meta"],
        &caps["debug"],
        &caps["smart"],
        &caps["adaptive"]
    );
    tracing::info!(
        "trim:{}, crop:{}x{}:{}x{}",
        &caps["trim"],
        &caps["crop_left"],
        &caps["crop_top"],
        &caps["crop_right"],
        &caps["crop_bottom"]
    );*/
}
