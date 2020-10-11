use crate::pixel::*;
use core::marker::PhantomData;
use core::ptr::NonNull;
pub struct Framebuffer<'a, T: Pixel> {
    base: NonNull<u8>,
    buffer: Option<NonNull<u8>>,
    width: usize,
    height: usize,
    stride: usize,
    background: T,
    foreground: T,
    _lifetime: PhantomData<&'a u8>,
}

impl<'a, T: Pixel> Framebuffer<'a, T> {
    pub unsafe fn new(
        base: NonNull<u8>,
        width: usize,
        height: usize,
        stride: usize,
        background: T,
        foreground: T,
    ) -> Framebuffer<'a, T> {
        Framebuffer {
            base,
            buffer: None,
            width,
            height,
            stride,
            background,
            foreground,
            _lifetime: PhantomData,
        }
    }

    pub unsafe fn set_double_buffer(&mut self, buffer: NonNull<u8>) {
        let real_buffer = self.base;
        self.base = buffer;
        self.buffer = Some(real_buffer);
    }

    pub fn flush(&mut self, rect: Option<Rect>) {
        if self.buffer.is_none() {
            return;
        }
        let real_buffer = self.buffer.unwrap();
        let rect = rect.unwrap_or(Rect::new(0, 0, self.width, self.height));
        assert!(
            rect.right() < self.width && rect.bottom() < self.height,
            "Rect is out of bounds: {:?}",
            rect
        );
        for y in rect.y..(rect.y + rect.height) {
            let index = (y * self.stride + rect.x) * T::size();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.base.as_ptr().add(index),
                    real_buffer.as_ptr().add(index),
                    rect.width * T::size(),
                );
            }
        }
    }

    #[inline]
    pub unsafe fn read(&self, index: usize) -> T {
        T::read_volatile(self.base.as_ptr().add(index * T::size()))
    }

    #[inline]
    pub unsafe fn write(&mut self, index: usize, val: T) {
        val.write_volatile(self.base.as_ptr().add(index * T::size()))
    }

    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> T {
        assert!(x < self.width, "Frame buffer accessed out of bounds");
        assert!(y < self.height, "Frame buffer accessed out of bounds");
        unsafe { self.read(y * self.stride + x) }
    }

    #[inline]
    pub unsafe fn draw_pixel(&mut self, x: usize, y: usize, pixel: T) {
        self.write(y * self.stride + x, pixel)
    }

    pub fn clear(&mut self) {
        for i in 0..self.height * self.stride {
            unsafe { self.write(i, self.background) }
        }
    }

    pub fn copy_rect(&mut self, src: Rect, dst: Rect) {
        assert!(src.top() != dst.top(), "can't copy to self"); // copy_nonoverlapping
        assert_eq!(
            src.width, dst.width,
            "The width of the source and target are different: src: {:?} != dst: {:?}",
            src, dst
        );
        assert_eq!(
            src.height, dst.height,
            "The height of the source and target are different: src: {:?} != dst: {:?}",
            src, dst
        );
        assert!(
            src.right() < self.width && src.bottom() < self.height,
            "source Rect is out of bounds: {:?}",
            src
        );
        assert!(
            dst.right() < self.width && dst.bottom() < self.height,
            "target Rect is out of bounds: {:?}",
            dst
        );
        for y in 0..src.height {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.base
                        .as_ptr()
                        .add(((src.y + y) * self.stride + src.x) * T::size()),
                    self.base
                        .as_ptr()
                        .add(((dst.y + y) * self.stride + dst.x) * T::size()),
                    src.width * T::size(),
                );
            }
        }
    }

    pub fn draw_rect(&mut self, dst: Rect, pixel: T) {
        assert!(
            dst.bottom() < self.height && dst.right() < self.width,
            "target Rect is out of bounds: {:?}",
            dst
        );
        for y in 0..dst.height {
            for x in 0..dst.width {
                unsafe {
                    self.write((dst.y + y) * self.stride + dst.x + x, pixel);
                }
            }
        }
    }

    #[inline]
    pub unsafe fn draw_alpha(&mut self, x: usize, y: usize, alpha: u8) {
        let fg = self.foreground.get();
        let bg = self.background.get();
        let pixel = if alpha == 0 {
            self.background
        } else if alpha == 255 {
            self.foreground
        } else {
            let map = |f: u8, b: u8, a: u8| {
                let f = f as i32;
                let b = b as i32;
                let diff = f - b;
                let add = (diff * (a as i32)) / 256i32;
                (b + add) as u8
            };
            T::new(
                map(fg.0, bg.0, alpha),
                map(fg.1, bg.1, alpha),
                map(fg.2, bg.2, alpha),
                map(fg.3, bg.3, alpha),
            )
        };
        self.draw_pixel(x, y, pixel)
    }

    #[inline]
    pub unsafe fn draw_bit(&mut self, x: usize, y: usize, bit: bool) {
        if bit {
            self.draw_pixel(x, y, self.foreground)
        } else {
            self.draw_pixel(x, y, self.background)
        }
    }

    #[inline]
    pub fn get_foreground(&self) -> T {
        self.foreground
    }

    #[inline]
    pub fn set_foreground(&mut self, pixel: T) {
        self.foreground = pixel
    }

    #[inline]
    pub fn get_background(&self) -> T {
        self.background
    }

    #[inline]
    pub fn set_background(&mut self, pixel: T) {
        self.background = pixel
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn buffer_size(&mut self) -> usize {
        self.height * self.stride * T::size()
    }

    #[inline]
    pub fn get_base(&self) -> NonNull<u8> {
        self.base
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Rect {
    #[inline]
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    #[inline]
    pub fn right(&self) -> usize {
        self.x + self.width - 1
    }

    #[inline]
    pub fn left(&self) -> usize {
        self.x
    }

    #[inline]
    pub fn top(&self) -> usize {
        self.y
    }

    #[inline]
    pub fn bottom(&self) -> usize {
        self.y + self.height - 1
    }
}
