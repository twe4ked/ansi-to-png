use crate::ansi::{self, Attr, Colors, List};

#[derive(Debug)]
pub enum Token {
    Color((u8, u8, u8)),
    Char(char),
}

#[derive(Debug)]
pub struct Parser {
    pub output: Vec<Token>,
}

impl Parser {
    pub fn chars_count(&self) -> usize {
        self.output
            .iter()
            .filter(|t| match t {
                Token::Char(_) => true,
                _ => false,
            })
            .count()
    }
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

        let indexed_colors = List::from(&Colors::default());

        for attr in ansi::attrs_from_sgr_parameters(&params) {
            if let Some(attr) = attr {
                let color = match attr {
                    Attr::Foreground(foreground) => match foreground {
                        ansi::Color::Indexed(index) => Some(indexed_colors[index].rgb()),
                        ansi::Color::Named(index) => Some(indexed_colors[index].rgb()),
                        ansi::Color::Spec(_) => todo!(),
                    },
                    Attr::Reset => Some((255, 255, 255)),
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
