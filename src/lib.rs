#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod fb;
mod font;
mod num;
mod pixel;

pub use fb::Framebuffer;
pub use fb::Rect;
use font::Glyph;
pub use font::{
    vga::{VGAFont, VGAFontConfig},
    Font, Point,
};
use num::Saturating;

#[cfg(feature = "alloc")]
pub use font::truetype::TrueTypeFont;

pub use pixel::*;

pub struct Fbterm<'a, P: Pixel, F: Font> {
    pub framebuffer: Framebuffer<'a, P>,
    font: F,
    x: Saturating,
    y: Saturating,
}

impl<'a, P: Pixel, F: Font> Fbterm<'a, P, F> {
    pub fn new(framebuffer: Framebuffer<'a, P>, font: F) -> Fbterm<'a, P, F> {
        let width = framebuffer.width();
        let height = framebuffer.height();
        Fbterm {
            framebuffer,
            font,
            x: Saturating::new(width - 1),
            y: Saturating::new(height - 1),
        }
    }

    pub fn clear(&mut self) {
        self.x.set(0);
        self.y.set(0);
        self.framebuffer.clear();
    }

    pub fn putc(&mut self, c: char) {
        match c {
            '\n' => {
                // FIXME: should \n reset x ?
                self.x.set(0);
                self.y += self.font.height();
                if self.y.add_check(self.font.height()).1 {
                    self.scroll();
                }
            }
            '\r' => {
                self.x.set(0);
            }
            '\t' => {
                self.print("    ");
            }
            _ => {
                let glyph = self
                    .font
                    .get_glyph(c)
                    .unwrap_or(self.font.get_glyph(' ').unwrap());
                let (mut next_x, overflow) = self.x.add_check(glyph.advance);
                if overflow {
                    self.x.set(0);
                    next_x = glyph.advance;
                    self.y += self.font.height();
                }
                if self.y.add_check(self.font.height()).1 {
                    self.scroll();
                }
                self.draw_glyph(glyph);
                self.x.set(next_x);
            }
        }
    }

    pub fn print(&mut self, s: &str) {
        for c in s.chars() {
            self.putc(c)
        }
    }

    pub fn get_font(&self) -> &F {
        &self.font
    }

    pub fn get_font_mut(&mut self) -> &mut F {
        &mut self.font
    }

    pub fn change_font<T: Font>(mut self, font: T) -> Fbterm<'a, P, T> {
        // TODO: redraw after reset
        self.clear();
        Fbterm::new(self.framebuffer, font)
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.framebuffer.width()
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.framebuffer.height()
    }

    fn draw_glyph(&mut self, glyph: Glyph) {
        let basex = *self.x + glyph.x;
        let basey = *self.y as isize + glyph.y;
        assert!(glyph.y >= 0);
        let basey = basey as usize;
        assert!(
            basex + glyph.width <= self.width(),
            "x is overflow: {} + {}",
            basex, glyph.width
        );
        assert!(
            basey + glyph.height <= self.height(),
            "y is overflow: {} + {}",
            basey, glyph.height
        );
        for y in 0..glyph.height {
            for x in 0..glyph.width {
                match self.font.get_pixel(&glyph, x, y) {
                    font::Point::Bit(bit) => unsafe {
                        self.framebuffer.draw_bit(basex + x, basey + y, bit)
                    },
                    font::Point::Coverage(cov) => unsafe {
                        self.framebuffer.draw_alpha(basex + x, basey + y, cov)
                    },
                };
            }
        }
    }

    /* FIXME: This is too slow */
    fn scroll(&mut self) {
        let diff = *self.y + self.font.height() + 1 - self.height();
        self.framebuffer.copy_rect(
            Rect::new(0, diff, self.framebuffer.width(), self.height() - diff),
            Rect::new(0, 0, self.framebuffer.width(), self.height() - diff),
        );
        self.framebuffer.draw_rect(
            Rect::new(0, self.height() - diff, self.framebuffer.width(), diff),
            self.framebuffer.get_background(),
        );
        self.y -= diff;
    }

    /*
    fn scroll(&mut self) {
        self.clear();
        self.y = 0;
    }
    */
}

impl<'a, P: Pixel, F: Font> core::fmt::Write for Fbterm<'a, P, F> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print(s);
        Ok(())
    }
}

// FIXME: Really safe?
unsafe impl<'a, P: Pixel, F: Font> Send for Fbterm<'a, P, F> {}
unsafe impl<'a, P: Pixel, F: Font> Sync for Fbterm<'a, P, F> {}
