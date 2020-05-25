// Copyright 2016 Joe Wilm, The Alacritty Project Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;
use std::ops::{Index, IndexMut, Mul};
use std::str::FromStr;

const COUNT: usize = 269;

/// Factor for automatic computation of dim colors used by terminal.
const DIM_FACTOR: f32 = 0.66;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

// A multiply function for Rgb, as the default dim is just *2/3.
impl Mul<f32> for Rgb {
    type Output = Rgb;

    fn mul(self, rhs: f32) -> Rgb {
        Rgb {
            r: (f32::from(self.r) * rhs).max(0.0).min(255.0) as u8,
            g: (f32::from(self.g) * rhs).max(0.0).min(255.0) as u8,
            b: (f32::from(self.b) * rhs).max(0.0).min(255.0) as u8,
        }
    }
}

impl FromStr for Rgb {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Rgb, ()> {
        let chars = if s.starts_with("0x") && s.len() == 8 {
            &s[2..]
        } else if s.starts_with('#') && s.len() == 7 {
            &s[1..]
        } else {
            return Err(());
        };

        match u32::from_str_radix(chars, 16) {
            Ok(mut color) => {
                let b = (color & 0xff) as u8;
                color >>= 8;
                let g = (color & 0xff) as u8;
                color >>= 8;
                let r = color as u8;
                Ok(Rgb { r, g, b })
            }
            Err(_) => Err(()),
        }
    }
}

/// List of indexed colors.
///
/// The first 16 entries are the standard ansi named colors. Items 16..232 are
/// the color cube.  Items 233..256 are the grayscale ramp. Item 256 is
/// the configured foreground color, item 257 is the configured background
/// color, item 258 is the cursor color. Following that are 8 positions for dim colors.
/// Item 267 is the bright foreground color, 268 the dim foreground.
#[derive(Copy, Clone)]
pub struct List([Rgb; COUNT]);

impl<'a> From<&'a Colors> for List {
    fn from(colors: &Colors) -> List {
        // Type inference fails without this annotation.
        let mut list = List([Rgb::default(); COUNT]);

        list.fill_named(colors);
        list.fill_cube(colors);
        list.fill_gray_ramp(colors);

        list
    }
}

impl List {
    pub fn fill_named(&mut self, colors: &Colors) {
        // Normals.
        self[NamedColor::Black] = colors.normal().black;
        self[NamedColor::Red] = colors.normal().red;
        self[NamedColor::Green] = colors.normal().green;
        self[NamedColor::Yellow] = colors.normal().yellow;
        self[NamedColor::Blue] = colors.normal().blue;
        self[NamedColor::Magenta] = colors.normal().magenta;
        self[NamedColor::Cyan] = colors.normal().cyan;
        self[NamedColor::White] = colors.normal().white;

        // Brights.
        self[NamedColor::BrightBlack] = colors.bright().black;
        self[NamedColor::BrightRed] = colors.bright().red;
        self[NamedColor::BrightGreen] = colors.bright().green;
        self[NamedColor::BrightYellow] = colors.bright().yellow;
        self[NamedColor::BrightBlue] = colors.bright().blue;
        self[NamedColor::BrightMagenta] = colors.bright().magenta;
        self[NamedColor::BrightCyan] = colors.bright().cyan;
        self[NamedColor::BrightWhite] = colors.bright().white;
        self[NamedColor::BrightForeground] = colors
            .primary
            .bright_foreground
            .unwrap_or(colors.primary.foreground);

        // Foreground and background.
        self[NamedColor::Foreground] = colors.primary.foreground;
        self[NamedColor::Background] = colors.primary.background;

        // Background for custom cursor colors.
        self[NamedColor::Cursor] = Rgb::default();

        // Dims.
        self[NamedColor::DimForeground] = colors
            .primary
            .dim_foreground
            .unwrap_or(colors.primary.foreground * DIM_FACTOR);
        match colors.dim {
            Some(ref dim) => {
                // trace!("Using config-provided dim colors");
                self[NamedColor::DimBlack] = dim.black;
                self[NamedColor::DimRed] = dim.red;
                self[NamedColor::DimGreen] = dim.green;
                self[NamedColor::DimYellow] = dim.yellow;
                self[NamedColor::DimBlue] = dim.blue;
                self[NamedColor::DimMagenta] = dim.magenta;
                self[NamedColor::DimCyan] = dim.cyan;
                self[NamedColor::DimWhite] = dim.white;
            }
            None => {
                // trace!("Deriving dim colors from normal colors");
                self[NamedColor::DimBlack] = colors.normal().black * DIM_FACTOR;
                self[NamedColor::DimRed] = colors.normal().red * DIM_FACTOR;
                self[NamedColor::DimGreen] = colors.normal().green * DIM_FACTOR;
                self[NamedColor::DimYellow] = colors.normal().yellow * DIM_FACTOR;
                self[NamedColor::DimBlue] = colors.normal().blue * DIM_FACTOR;
                self[NamedColor::DimMagenta] = colors.normal().magenta * DIM_FACTOR;
                self[NamedColor::DimCyan] = colors.normal().cyan * DIM_FACTOR;
                self[NamedColor::DimWhite] = colors.normal().white * DIM_FACTOR;
            }
        }
    }

    pub fn fill_cube(&mut self, colors: &Colors) {
        let mut index: usize = 16;
        // Build colors.
        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    // Override colors 16..232 with the config (if present).
                    if let Some(indexed_color) = colors
                        .indexed_colors
                        .iter()
                        .find(|ic| ic.index == index as u8)
                    {
                        self[index] = indexed_color.color;
                    } else {
                        self[index] = Rgb {
                            r: if r == 0 { 0 } else { r * 40 + 55 },
                            b: if b == 0 { 0 } else { b * 40 + 55 },
                            g: if g == 0 { 0 } else { g * 40 + 55 },
                        };
                    }
                    index += 1;
                }
            }
        }

        debug_assert!(index == 232);
    }

    pub fn fill_gray_ramp(&mut self, colors: &Colors) {
        let mut index: usize = 232;

        for i in 0..24 {
            // Index of the color is number of named colors + number of cube colors + i.
            let color_index = 16 + 216 + i;

            // Override colors 232..256 with the config (if present).
            if let Some(indexed_color) = colors
                .indexed_colors
                .iter()
                .find(|ic| ic.index == color_index)
            {
                self[index] = indexed_color.color;
                index += 1;
                continue;
            }

            let value = i * 10 + 8;
            self[index] = Rgb {
                r: value,
                g: value,
                b: value,
            };
            index += 1;
        }

        debug_assert!(index == 256);
    }
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("List[..]")
    }
}

impl Index<NamedColor> for List {
    type Output = Rgb;

    #[inline]
    fn index(&self, idx: NamedColor) -> &Self::Output {
        &self.0[idx as usize]
    }
}

impl IndexMut<NamedColor> for List {
    #[inline]
    fn index_mut(&mut self, idx: NamedColor) -> &mut Self::Output {
        &mut self.0[idx as usize]
    }
}

impl Index<usize> for List {
    type Output = Rgb;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

impl IndexMut<usize> for List {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

impl Index<u8> for List {
    type Output = Rgb;

    #[inline]
    fn index(&self, idx: u8) -> &Self::Output {
        &self.0[idx as usize]
    }
}

impl IndexMut<u8> for List {
    #[inline]
    fn index_mut(&mut self, idx: u8) -> &mut Self::Output {
        &mut self.0[idx as usize]
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Colors {
    pub primary: PrimaryColors,
    normal: NormalColors,
    bright: BrightColors,
    pub dim: Option<AnsiColors>,
    pub indexed_colors: Vec<IndexedColor>,
}

impl Colors {
    pub fn normal(&self) -> &AnsiColors {
        &self.normal.0
    }

    pub fn bright(&self) -> &AnsiColors {
        &self.bright.0
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct IndexedColor {
    pub index: u8,
    pub color: Rgb,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
struct SelectionColors {
    pub text: Option<Rgb>,
    pub background: Option<Rgb>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrimaryColors {
    pub background: Rgb,
    pub foreground: Rgb,
    pub bright_foreground: Option<Rgb>,
    pub dim_foreground: Option<Rgb>,
}

impl Default for PrimaryColors {
    fn default() -> Self {
        PrimaryColors {
            background: default_background(),
            foreground: default_foreground(),
            bright_foreground: Default::default(),
            dim_foreground: Default::default(),
        }
    }
}

fn default_background() -> Rgb {
    Rgb {
        r: 0x1d,
        g: 0x1f,
        b: 0x21,
    }
}

fn default_foreground() -> Rgb {
    Rgb {
        r: 0xc5,
        g: 0xc8,
        b: 0xc6,
    }
}

/// The 8-colors sections of config.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnsiColors {
    pub black: Rgb,
    pub red: Rgb,
    pub green: Rgb,
    pub yellow: Rgb,
    pub blue: Rgb,
    pub magenta: Rgb,
    pub cyan: Rgb,
    pub white: Rgb,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NormalColors(AnsiColors);

impl Default for NormalColors {
    fn default() -> Self {
        NormalColors(AnsiColors {
            black: Rgb {
                r: 0x1d,
                g: 0x1f,
                b: 0x21,
            },
            red: Rgb {
                r: 0xcc,
                g: 0x66,
                b: 0x66,
            },
            green: Rgb {
                r: 0xb5,
                g: 0xbd,
                b: 0x68,
            },
            yellow: Rgb {
                r: 0xf0,
                g: 0xc6,
                b: 0x74,
            },
            blue: Rgb {
                r: 0x81,
                g: 0xa2,
                b: 0xbe,
            },
            magenta: Rgb {
                r: 0xb2,
                g: 0x94,
                b: 0xbb,
            },
            cyan: Rgb {
                r: 0x8a,
                g: 0xbe,
                b: 0xb7,
            },
            white: Rgb {
                r: 0xc5,
                g: 0xc8,
                b: 0xc6,
            },
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BrightColors(AnsiColors);

impl Default for BrightColors {
    fn default() -> Self {
        BrightColors(AnsiColors {
            black: Rgb {
                r: 0x66,
                g: 0x66,
                b: 0x66,
            },
            red: Rgb {
                r: 0xd5,
                g: 0x4e,
                b: 0x53,
            },
            green: Rgb {
                r: 0xb9,
                g: 0xca,
                b: 0x4a,
            },
            yellow: Rgb {
                r: 0xe7,
                g: 0xc5,
                b: 0x47,
            },
            blue: Rgb {
                r: 0x7a,
                g: 0xa6,
                b: 0xda,
            },
            magenta: Rgb {
                r: 0xc3,
                g: 0x97,
                b: 0xd8,
            },
            cyan: Rgb {
                r: 0x70,
                g: 0xc0,
                b: 0xb1,
            },
            white: Rgb {
                r: 0xea,
                g: 0xea,
                b: 0xea,
            },
        })
    }
}

/// Standard colors.
///
/// The order here matters since the enum should be castable to a `usize` for
/// indexing a color list.
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum NamedColor {
    Black = 0,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Foreground = 256,
    Background,
    Cursor,
    DimBlack,
    DimRed,
    DimGreen,
    DimYellow,
    DimBlue,
    DimMagenta,
    DimCyan,
    DimWhite,
    BrightForeground,
    DimForeground,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Named(NamedColor),
    Spec(Rgb),
    Indexed(u8),
}

/// Terminal character attributes.
#[derive(Debug, Eq, PartialEq)]
pub enum Attr {
    Reset,
    Bold,
    Dim,
    Italic,
    Underline,
    BlinkSlow,
    BlinkFast,
    Reverse,
    Hidden,
    Strike,
    CancelBold,
    CancelBoldDim,
    CancelItalic,
    CancelUnderline,
    CancelBlink,
    CancelReverse,
    CancelHidden,
    CancelStrike,
    Foreground(Color),
    Background(Color),
}

pub fn attrs_from_sgr_parameters(parameters: &[i64]) -> Vec<Option<Attr>> {
    let mut i = 0;
    let mut attrs = Vec::with_capacity(parameters.len());
    loop {
        if i >= parameters.len() {
            break;
        }

        let attr = match parameters[i] {
            0 => Some(Attr::Reset),
            1 => Some(Attr::Bold),
            2 => Some(Attr::Dim),
            3 => Some(Attr::Italic),
            4 => Some(Attr::Underline),
            5 => Some(Attr::BlinkSlow),
            6 => Some(Attr::BlinkFast),
            7 => Some(Attr::Reverse),
            8 => Some(Attr::Hidden),
            9 => Some(Attr::Strike),
            21 => Some(Attr::CancelBold),
            22 => Some(Attr::CancelBoldDim),
            23 => Some(Attr::CancelItalic),
            24 => Some(Attr::CancelUnderline),
            25 => Some(Attr::CancelBlink),
            27 => Some(Attr::CancelReverse),
            28 => Some(Attr::CancelHidden),
            29 => Some(Attr::CancelStrike),
            30 => Some(Attr::Foreground(Color::Named(NamedColor::Black))),
            31 => Some(Attr::Foreground(Color::Named(NamedColor::Red))),
            32 => Some(Attr::Foreground(Color::Named(NamedColor::Green))),
            33 => Some(Attr::Foreground(Color::Named(NamedColor::Yellow))),
            34 => Some(Attr::Foreground(Color::Named(NamedColor::Blue))),
            35 => Some(Attr::Foreground(Color::Named(NamedColor::Magenta))),
            36 => Some(Attr::Foreground(Color::Named(NamedColor::Cyan))),
            37 => Some(Attr::Foreground(Color::Named(NamedColor::White))),
            38 => {
                let mut start = 0;
                if let Some(color) = parse_sgr_color(&parameters[i..], &mut start) {
                    i += start;
                    Some(Attr::Foreground(color))
                } else {
                    None
                }
            }
            39 => Some(Attr::Foreground(Color::Named(NamedColor::Foreground))),
            40 => Some(Attr::Background(Color::Named(NamedColor::Black))),
            41 => Some(Attr::Background(Color::Named(NamedColor::Red))),
            42 => Some(Attr::Background(Color::Named(NamedColor::Green))),
            43 => Some(Attr::Background(Color::Named(NamedColor::Yellow))),
            44 => Some(Attr::Background(Color::Named(NamedColor::Blue))),
            45 => Some(Attr::Background(Color::Named(NamedColor::Magenta))),
            46 => Some(Attr::Background(Color::Named(NamedColor::Cyan))),
            47 => Some(Attr::Background(Color::Named(NamedColor::White))),
            48 => {
                let mut start = 0;
                if let Some(color) = parse_sgr_color(&parameters[i..], &mut start) {
                    i += start;
                    Some(Attr::Background(color))
                } else {
                    None
                }
            }
            49 => Some(Attr::Background(Color::Named(NamedColor::Background))),
            90 => Some(Attr::Foreground(Color::Named(NamedColor::BrightBlack))),
            91 => Some(Attr::Foreground(Color::Named(NamedColor::BrightRed))),
            92 => Some(Attr::Foreground(Color::Named(NamedColor::BrightGreen))),
            93 => Some(Attr::Foreground(Color::Named(NamedColor::BrightYellow))),
            94 => Some(Attr::Foreground(Color::Named(NamedColor::BrightBlue))),
            95 => Some(Attr::Foreground(Color::Named(NamedColor::BrightMagenta))),
            96 => Some(Attr::Foreground(Color::Named(NamedColor::BrightCyan))),
            97 => Some(Attr::Foreground(Color::Named(NamedColor::BrightWhite))),
            100 => Some(Attr::Background(Color::Named(NamedColor::BrightBlack))),
            101 => Some(Attr::Background(Color::Named(NamedColor::BrightRed))),
            102 => Some(Attr::Background(Color::Named(NamedColor::BrightGreen))),
            103 => Some(Attr::Background(Color::Named(NamedColor::BrightYellow))),
            104 => Some(Attr::Background(Color::Named(NamedColor::BrightBlue))),
            105 => Some(Attr::Background(Color::Named(NamedColor::BrightMagenta))),
            106 => Some(Attr::Background(Color::Named(NamedColor::BrightCyan))),
            107 => Some(Attr::Background(Color::Named(NamedColor::BrightWhite))),
            _ => None,
        };

        attrs.push(attr);

        i += 1;
    }
    attrs
}

/// Parse a color specifier from list of attributes.
fn parse_sgr_color(attrs: &[i64], i: &mut usize) -> Option<Color> {
    if attrs.len() < 2 {
        return None;
    }

    match attrs[*i + 1] {
        2 => {
            // RGB color spec.
            if attrs.len() < 5 {
                eprintln!("Expected RGB color spec; got {:?}", attrs);
                return None;
            }

            let r = attrs[*i + 2];
            let g = attrs[*i + 3];
            let b = attrs[*i + 4];

            *i += 4;

            let range = 0..256;
            if !range.contains(&r) || !range.contains(&g) || !range.contains(&b) {
                eprintln!("Invalid RGB color spec: ({}, {}, {})", r, g, b);
                return None;
            }

            Some(Color::Spec(Rgb {
                r: r as u8,
                g: g as u8,
                b: b as u8,
            }))
        }
        5 => {
            if attrs.len() < 3 {
                eprintln!("Expected color index; got {:?}", attrs);
                None
            } else {
                *i += 2;
                let idx = attrs[*i];
                match idx {
                    0..=255 => Some(Color::Indexed(idx as u8)),
                    _ => {
                        eprintln!("Invalid color index: {}", idx);
                        None
                    }
                }
            }
        }
        _ => {
            eprintln!("Unexpected color attr: {}", attrs[*i + 1]);
            None
        }
    }
}
