use vte;

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Aqua,
    Black,
    Blue,
    Fuchsia,
    Green,
    Grey,
    Lime,
    Maroon,
    Navy,
    Olive,
    Purple,
    Red,
    Silver,
    Teal,
    White,
    Yellow,
}

impl Color {
    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::Black => (0, 0, 0),
            Color::Maroon => (128, 0, 0),
            Color::Green => (0, 128, 0),
            Color::Olive => (128, 128, 0),
            Color::Navy => (0, 0, 128),
            Color::Purple => (128, 0, 128),
            Color::Teal => (0, 128, 128),
            Color::Silver => (192, 192, 192),
            Color::Grey => (128, 128, 128),
            Color::Red => (255, 0, 0),
            Color::Lime => (0, 255, 0),
            Color::Yellow => (255, 255, 0),
            Color::Blue => (0, 0, 255),
            Color::Fuchsia => (255, 0, 255),
            Color::Aqua => (0, 255, 255),
            Color::White => (255, 255, 255),
        }
    }
}

#[derive(Debug)]
pub enum Token {
    Color(Color),
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
        use crate::ansi::{self, Attr};

        if c == 'm' {
            for attr in crate::ansi::attrs_from_sgr_parameters(&params) {
                if let Some(attr) = attr {
                    let color = match attr {
                        Attr::Foreground(foreground) => match foreground {
                            ansi::Color::Indexed(index) => Some(match index {
                                // Xterm system colors.
                                0 => Color::Black,
                                1 => Color::Maroon,
                                2 => Color::Green,
                                3 => Color::Olive,
                                4 => Color::Navy,
                                5 => Color::Purple,
                                6 => Color::Teal,
                                7 => Color::Silver,
                                8 => Color::Grey,
                                9 => Color::Red,
                                10 => Color::Lime,
                                11 => Color::Yellow,
                                12 => Color::Blue,
                                13 => Color::Fuchsia,
                                14 => Color::Aqua,
                                15 => Color::White,
                                _ => todo!(),
                            }),
                            ansi::Color::Named(_) => todo!(),
                            ansi::Color::Spec(_) => todo!(),
                        },
                        Attr::Reset
                        | Attr::Bold
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
        } else {
            // Not a color "CSI"
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}
