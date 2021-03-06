#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod fb;
mod font;
mod num;
mod pixel;

#[cfg(feature = "alloc")]
use alloc::{collections::VecDeque, string::String};
pub use fb::Framebuffer;
pub use fb::Rect;
#[cfg(feature = "alloc")]
pub use font::truetype::TrueTypeFont;
pub use font::{
    vga::{VGAFont, VGAFontConfig},
    Font, Glyph, Point,
};
use num::Saturating;
pub use pixel::*;

pub struct Fbterm<'a, P: Pixel, F: Font> {
    pub framebuffer: Framebuffer<'a, P>,
    font: F,
    x: Saturating,
    y: Saturating,
    dirty: Option<Rect>,
    #[cfg(feature = "alloc")]
    lines: VecDeque<String>,
}

impl<'a, P: Pixel, F: Font> Fbterm<'a, P, F> {
    pub fn new(framebuffer: Framebuffer<'a, P>, font: F) -> Fbterm<'a, P, F> {
        let width = framebuffer.width();
        let height = framebuffer.height();
        #[cfg(feature = "alloc")]
        let lines = {
            let lines_len = height / font.height();
            let mut lines = VecDeque::with_capacity(lines_len);
            lines.push_back(String::new());
            lines
        };
        Fbterm {
            framebuffer,
            font,
            x: Saturating::new(width - 1),
            y: Saturating::new(height - 1),
            dirty: None,
            #[cfg(feature = "alloc")]
            lines,
        }
    }

    pub fn clear(&mut self) {
        self.x.set(0);
        self.y.set(0);
        self.framebuffer.clear();
        self.framebuffer.flush(None);
        self.dirty = None;
        #[cfg(feature = "alloc")]
        {
            self.lines.clear();
            self.lines.push_back(String::new());
        }
    }

    pub fn flush(&mut self) {
        if self.dirty.is_none() {
            return;
        }
        self.framebuffer.flush(self.dirty);
        self.dirty = None;
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
                #[cfg(feature = "alloc")]
                self.lines.push_back(String::new());
            }
            '\r' => {
                self.x.set(0);
                // FIXME: should \r drop all char ?
                #[cfg(feature = "alloc")]
                self.lines.back_mut().unwrap().clear();
            }
            '\t' => {
                self.print("    ");
            }
            #[cfg(feature = "alloc")]
            '\u{08}' => {
                let last_line = self.lines.back_mut().unwrap();
                let last_char = last_line.pop();
                match last_char {
                    Some(c) => {
                        let metrics = self.font.metrics(c).unwrap();
                        self.x -= metrics.advance;
                        let basex = *self.x + metrics.x;
                        let basey = (*self.y as isize + metrics.y) as usize;
                        let clean = Rect::new(basex, basey, metrics.width, metrics.height);
                        self.framebuffer
                            .draw_rect(clean, self.framebuffer.get_background());
                        self.add_dirty(clean);
                    }
                    None => {
                        // FIXME: deal with line change
                    }
                }
            }
            _ => {
                let (c, glyph) = match self.font.get_glyph(c) {
                    Some(g) => (c, g),
                    None => (' ', self.font.get_glyph(' ').unwrap()),
                };
                let (mut next_x, overflow) = self.x.add_check(glyph.advance);
                if overflow {
                    self.x.set(0);
                    next_x = glyph.advance;
                    self.y += self.font.height();
                    if self.y.add_check(self.font.height()).1 {
                        self.scroll();
                    }
                    #[cfg(feature = "alloc")]
                    self.lines.push_back(String::new());
                }
                #[cfg(feature = "alloc")]
                self.lines.back_mut().unwrap().push(c);
                self.draw_glyph(glyph);
                self.x.set(next_x);
            }
        }
    }

    pub fn print(&mut self, s: &str) {
        for c in s.chars() {
            self.putc(c)
        }
        self.flush()
    }

    pub fn get_font(&self) -> &F {
        &self.font
    }

    pub fn get_font_mut(&mut self) -> &mut F {
        &mut self.font
    }

    pub fn change_font<T: Font>(mut self, font: T) -> Fbterm<'a, P, T> {
        #[cfg(feature = "alloc")]
        let lines = core::mem::replace(&mut self.lines, VecDeque::new());
        self.clear();
        let mut term = Fbterm::new(self.framebuffer, font);
        #[cfg(feature = "alloc")]
        {
            for line in lines {
                term.print(&line);
                term.putc('\n');
            }
        }
        term
    }

    fn add_dirty(&mut self, new: Rect) {
        if self.dirty.is_none() {
            self.dirty = Some(new);
            return;
        }
        let old = self.dirty.unwrap();
        let x = old.left().min(new.left());
        let y = old.top().min(new.top());
        let right = old.right().max(new.right());
        let bottom = old.bottom().max(new.bottom());
        let width = right - x;
        let height = bottom - y;
        self.dirty = Some(Rect::new(x, y, width, height))
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.framebuffer.width()
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.framebuffer.height()
    }

    #[cfg(feature = "alloc")]
    #[inline]
    pub fn lines(&self) -> &VecDeque<String> {
        &self.lines
    }

    fn draw_glyph(&mut self, glyph: Glyph) {
        let basex = *self.x + glyph.x;
        let basey = *self.y as isize + glyph.y;
        assert!(glyph.y >= 0);
        let basey = basey as usize;
        assert!(
            basex + glyph.width <= self.width(),
            "x is overflow: {} + {}",
            basex,
            glyph.width
        );
        assert!(
            basey + glyph.height <= self.height(),
            "y is overflow: {} + {}",
            basey,
            glyph.height
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
        self.add_dirty(Rect::new(basex, basey, glyph.width, glyph.height));
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
        self.add_dirty(Rect::new(0, 0, self.width(), self.height()));
        #[cfg(feature = "alloc")]
        self.lines.pop_front();
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
