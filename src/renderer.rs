use freetype as ft;
use image::{DynamicImage, ImageBuffer, Pixel, Rgba};

use crate::ansi::PrimaryColors;
use crate::parser::Token;

const HEIGHT: u32 = 32;

fn draw_bitmap(
    bitmap: ft::Bitmap,
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    color: crate::ansi::Rgb,
    x: usize,
    y: usize,
) {
    let mut p = 0;
    let mut q = 0;
    let w = bitmap.width() as usize;
    let x_max = x + w;
    let y_max = y + bitmap.rows() as usize;

    for i in x..x_max {
        for j in y..y_max {
            let v = bitmap.buffer()[q * w + p] as f32;
            let color = Rgba([color.r, color.g, color.b, v as u8]);

            image.get_pixel_mut(i as u32, j as u32).blend(&color);

            // For debugging bounding boxes:
            // image.put_pixel(i as u32, j as u32, color);

            q += 1;
        }
        q = 0;
        p += 1;
    }
}

pub fn render(tokens: &[Token], font: &str, out: &str) {
    let library = ft::Library::init().unwrap();

    // Load the font
    let face = library.new_face(font, 0).unwrap();

    // The font size to use
    face.set_pixel_sizes(0, HEIGHT).unwrap();

    let chars_count = tokens
        .iter()
        .filter(|t| match t {
            Token::Char(_) => true,
            _ => false,
        })
        .count();

    // Create a new RGBA image
    let padding_left = 10;
    let padding_right = 10;
    let image_width: u32 = {
        // Generate a glyph to get the width. Because we're using a monospaced font we can use any char.
        face.load_char('m' as usize, ft::face::LoadFlag::RENDER)
            .unwrap();
        let width = face.glyph().advance().x >> 6;
        (width as u32 * chars_count as u32) + padding_left + padding_right
    };
    let image_height = HEIGHT + 16;
    let mut image = DynamicImage::new_rgba8(image_width, image_height).to_rgba();

    // Black background
    for (_, _, p) in image.enumerate_pixels_mut() {
        *p = Rgba([0, 0, 0, 255]);
    }

    let mut color = PrimaryColors::default().foreground;
    let mut x_pos = padding_left as usize;

    for token in tokens {
        match token {
            Token::Color(c) => {
                color = *c;
            }
            Token::Char(c) => {
                face.load_char(*c as usize, ft::face::LoadFlag::RENDER)
                    .unwrap();
                let glyph = face.glyph();

                let x = glyph.bitmap_left() as usize + x_pos;
                let y = HEIGHT as usize - glyph.bitmap_top() as usize;

                draw_bitmap(glyph.bitmap(), &mut image, color, x, y);
                let advance = glyph.advance();
                x_pos += advance.x as usize >> 6;
            }
        }
    }

    // Save the image to a png file
    image.save(&out).unwrap();
}
