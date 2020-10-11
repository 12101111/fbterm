#[cfg(feature = "alloc")]
pub(crate) mod truetype;

pub(crate) mod vga;

#[cfg(feature = "alloc")]
use alloc::sync::Arc;
use core::ops::Deref;

pub enum Point {
    Bit(bool),
    Coverage(u8),
}

pub trait Font {
    fn height(&self) -> usize;
    fn get_glyph(&mut self, c: char) -> Option<Glyph>;
    fn get_pixel(&self, glyph: &Glyph, x: usize, y: usize) -> Point;
}

#[derive(Clone, Debug)]
pub struct Glyph {
    pub width: usize,
    pub height: usize,
    pub advance: usize,
    pub x: usize,
    pub y: isize,
    pub data: Cow,
}

#[derive(Clone, Debug)]
pub enum Cow {
    Borrowed(&'static [u8]),

    #[cfg(feature = "alloc")]
    Arc(Arc<[u8]>),
}

impl From<&'static [u8]> for Cow {
    fn from(val: &'static [u8]) -> Cow {
        Cow::Borrowed(val)
    }
}

#[cfg(feature = "alloc")]
impl From<Arc<[u8]>> for Cow {
    fn from(val: Arc<[u8]>) -> Cow {
        Cow::Arc(val)
    }
}

impl Deref for Cow {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self {
            #[cfg(feature = "alloc")]
            Cow::Arc(arc) => arc.as_ref(),
            Cow::Borrowed(slice) => slice,
        }
    }
}
