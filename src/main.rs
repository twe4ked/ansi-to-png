mod ansi;
mod parser;
mod renderer;

use ansi::Rgb;
use parser::{Parser, Token};

use std::io::{self, Read};

use clap::Clap;

#[derive(Clap)]
struct Opts {
    /// Filename to save PNG
    #[clap(short, long, default_value = "out.png")]
    out: String,
    /// Font to use (should be a monospaced font)
    #[clap(short, long)]
    font: String,
}

const WHITE: Rgb = Rgb {
    r: 255,
    g: 255,
    b: 255,
};

fn main() {
    let opts: Opts = Opts::parse();

    let input = io::stdin();
    let mut handle = input.lock();

    let mut statemachine = vte::Parser::new();
    let mut parser = Parser {
        output: vec![Token::Color(WHITE)],
    };

    let mut buf = [0; 2048];

    loop {
        match handle.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                for byte in &buf[..n] {
                    statemachine.advance(&mut parser, *byte);
                }
            }
            Err(err) => {
                println!("err: {}", err);
                break;
            }
        }
    }

    renderer::render(&parser.output, parser.chars_count(), &opts.font, &opts.out);

    println!("Generated: {}", opts.out);
}
