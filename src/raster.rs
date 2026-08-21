//! Filled-triangle rasterization via barycentric coordinates with a per-pixel depth test.

use crate::framebuffer::Framebuffer;

/// A screen-space vertex: x, y in pixels, z in the depth range used by the z-buffer.
#[derive(Debug, Clone, Copy)]
pub struct ScreenVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

fn edge(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0)
}

/// Rasterizes a triangle with a single flat color, testing depth per pixel.
pub fn fill_triangle(fb: &mut Framebuffer, v0: ScreenVertex, v1: ScreenVertex, v2: ScreenVertex, color: [u8; 3]) {
    let a = (v0.x, v0.y);
    let b = (v1.x, v1.y);
    let c = (v2.x, v2.y);

    let area = edge(a, b, c);
    if area == 0.0 {
        return;
    }

    let min_x = a.0.min(b.0).min(c.0).floor().max(0.0) as usize;
    let max_x = (a.0.max(b.0).max(c.0).ceil() as isize).min(fb.width as isize - 1);
    let min_y = a.1.min(b.1).min(c.1).floor().max(0.0) as usize;
    let max_y = (a.1.max(b.1).max(c.1).ceil() as isize).min(fb.height as isize - 1);

    if max_x < 0 || max_y < 0 {
        return;
    }

    for y in min_y..=(max_y as usize) {
        for x in min_x..=(max_x as usize) {
            let p = (x as f32 + 0.5, y as f32 + 0.5);
            let w0 = edge(b, c, p);
            let w1 = edge(c, a, p);
            let w2 = edge(a, b, p);

            let inside = (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0);
            if !inside {
                continue;
            }

            let l0 = w0 / area;
            let l1 = w1 / area;
            let l2 = w2 / area;
            let z = l0 * v0.z + l1 * v1.z + l2 * v2.z;

            fb.set_with_depth(x, y, z, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_interior_and_leaves_exterior_clear() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear([0, 0, 0]);
        let v0 = ScreenVertex { x: 1.0, y: 1.0, z: 0.5 };
        let v1 = ScreenVertex { x: 8.0, y: 1.0, z: 0.5 };
        let v2 = ScreenVertex { x: 1.0, y: 8.0, z: 0.5 };
        fill_triangle(&mut fb, v0, v1, v2, [200, 100, 50]);

        // Interior point, well inside the right triangle.
        assert_eq!(fb.get(2, 2), [200, 100, 50]);
        // Exterior point, past the hypotenuse.
        assert_eq!(fb.get(9, 9), [0, 0, 0]);
        // Exterior corner, outside the bounding triangle entirely.
        assert_eq!(fb.get(0, 0), [0, 0, 0]);
    }

    #[test]
    fn nearer_triangle_wins_the_overlap() {
        let mut fb = Framebuffer::new(10, 10);
        fb.clear([0, 0, 0]);
        let far = (
            ScreenVertex { x: 0.0, y: 0.0, z: 5.0 },
            ScreenVertex { x: 9.0, y: 0.0, z: 5.0 },
            ScreenVertex { x: 0.0, y: 9.0, z: 5.0 },
        );
        let near = (
            ScreenVertex { x: 0.0, y: 0.0, z: 1.0 },
            ScreenVertex { x: 9.0, y: 0.0, z: 1.0 },
            ScreenVertex { x: 0.0, y: 9.0, z: 1.0 },
        );

        fill_triangle(&mut fb, far.0, far.1, far.2, [255, 0, 0]);
        fill_triangle(&mut fb, near.0, near.1, near.2, [0, 255, 0]);
        assert_eq!(fb.get(2, 2), [0, 255, 0]);
        assert_eq!(fb.get_depth(2, 2), 1.0);

        // Drawing the far triangle again afterwards must not overwrite the nearer pixel.
        fill_triangle(&mut fb, far.0, far.1, far.2, [255, 0, 0]);
        assert_eq!(fb.get(2, 2), [0, 255, 0]);
    }
}
