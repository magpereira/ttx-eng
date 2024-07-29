use clap::Parser;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::stdout;
use tracing::Level;

use ttx_eng::cli;

fn main() -> Result<(), Box<dyn Error>> {
    //setup tracing subscriber that will output to stderr
    let collector = tracing_subscriber::fmt()
        .with_max_level(Level::ERROR)
        .with_writer(io::stderr)
        .finish();
    tracing::subscriber::set_global_default(collector)
        .expect("failed to set tracing default subscriber");

    //parse cli args
    let args = cli::Cli::parse();
    let input_file = File::open(&args.file_path)?;

    cli::process_input(input_file, stdout())
}
