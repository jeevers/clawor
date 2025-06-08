mod common;
mod image_processing;
mod path_signature;
mod process;
mod serve;

use crate::path_signature::{thumbor_path_signature, PathSigArgs};
use crate::process::{process, ProcessArgs};
use crate::serve::{serve_app, ServeArgs};
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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

fn main() {
    // log level can be controlled with RUST_LOG env-var
    let filter = EnvFilter::from_default_env();
    let filtered_layer = fmt::layer().with_filter(filter);
    tracing_subscriber::registry().with(filtered_layer).init();

    info!("starting CLI");
    let cli = Cli::parse();
    match &cli.command {
        Commands::Process(args) => process(args),
        Commands::Serve(args) => serve_app(args),
        Commands::PathSignature(args) => thumbor_path_signature(args),
    }
}
