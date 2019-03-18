use crate::pixel::*;
use core::marker::PhantomData;
pub struct Framebuffer<'a, T: Pixel> {
    base: *mut u8,
    width: usize,
    height: usize,
    background: T,
    foreground: T,
    _lifetime: PhantomData<&'a u8>,
}

impl<'a, T: Pixel> Framebuffer<'a, T> {
    pub unsafe fn new(
        base: *mut u8,
        width: usize,
        height: usize,
        background: T,
        foreground: T,
    ) -> Framebuffer<'a, T> {
        Framebuffer {
            base,
            width,
            height,
            background,
            foreground,
            _lifetime: PhantomData,
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.height * self.width {
            unsafe { self.background.write_volatile(self.base.add(i * T::size())) }
        }
    }

    pub fn copy_rect(&mut self, src: Rect, dst: Rect) {
        debug_assert!(
            src.right() < dst.left()
                || src.bottom() < dst.top()
                || dst.right() < src.left()
                || dst.bottom() < src.top()
        );
        debug_assert!(
            src.right() < self.width
                && src.bottom() < self.height
                && dst.right() < self.width
                && dst.bottom() < self.height
        );
        debug_assert_eq!(src.width, dst.width);
        debug_assert_eq!(src.height, dst.height);
        for y in 0..src.height {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.base
                        .add(((src.y + y) * self.width + src.x) * T::size()),
                    self.base
                        .add(((dst.y + y) * self.width + dst.x) * T::size()),
                    src.width * T::size(),
                );
            }
        }
    }

    pub fn draw_rect(&mut self, dst: Rect, pixel: T) {
        debug_assert!(dst.bottom() < self.height && dst.right() < self.width);
        for y in 0..dst.height {
            for x in 0..dst.width {
                unsafe {
                    pixel.write_volatile(
                        self.base
                            .add(((dst.y + y) * self.width + dst.x + x) * T::size()),
                    )
                }
            }
        }
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, pixel: T) {
        debug_assert!(x < self.width, "Frame buffer accessed out of bounds");
        debug_assert!(y < self.height, "Frame buffer accessed out of bounds");
        unsafe { pixel.write_volatile(self.base.add((y * self.width + x) * T::size())) }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> T {
        debug_assert!(x < self.width, "Frame buffer accessed out of bounds");
        debug_assert!(y < self.height, "Frame buffer accessed out of bounds");
        unsafe { T::read_volatile(self.base.add((y * self.height + x) * T::size())) }
    }

    pub fn get_foreground(&self) -> T {
        self.foreground
    }

    pub fn set_foreground(&mut self, pixel: T) {
        self.foreground = pixel
    }

    pub fn get_background(&self) -> T {
        self.background
    }

    pub fn set_background(&mut self, pixel: T) {
        self.background = pixel
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get_base(&self)->*mut u8{
        self.base
    }
}

#[derive(Copy, Clone)]
pub struct Rect {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Rect {
        debug_assert!(width > 0 && height > 0);
        Rect {
            x,
            y,
            width,
            height,
        }
    }
    pub fn right(&self) -> usize {
        self.x + self.width - 1
    }
    pub fn left(&self) -> usize {
        self.x
    }
    pub fn top(&self) -> usize {
        self.y
    }
    pub fn bottom(&self) -> usize {
        self.y + self.height - 1
    }
}
