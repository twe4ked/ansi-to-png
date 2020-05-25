use crate::ansi::{attrs_from_sgr_parameters, Attr, Color, Colors, List, PrimaryColors, Rgb};
use std::io::Read;

pub fn parse(input: std::io::Stdin) -> Vec<Token> {
    let mut handle = input.lock();

    let foreground = PrimaryColors::default().foreground;

    let mut statemachine = vte::Parser::new();
    let mut parser = Parser {
        output: vec![Token::Color(foreground)],
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

    parser.output
}

#[derive(Debug)]
pub enum Token {
    Color(Rgb),
    Char(char),
}

#[derive(Debug)]
pub struct Parser {
    pub output: Vec<Token>,
}

impl vte::Perform for Parser {
    fn print(&mut self, c: char) {
        self.output.push(Token::Char(c));
    }

    fn execute(&mut self, _byte: u8) {}
    fn hook(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool, _c: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &[i64], _intermediates: &[u8], _ignore: bool, c: char) {
        if c != 'm' {
            // Not a color "CSI"
            return;
        }

        let foreground = PrimaryColors::default().foreground;
        let indexed_colors = List::from(&Colors::default());

        for attr in attrs_from_sgr_parameters(&params) {
            if let Some(attr) = attr {
                let color = match attr {
                    Attr::Foreground(foreground) => match foreground {
                        Color::Indexed(index) => Some(indexed_colors[index]),
                        Color::Named(index) => Some(indexed_colors[index]),
                        Color::Spec(_) => todo!(),
                    },
                    Attr::Reset => Some(foreground),
                    Attr::Bold
                    | Attr::Dim
                    | Attr::Italic
                    | Attr::Underline
                    | Attr::BlinkSlow
                    | Attr::BlinkFast
                    | Attr::Reverse
                    | Attr::Hidden
                    | Attr::Strike
                    | Attr::CancelBold
                    | Attr::CancelBoldDim
                    | Attr::CancelItalic
                    | Attr::CancelUnderline
                    | Attr::CancelBlink
                    | Attr::CancelReverse
                    | Attr::CancelHidden
                    | Attr::CancelStrike
                    | Attr::Background { .. } => None,
                };
                if let Some(c) = color {
                    self.output.push(Token::Color(c));
                }
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}
