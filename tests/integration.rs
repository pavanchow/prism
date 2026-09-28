//! Public-API integration tests for prism, the from-scratch software 3D rasterizer.
//! Exercises the crate the way a downstream consumer would: the re-exported
//! `Framebuffer` / `render_demo`, plus the public `vec`, `raster`, and `framebuffer`
//! modules.

use prism::framebuffer::Framebuffer;
use prism::raster::{fill_triangle, ScreenVertex};
use prism::vec::{Mat4, Vec3};
use prism::render_demo;

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

// ---- core render round-trip ----------------------------------------------

#[test]
fn render_demo_round_trips_through_ppm() {
    let fb = render_demo(64, 48);
    assert_eq!(fb.width, 64);
    assert_eq!(fb.height, 48);

    let ppm = fb.to_ppm();
    // P6 header for a 64x48 image, then exactly one RGB triple per pixel.
    let header = b"P6\n64 48\n255\n";
    assert_eq!(&ppm[..header.len()], header);
    assert_eq!(ppm.len(), header.len() + 64 * 48 * 3);

    // Parse the pixel body back out and confirm it matches the framebuffer.
    let body = &ppm[header.len()..];
    for (i, px) in fb.pixels.iter().enumerate() {
        assert_eq!(&body[i * 3..i * 3 + 3], &px[..]);
    }
}

#[test]
fn render_demo_is_deterministic() {
    let a = render_demo(80, 60);
    let b = render_demo(80, 60);
    assert_eq!(a.pixels, b.pixels);
    assert_eq!(a.depth, b.depth);
}

#[test]
fn render_demo_draws_geometry_over_the_clear_color() {
    let fb = render_demo(96, 72);
    let clear = [20u8, 20, 30];
    let drawn = fb.pixels.iter().filter(|p| **p != clear).count();
    // A lit cube filling much of the frame should cover a meaningful fraction.
    assert!(drawn > 96 * 72 / 10, "only {drawn} pixels drawn");
    // Depth buffer must have advanced from +inf wherever we drew.
    let advanced = fb.depth.iter().filter(|d| d.is_finite()).count();
    assert_eq!(advanced, drawn);
}

// ---- framebuffer edge cases -----------------------------------------------

#[test]
fn minimum_size_framebuffer_writes_its_single_pixel() {
    let mut fb = Framebuffer::new(1, 1);
    fb.clear([0, 0, 0]);
    let full = ScreenVertex { x: 0.5, y: 0.5, z: 0.5 };
    // A tiny triangle around the pixel center paints the lone pixel.
    fill_triangle(
        &mut fb,
        ScreenVertex { x: -1.0, y: -1.0, z: 0.5 },
        ScreenVertex { x: 3.0, y: -1.0, z: 0.5 },
        ScreenVertex { x: -1.0, y: 3.0, z: 0.5 },
        [7, 8, 9],
    );
    let _ = full;
    assert_eq!(fb.get(0, 0), [7, 8, 9]);
    assert_eq!(fb.to_ppm().len(), b"P6\n1 1\n255\n".len() + 3);
}

#[test]
fn depth_test_keeps_the_nearer_fragment() {
    let mut fb = Framebuffer::new(1, 1);
    assert!(fb.set_with_depth(0, 0, 2.0, [255, 0, 0]));
    // Farther fragment is rejected.
    assert!(!fb.set_with_depth(0, 0, 3.0, [0, 255, 0]));
    assert_eq!(fb.get(0, 0), [255, 0, 0]);
    // Nearer fragment wins.
    assert!(fb.set_with_depth(0, 0, 1.0, [0, 0, 255]));
    assert_eq!(fb.get(0, 0), [0, 0, 255]);
    assert!(approx(fb.get_depth(0, 0), 1.0));
}

#[test]
fn clear_resets_color_and_depth_together() {
    let mut fb = Framebuffer::new(3, 2);
    fb.set_with_depth(1, 1, 0.5, [1, 2, 3]);
    fb.clear([9, 9, 9]);
    for y in 0..2 {
        for x in 0..3 {
            assert_eq!(fb.get(x, y), [9, 9, 9]);
            assert_eq!(fb.get_depth(x, y), f32::INFINITY);
        }
    }
}

// ---- rasterizer: fill, bounds, depth ordering -----------------------------

#[test]
fn full_cover_triangle_paints_every_pixel_no_off_by_one() {
    // A triangle that comfortably covers a whole 16x16 buffer must leave no
    // pixel unpainted at any edge (guards min/max_x, min/max_y clamping).
    let (w, h) = (16usize, 16usize);
    let mut fb = Framebuffer::new(w, h);
    fb.clear([0, 0, 0]);
    fill_triangle(
        &mut fb,
        ScreenVertex { x: -10.0, y: -10.0, z: 0.5 },
        ScreenVertex { x: (w as f32) * 3.0, y: -10.0, z: 0.5 },
        ScreenVertex { x: -10.0, y: (h as f32) * 3.0, z: 0.5 },
        [200, 100, 50],
    );
    let unpainted = fb.pixels.iter().filter(|p| **p == [0, 0, 0]).count();
    assert_eq!(unpainted, 0, "{unpainted} pixels left unpainted at edges");
}

#[test]
fn interior_filled_exterior_untouched() {
    let mut fb = Framebuffer::new(10, 10);
    fb.clear([0, 0, 0]);
    fill_triangle(
        &mut fb,
        ScreenVertex { x: 1.0, y: 1.0, z: 0.5 },
        ScreenVertex { x: 8.0, y: 1.0, z: 0.5 },
        ScreenVertex { x: 1.0, y: 8.0, z: 0.5 },
        [10, 20, 30],
    );
    assert_eq!(fb.get(2, 2), [10, 20, 30]); // inside
    assert_eq!(fb.get(9, 9), [0, 0, 0]); // past hypotenuse
}

#[test]
fn degenerate_zero_area_triangle_draws_nothing() {
    let mut fb = Framebuffer::new(8, 8);
    fb.clear([0, 0, 0]);
    // Three collinear vertices => zero signed area => early return.
    fill_triangle(
        &mut fb,
        ScreenVertex { x: 0.0, y: 0.0, z: 0.5 },
        ScreenVertex { x: 4.0, y: 4.0, z: 0.5 },
        ScreenVertex { x: 7.0, y: 7.0, z: 0.5 },
        [255, 255, 255],
    );
    assert!(fb.pixels.iter().all(|p| *p == [0, 0, 0]));
}

#[test]
fn nearer_triangle_wins_regardless_of_draw_order() {
    let mut fb = Framebuffer::new(10, 10);
    fb.clear([0, 0, 0]);
    let tri = |z: f32| {
        (
            ScreenVertex { x: 0.0, y: 0.0, z },
            ScreenVertex { x: 9.0, y: 0.0, z },
            ScreenVertex { x: 0.0, y: 9.0, z },
        )
    };
    let (fa, fb2, fc) = tri(5.0);
    let (na, nb, nc) = tri(1.0);
    fill_triangle(&mut fb, na, nb, nc, [0, 255, 0]);
    // Drawing the farther triangle afterward must not overwrite the nearer one.
    fill_triangle(&mut fb, fa, fb2, fc, [255, 0, 0]);
    assert_eq!(fb.get(2, 2), [0, 255, 0]);
    assert!(approx(fb.get_depth(2, 2), 1.0));
}

// ---- vector / matrix math invariants --------------------------------------

#[test]
fn cross_product_is_orthogonal_and_right_handed() {
    let x = Vec3::new(1.0, 0.0, 0.0);
    let y = Vec3::new(0.0, 1.0, 0.0);
    let z = x.cross(y);
    assert_eq!(z, Vec3::new(0.0, 0.0, 1.0));
    assert!(approx(z.dot(x), 0.0));
    assert!(approx(z.dot(y), 0.0));
    assert!(approx(x.normalize().length(), 1.0));
}

#[test]
fn identity_and_translation_transform_points_correctly() {
    let p = Vec3::new(1.0, 2.0, 3.0);
    assert_eq!(Mat4::identity().mul_point(p), [1.0, 2.0, 3.0, 1.0]);

    let t = Mat4::translation(Vec3::new(10.0, -5.0, 2.0));
    assert_eq!(t.mul_point(p), [11.0, -3.0, 5.0, 1.0]);

    // identity is a left/right multiplication neutral element.
    let m = Mat4::rotation_y(0.7).mul(&Mat4::rotation_x(0.3));
    assert_eq!(m.mul(&Mat4::identity()), m);
    assert_eq!(Mat4::identity().mul(&m), m);
}

#[test]
fn rotation_y_by_half_pi_maps_x_axis_onto_negative_z() {
    let r = Mat4::rotation_y(std::f32::consts::FRAC_PI_2);
    let out = r.mul_point(Vec3::new(1.0, 0.0, 0.0));
    // With this matrix, +X rotates toward -Z.
    assert!(approx(out[0], 0.0));
    assert!(approx(out[1], 0.0));
    assert!(approx(out[2], -1.0));
    assert!(approx(out[3], 1.0));
}
