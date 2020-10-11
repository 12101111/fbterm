pub trait Pixel: Sized + Copy + Clone {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Self;
    fn size() -> usize {
        core::mem::size_of::<Self>()
    }
    fn get(&self) -> (u8, u8, u8, u8);
    unsafe fn write_volatile(&self, ptr: *mut u8) {
        (ptr as *mut Self).write_volatile(*self)
    }
    unsafe fn read_volatile(ptr: *mut u8) -> Self {
        (ptr as *mut Self).read_volatile()
    }
}

/// Pixel for UEFI or SDL2
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct RGBA8888 {
    pub a: u8,
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl Pixel for RGBA8888 {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        RGBA8888 { a, b, g, r }
    }
    fn get(&self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }
}
