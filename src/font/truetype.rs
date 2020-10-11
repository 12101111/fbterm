use super::{Cow, Font, Glyph, Point};
use alloc::sync::Arc;

pub struct TrueTypeFont {
    inner: fontdue::Font,
    cache: lru::LruCache<char, Glyph>,
    line_size: usize,
    height: usize,
    size: f32,
}

impl TrueTypeFont {
    pub fn new(data: &[u8], size: f32) -> TrueTypeFont {
        let settings = fontdue::FontSettings {
            scale: size,
            ..fontdue::FontSettings::default()
        };
        let font = fontdue::Font::from_bytes(data, settings).unwrap();
        let line = font
            .vertical_line_metrics(size)
            .unwrap_or(font.horizontal_line_metrics(size).unwrap());
        let height = (line.new_line_size * 1.25) as usize;
        TrueTypeFont {
            inner: font,
            cache: lru::LruCache::new(128),
            line_size: line.new_line_size as usize,
            height,
            size,
        }
    }
}

impl Font for TrueTypeFont {
    #[inline]
    fn height(&self) -> usize {
        self.height
    }

    fn get_glyph(&mut self, c: char) -> Option<Glyph> {
        let cache = self.cache.get(&c);
        match cache {
            Some(glyph) => glyph.clone().into(),
            None => {
                let (metrics, data) = self.inner.rasterize(c, self.size);
                assert!(metrics.xmin >= 0);
                assert!(metrics.xmin as usize + metrics.width <= metrics.advance_width as usize);
                let data = Cow::Arc(Arc::from(data));
                let glyph = Glyph {
                    data,
                    width: metrics.width,
                    advance: metrics.advance_width as usize,
                    height: metrics.height,
                    x: metrics.xmin as usize,
                    y: self.line_size as isize - metrics.ymin as isize - metrics.height as isize,
                };
                self.cache.put(c, glyph.clone());
                Some(glyph)
            }
        }
    }

    #[inline]
    fn get_pixel(&self, glyph: &Glyph, x: usize, y: usize) -> Point {
        Point::Coverage(glyph.data[y * glyph.width + x])
    }
}
