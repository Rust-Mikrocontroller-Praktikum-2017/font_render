#![no_std]

extern crate stb_truetype;
extern crate font_rs;

use stb_truetype::{FontInfo, Rect};
use font_rs::font;

pub static TTF: &[u8] = include_bytes!("RobotoMono-Bold.ttf");

#[derive(Debug)]
pub enum Error {
    FontInfo,
    Font(font::FontError),
}

impl From<font::FontError> for Error {
    fn from(e: font::FontError) -> Error {
        Error::Font(e)
    }
}

pub struct TextWriter<'a> {
    font_info: FontInfo<&'a [u8]>,
    font: font::Font<'a>,
    text_size: u32,
    wrap_at: usize,
    off_x: usize,
    off_y: usize,
}

impl<'a> TextWriter<'a> {
    pub fn new(ttf: &'a [u8], text_size: u32, wrap_at: usize) -> Result<Self, Error> {
        Ok(TextWriter {
               font_info: FontInfo::new(ttf, 0).ok_or(Error::FontInfo)?,
               font: font::parse(ttf)?,
               text_size: text_size,
               wrap_at: wrap_at,
               off_x: 0,
               off_y: 0,
           })
    }

    pub fn default() -> Result<Self, Error> {
        TextWriter::new(TTF, 11, 480)
    }

    pub fn set_text_size(&mut self, text_size: u32) {
        self.text_size = text_size;
    }

    pub fn print_char<F>(&mut self, mut c: char, mut print_at: F)
        where
        F: FnMut(Coords, u8)
    {
        let space = match c {
            ' ' => {
                c = '-';
                true
            },
            '\n' => {
                self.off_x = 0;
                self.off_y += self.text_size as usize;
                return;
            }
            _ => false,
        };

        let glyph_id = self.font_info.find_glyph_index(c.into());
        let glyph = self.font
            .render_glyph(glyph_id as u16, self.text_size)
            .expect("Failed to render glyph");
        if self.off_x + glyph.width >= self.wrap_at && !space {
            self.off_x = 0;
            self.off_y += self.text_size as usize;
        }
        if !space {
            for y in 0..glyph.height {
                for x in 0..glyph.width {
                    let val = glyph.data[y * glyph.width + x];
                    let x_coord = (self.off_x + x) as i32 + glyph.left;
                    let y_coord = (self.off_y + y) as i32 + glyph.top + self.text_size as i32;
                    print_at(Coords{x: x_coord as usize, y: y_coord as usize}, val);
                }
            }
        }
        self.off_x += (glyph.width as i32 + glyph.left) as usize;
    }

    pub fn print_str<F>(&mut self, s: &str, mut print_at: F)
        where
        F: FnMut(Coords, u8)
    {
        for c in s.chars() {
            self.print_char(c, &mut print_at);
        }
    }

    pub fn aabb_char(&self, mut c: char) -> Rect<i32> {
        match c {
            ' ' => {
                c = '-';
            },
            '\n' => {
                c = '-';
            }
            _ => {},
        }

        let mut rect = self.font_info.get_codepoint_bitmap_box(c.into(),1.0,1.0).unwrap();

        //let scale_factor = self.font_info.scale_for_pixel_height(self.text_size as f32);
        let scale_factor = self.font_info.scale_for_mapping_em_to_pixels(self.text_size as f32);

        rect.x0 = ((rect.x0 as f32) * scale_factor) as i32;
        rect.x1 = ((rect.x1 as f32) * scale_factor) as i32;
        rect.y0 = ((rect.y0 as f32) * scale_factor) as i32;
        rect.y1 = ((rect.y1 as f32) * scale_factor) as i32;

        rect
    }

    pub fn width_height(&self, s: &str) -> (u32, u32) {
        use core::cmp::max;

        let mut width = 0;
        let mut height = 0;

        for c in s.chars() {
            let char_rect = self.aabb_char(c);
            width += char_rect.x1 - char_rect.x0;
            height = max(height, char_rect.y1 - char_rect.y0);
        }
        (width as u32, height as u32)
    }
}

pub struct Coords {
    pub x: usize,
    pub y: usize,
}
