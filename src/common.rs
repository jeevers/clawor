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

/// parse thumbor style url (without hmac signature)
pub fn parse_thumbor_path(path: String) {
    // adapted from libthumbor:
    // https://github.com/thumbor/libthumbor/blob/master/libthumbor/url.py
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

    let full_regex = regex_patt_parts.join("");
    let re = Regex::new(&full_regex).unwrap();

    let Some(caps) = re.captures(&path) else { return };

    println!("{:?}", caps);
}
