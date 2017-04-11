#![feature(core_float)]
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

    pub fn set_offset(&mut self, off_x: usize, off_y: usize) {
        self.off_x = off_x;
        self.off_y = off_y;
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

        //let scale_factor = self.font_info.scale_for_pixel_height(self.text_size as f32);
        let scale_factor = self.font_info.scale_for_mapping_em_to_pixels(self.text_size as f32);

        let rect = self.font_info.get_codepoint_bitmap_box(c.into(),scale_factor,scale_factor).unwrap();

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

#[no_mangle]
pub unsafe extern fn fmodf(x : f32, y : f32) -> f32 {
    use core::mem;
    use core::num::Float;

    let mut ux_i: u32 = mem::transmute(x);
    let mut uy_i: u32 = mem::transmute(y);

    let mut _current_block;
    let mut ex : i32 = (ux_i >> 23i32 & 0xffu32) as (i32);
    let mut ey : i32 = (uy_i >> 23i32 & 0xffu32) as (i32);
    let sx : u32 = ux_i & 0x80000000u32;
    let mut i : u32;
    let mut uxi : u32 = ux_i;
    if uy_i << 1i32 == 0u32 || y.is_nan() || ex == 0xffi32 {
        x * y / (x * y)
    } else if uxi << 1i32 <= uy_i << 1i32 {
        if uxi << 1i32 == uy_i << 1i32 { 0i32 as (f32) * x } else { x }
    } else {
        if ex == 0 {
            i = uxi << 9i32;
            'loop5: loop {
                if !(i >> 31i32 == 0u32) {
                    break;
                }
                ex = ex - 1;
                i = i << 1i32;
            }
            uxi = uxi << -ex + 1i32;
        } else {
            uxi = uxi & 1u32.wrapping_neg() >> 9i32;
            uxi = uxi | 1u32 << 23i32;
        }
        if ey == 0 {
            i = uy_i << 9i32;
            'loop10: loop {
                if !(i >> 31i32 == 0u32) {
                    break;
                }
                ey = ey - 1;
                i = i << 1i32;
            }
            uy_i = uy_i << -ey + 1i32;
        } else {
            uy_i = uy_i & 1u32.wrapping_neg() >> 9i32;
            uy_i = uy_i | 1u32 << 23i32;
        }
        'loop12: loop {
            if !(ex > ey) {
                _current_block = 13;
                break;
            }
            i = uxi.wrapping_sub(uy_i);
            if i >> 31i32 == 0u32 {
                if i == 0u32 {
                    _current_block = 28;
                    break;
                }
                uxi = i;
            }
            uxi = uxi << 1i32;
            ex = ex - 1;
        }
        if _current_block == 13 {
            i = uxi.wrapping_sub(uy_i);
            if i >> 31i32 == 0u32 {
                if i == 0u32 {
                    return 0i32 as (f32) * x;
                } else {
                    uxi = i;
                }
            }
            'loop16: loop {
                if !(uxi >> 23i32 == 0u32) {
                    break;
                }
                uxi = uxi << 1i32;
                ex = ex - 1;
            }
            if ex > 0i32 {
                uxi = uxi.wrapping_sub(1u32 << 23i32);
                uxi = uxi | ex as (u32) << 23i32;
            } else {
                uxi = uxi >> -ex + 1i32;
            }
            uxi = uxi | sx;
            ux_i = uxi;
            mem::transmute(ux_i)
        } else {
            0i32 as (f32) * x
        }
    }
}
