# font_render

This library uses [font-rs] and [stb_truetype] to render truetype fonts und print them to screen.

[font-rs]: https://github.com/google/font-rs
[stb_truetype]: https://docs.rs/crate/stb_truetype

## Usage

Include `font_render` as dependency in your `Cargo.toml`:

```toml
[dependencies]
font_render = {git = "https://github.com/Rust-Mikrocontroller-Praktikum-2017/font_render.git"}
```

and link it to your crate through `extern crate font_render`.

### Creating a `Writer` struct
Now you can create a `Lcd::text_writer` method in `src/lcd/mod.rs`:

```rust
use font_render;

pub struct Writer<'a> {
    lcd: &'a mut Lcd,
    text_writer: font_render::TextWriter<'a>,
}

impl Lcd {
    ...

    pub fn text_writer(&mut self) -> Result<Writer, font_render::Error> {
        Ok(Writer {
            lcd: self,
            text_writer: font_render::TextWriter::default(),
        })
    }
}
```

The `text_writer` method returns a new `Writer` struct that uses a default `TextWriter`. The default is using the bold Roboto Mono font with size 11 and wrapping at 480 pixels (display width). If you want to use your own parameters, you can use the `new` method instead of `default`.

In order to print strings, we add the following methods to our new `Writer` struct:

```rust
impl<'a> Writer<'a> {
    pub fn print_char(&mut self, c: char) {
        let &mut Self { ref mut text_writer, ref mut lcd} = self;

        text_writer.print_char(c, |coords, value| {
            let color = ((value as u16) << 8) | 0xff;
            lcd.print_point_color_at(coords.x as u16, coords.y as u16, color);
        });
    }

    pub fn print_str(&mut self, s: &str) {
        for c in s.chars() {
            self.print_char(c);
        }
    }
}
```

Now we can print characters and strings to the LCD. The `text_writer.print_char` method takes a closure that is passed the pixel coordinates and the transparency value. From this value, we calculate a color and print it to screen using `lcd.print_point_color_at`.

### Changing the color format
Fonts only look good with transparency support, so we change our color format to `AL88` (one byte alpha, one byte luminance):

```rust
// in src/lcd/init.rs near line 116
ltdc.l2pfcr.update(|r| r.set_pf(0b111)); // set_pixel_format to AL88
```

So we change the `0b011` (for ARGB1555) to `0b111` (for `AL88`).

Unfortunately this means that we can't really display colors anymore. To solve this, you could switch to a 4 byte color format such as `ARGB888` (`0b000`), but this means that you have to rewrite the LCD methods (including `print_point_at` and `print_point_color_at`). Pull requests welcome!

### Formatting macros
To support Rust's `writeln` macro, we can implement the `core::fmt::Write` trait:

```rust
use core::fmt;

impl<'a> fmt::Write for Writer<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print_str(s);
        Ok(())
    }
}
```

Now we can print from our `main`:

```rust
use core::fmt::Write;
let mut text_writer = lcd.text_writer().unwrap();
text_writer.print_str("Hello World!\n");
writeln!(&mut text_writer, "Using formatting macros {} {} {:?}", 42, 3.14, &[1,2,3,4]);
```

Note that you can't use the lcd struct directly as long as the `Writer` instance exists. To drop it earlier, you can put it into an additional block:

```rust
{
    let mut text_writer = lcd.text_writer().unwrap();
    // text_writer is valid here, but lcd is not
}
// lcd is valid again
```
