pub trait Pixel: Sized + Copy + Clone {
    fn size() -> usize {
        core::mem::size_of::<Self>()
    }
    fn new(r: u8, g: u8, b: u8, a: u8) -> Self;
    unsafe fn write_volatile(&self, ptr: *mut u8);
    unsafe fn read_volatile(ptr: *mut u8) -> Self;
}

macro_rules! define_pixel {
    ($name:ident,$repr:ident,$new:expr) => {
        #[derive(Copy, Clone)]
        pub struct $name($repr);
        impl Pixel for $name {
            fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
                $name($repr::from_le_bytes($new(r, g, b, a)))
            }
            unsafe fn write_volatile(&self, ptr: *mut u8) {
                (ptr as *mut $repr).write_volatile(self.0)
            }
            unsafe fn read_volatile(ptr: *mut u8) -> Self {
                $name((ptr as *mut $repr).read_volatile())
            }
        }
    };
}

// SDL
define_pixel!(RGBA8888, u32, |r, g, b, a| [a, b, g, r]);
// UEFI
define_pixel!(XRGB8888, u32, |x, r, g, b| [b, g, r, x]);
// TFT Display
define_pixel!(RGB565, u16, |r, g, b, _a| [
    r & 0b1111_1000 | g & 0b0000_0111,
    g & 0b1110_0000 | b & 0b0001_1111,
]);
