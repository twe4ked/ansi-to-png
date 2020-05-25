use image::{DynamicImage, Pixel, Rgba};
use rusttype::{point, Font, Scale};

use std::fs;

use crate::ansi::WHITE;
use crate::parser::Token;

pub fn render(tokens: &[Token], chars_count: usize, font: &str, out: &str) {
    // Load the font
    let font_data = fs::read(&font).unwrap();

    // This only succeeds if collection consists of one font
    let font = Font::try_from_bytes(&font_data).unwrap();

    // The font size to use
    let scale = Scale::uniform(32.0);

    let (glyphs_height, glyphs_width) = {
        let v_metrics = font.v_metrics(scale);
        let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;

        // Generate a glyph to get the width/height. Because we're using a monospaced font we can
        // use any char.
        let glyph = font.glyph('m').scaled(scale).positioned(point(0.0, 0.0));
        let glyphs_width = glyph.pixel_bounding_box().unwrap().width() as u32;

        (glyphs_height, glyphs_width)
    };

    let some_random_height_padding = 10;

    // Create a new RGBA image
    let padding_left = 10;
    let padding_right = 10;
    let image_width = (glyphs_width * chars_count as u32) + padding_left + padding_right;
    let image_height = glyphs_height + some_random_height_padding;
    let mut image = DynamicImage::new_rgba8(image_width, image_height).to_rgba();
    // Black background
    for (_, _, p) in image.enumerate_pixels_mut() {
        *p = Rgba([0, 0, 0, 255]);
    }

    let some_random_padding = 28.0;

    let mut color = WHITE;
    let mut x_pos = padding_left;

    let colors_and_glyphs: Vec<_> = tokens
        .iter()
        .filter_map(|token| match token {
            Token::Color(c) => {
                color = *c;
                None
            }
            Token::Char(c) => {
                let glyph = font
                    .glyph(*c)
                    .scaled(scale)
                    .positioned(point(x_pos as f32, some_random_padding));

                x_pos += glyphs_width;

                Some((color, glyph))
            }
        })
        .collect();

    for (color, glyph) in colors_and_glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            // Draw the glyph into the image per-pixel by using the draw closure
            glyph.draw(|x, y, v| {
                // Turn the coverage into an alpha value
                let color = Rgba([color.r, color.g, color.b, (v * 255.0) as u8]);

                // Offset the position by the glyph bounding box
                let x = x + bounding_box.min.x as u32;
                let y = y + bounding_box.min.y as u32;

                image.get_pixel_mut(x, y).blend(&color);

                // image.put_pixel(x, y, color);
            });
        }
    }

    // Save the image to a png file
    image.save(&out).unwrap();
}
