use clap::Parser;

mod srr;
use crate::srr::SrrFile;

/// Simple util program for srr files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    file: String,
}

fn main() {
    let args = Args::parse();

    let srr_file = SrrFile::from_file(&args.file);
}
