mod ansi;
mod parser;
mod renderer;

use clap::Clap;
use std::io;

#[derive(Clap)]
struct Opts {
    /// Filename to save PNG
    #[clap(short, long, default_value = "out.png")]
    out: String,
    /// Font to use (should be a monospaced font)
    #[clap(short, long)]
    font: String,
}

fn main() {
    let opts: Opts = Opts::parse();

    let tokens = parser::parse(io::stdin());

    renderer::render(&tokens, &opts.font, &opts.out);

    println!("Generated: {}", opts.out);
}
