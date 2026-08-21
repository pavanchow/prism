//! RGB framebuffer, z-buffer, and PPM (P6) output.

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[u8; 3]>,
    pub depth: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            pixels: vec![[0, 0, 0]; width * height],
            depth: vec![f32::INFINITY; width * height],
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn get(&self, x: usize, y: usize) -> [u8; 3] {
        self.pixels[self.index(x, y)]
    }

    pub fn get_depth(&self, x: usize, y: usize) -> f32 {
        self.depth[self.index(x, y)]
    }

    /// Sets a pixel if it passes the depth test (smaller z wins). Returns true if written.
    pub fn set_with_depth(&mut self, x: usize, y: usize, z: f32, color: [u8; 3]) -> bool {
        let i = self.index(x, y);
        if z < self.depth[i] {
            self.depth[i] = z;
            self.pixels[i] = color;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self, color: [u8; 3]) {
        for p in self.pixels.iter_mut() {
            *p = color;
        }
        for d in self.depth.iter_mut() {
            *d = f32::INFINITY;
        }
    }

    /// Encodes the framebuffer as a binary PPM (P6) image.
    pub fn to_ppm(&self) -> Vec<u8> {
        let header = format!("P6\n{} {}\n255\n", self.width, self.height);
        let mut out = Vec::with_capacity(header.len() + self.pixels.len() * 3);
        out.extend_from_slice(header.as_bytes());
        for p in &self.pixels {
            out.extend_from_slice(p);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ppm_header_and_length_match_dimensions() {
        let fb = Framebuffer::new(4, 3);
        let bytes = fb.to_ppm();
        let header = b"P6\n4 3\n255\n";
        assert_eq!(&bytes[..header.len()], header);
        assert_eq!(bytes.len(), header.len() + 4 * 3 * 3);
    }

    #[test]
    fn clear_resets_pixels_and_depth() {
        let mut fb = Framebuffer::new(2, 2);
        fb.set_with_depth(0, 0, 0.5, [255, 0, 0]);
        fb.clear([10, 20, 30]);
        assert_eq!(fb.get(0, 0), [10, 20, 30]);
        assert_eq!(fb.get_depth(0, 0), f32::INFINITY);
    }

    #[test]
    fn depth_test_keeps_nearer_write() {
        let mut fb = Framebuffer::new(1, 1);
        assert!(fb.set_with_depth(0, 0, 1.0, [255, 0, 0]));
        assert!(!fb.set_with_depth(0, 0, 2.0, [0, 255, 0]));
        assert_eq!(fb.get(0, 0), [255, 0, 0]);
        assert!(fb.set_with_depth(0, 0, 0.5, [0, 0, 255]));
        assert_eq!(fb.get(0, 0), [0, 0, 255]);
    }
}
