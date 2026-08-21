//! A minimal demo scene: a cube, plus the model/view/projection pipeline that
//! turns its triangles into shaded, screen-space triangles ready for rasterization.

use crate::framebuffer::Framebuffer;
use crate::raster::{fill_triangle, ScreenVertex};
use crate::vec::{Mat4, Vec3};

struct Triangle {
    a: Vec3,
    b: Vec3,
    c: Vec3,
}

fn cube_triangles() -> Vec<Triangle> {
    let p = [
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, -1.0, -1.0),
        Vec3::new(1.0, 1.0, -1.0),
        Vec3::new(-1.0, 1.0, -1.0),
        Vec3::new(-1.0, -1.0, 1.0),
        Vec3::new(1.0, -1.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(-1.0, 1.0, 1.0),
    ];
    let faces: [[usize; 4]; 6] = [
        [0, 1, 2, 3], // back
        [5, 4, 7, 6], // front
        [4, 0, 3, 7], // left
        [1, 5, 6, 2], // right
        [3, 2, 6, 7], // top
        [4, 5, 1, 0], // bottom
    ];
    let mut tris = Vec::new();
    for f in faces {
        tris.push(Triangle { a: p[f[0]], b: p[f[1]], c: p[f[2]] });
        tris.push(Triangle { a: p[f[0]], b: p[f[2]], c: p[f[3]] });
    }
    tris
}

fn to_screen(clip: [f32; 4], width: usize, height: usize) -> ScreenVertex {
    let ndc_x = clip[0] / clip[3];
    let ndc_y = clip[1] / clip[3];
    let ndc_z = clip[2] / clip[3];
    ScreenVertex {
        x: (ndc_x * 0.5 + 0.5) * width as f32,
        y: (1.0 - (ndc_y * 0.5 + 0.5)) * height as f32,
        z: ndc_z,
    }
}

/// Renders the built-in demo scene (a rotated, lit cube) into a fresh framebuffer.
pub fn render_demo(width: usize, height: usize) -> Framebuffer {
    let mut fb = Framebuffer::new(width, height);
    fb.clear([20, 20, 30]);

    let model = Mat4::rotation_y(0.6).mul(&Mat4::rotation_x(0.35));
    let view = Mat4::look_at(Vec3::new(0.0, 0.0, 5.0), Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
    let proj = Mat4::perspective(std::f32::consts::FRAC_PI_4, width as f32 / height as f32, 0.1, 100.0);
    let mvp = proj.mul(&view).mul(&model);

    let light_dir = Vec3::new(0.4, 0.6, 1.0).normalize();

    for tri in cube_triangles() {
        let world_normal = model_normal(&model, tri.a, tri.b, tri.c);
        let intensity = world_normal.dot(light_dir).max(0.05);
        let base = 220.0;
        let shade = (base * intensity) as u8;
        let color = [shade, shade, (shade as f32 * 0.9) as u8];

        let ca = mvp.mul_point(tri.a);
        let cb = mvp.mul_point(tri.b);
        let cc = mvp.mul_point(tri.c);

        let sa = to_screen(ca, width, height);
        let sb = to_screen(cb, width, height);
        let sc = to_screen(cc, width, height);

        fill_triangle(&mut fb, sa, sb, sc, color);
    }

    fb
}

fn model_normal(model: &Mat4, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ta = point3(model.mul_point(a));
    let tb = point3(model.mul_point(b));
    let tc = point3(model.mul_point(c));
    tb.sub(ta).cross(tc.sub(ta)).normalize()
}

fn point3(p: [f32; 4]) -> Vec3 {
    Vec3::new(p[0], p[1], p[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_scene_produces_expected_dimensions() {
        let fb = render_demo(64, 48);
        assert_eq!(fb.width, 64);
        assert_eq!(fb.height, 48);
        assert_eq!(fb.pixels.len(), 64 * 48);
    }

    #[test]
    fn demo_scene_draws_something_over_the_clear_color() {
        let fb = render_demo(64, 48);
        let clear = [20u8, 20, 30];
        let drawn = fb.pixels.iter().any(|p| *p != clear);
        assert!(drawn);
    }
}
