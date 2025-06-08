/// for common utility functions used by multiple subcommands
use base64::display::Base64Display;
use hmac::{Hmac, Mac};
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
    let operations = path.split("/");
    for op in operations {
        println!("{}", op);
    }
}
