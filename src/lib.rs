#![no_std]
mod fb;
mod font;
mod pixel;

pub use fb::Framebuffer;
use fb::Rect;
use font::Font;
pub use font::Fonts;
pub use pixel::*;

pub struct Fbterm<'a, T: Pixel> {
    pub framebuffer: Framebuffer<'a, T>,
    font: Font<'a>,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl<'a, T: Pixel> Fbterm<'a, T> {
    pub fn new(framebuffer: Framebuffer<'a, T>, font: Fonts) -> Fbterm<'a, T> {
        let font = Font::new(font);
        let height = framebuffer.height() / font.height();
        let width = framebuffer.width() / font.width();
        Fbterm {
            framebuffer,
            font,
            x: 0,
            y: 0,
            height,
            width,
        }
    }

    pub fn clear(&mut self) {
        self.x = 0;
        self.y = 0;
        self.framebuffer.clear();
    }

    pub fn putc(&mut self, c: u8) {
        match c {
            b'\n' => {
                self.x = 0;
                self.y += 1;
            }
            _ => {
                let bitmap = self.font.char(c);
                self.draw_font(bitmap);
                self.x += 1;
                if self.x >= self.width {
                    self.x = 0;
                    self.y += 1;
                }
                if self.y >= self.height {
                    self.scroll();
                    self.y = self.height - 1;
                }
            }
        }
    }

    pub fn print(&mut self, s: &str) {
        for c in s.bytes() {
            self.putc(c)
        }
    }

    fn draw_font(&mut self, bitmap: &'a [u8]) {
        bitmap.into_iter().enumerate().for_each(|(y, &c)| {
            for x in 0..self.font.width() {
                let pixel = if ((c >> (self.font.width() - 1 - x)) & 0x1) == 0x1 {
                    self.framebuffer.get_foreground()
                } else {
                    self.framebuffer.get_background()
                };
                self.framebuffer.draw_pixel(
                    self.x * self.font.width() + x,
                    self.y * self.font.height() + y,
                    pixel,
                );
            }
        })
    }

    fn scroll(&mut self) {
        for y in 0..(self.height - 1) {
            self.framebuffer.copy_rect(
                Rect::new(
                    0,
                    (y + 1) * self.font.height(),
                    self.framebuffer.width(),
                    self.font.height(),
                ),
                Rect::new(
                    0,
                    y * self.font.height(),
                    self.framebuffer.width(),
                    self.font.height(),
                ),
            );
        }
        self.framebuffer.draw_rect(
            Rect::new(
                0,
                (self.height - 1) * self.font.height(),
                self.framebuffer.width(),
                self.font.height(),
            ),
            self.framebuffer.get_background(),
        );
    }
}

impl<'a, T: Pixel> core::fmt::Write for Fbterm<'a, T> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s);
        Ok(())
    }
}

// FIXME: Really safe?
unsafe impl<'a, T: Pixel> Send for Fbterm<'a, T> {}
unsafe impl<'a, T: Pixel> Sync for Fbterm<'a, T> {}
