use crate::common::hmac_signature;
use base64::engine::general_purpose::URL_SAFE;
use clap::Args;
use secrecy::SecretString;

/// Generates a HMAC signature from a shared key and the part of the thumbor URL containing
/// transformation directives and image path
#[derive(Args, Debug)]
pub struct PathSigArgs {
    /// sensitive key used to generate HMAC signature; environment variable value will not be
    /// displayed in `--help` output

    // SECRET_KEY env-var, as well as '--secret-key'
    #[arg(long, env, hide_env_values = true, default_value = "secret")]
    secret_key: SecretString,
    /// the part of the thumbor URL that contains the image transformation params and image path;
    /// i.e. `210x138/filters:no_upscale():max_bytes(150000):strip_icc():format(webp)/someimage.jpg`
    #[arg(short, long)]
    part: String,
}

#[tracing::instrument]
pub fn thumbor_path_signature(args: &PathSigArgs) {
    let signature_urlsafe = hmac_signature(args.secret_key.clone(), args.part.clone(), &URL_SAFE);
    println!("{}/{}", signature_urlsafe, args.part);
}
