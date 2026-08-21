//! Minimal vector and matrix math for a 3D pipeline. No external crate.
// Explicit index loops read closer to the standard matrix-math notation here,
// and `add`/`sub` are intentional inherent methods on the vector types.
#![allow(clippy::needless_range_loop, clippy::should_implement_trait)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    pub fn zero() -> Self {
        Vec3::new(0.0, 0.0, 0.0)
    }

    pub fn add(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }

    pub fn sub(self, o: Vec3) -> Vec3 {
        Vec3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }

    pub fn scale(self, s: f32) -> Vec3 {
        Vec3::new(self.x * s, self.y * s, self.z * s)
    }

    pub fn dot(self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len == 0.0 {
            self
        } else {
            self.scale(1.0 / len)
        }
    }
}

/// A 4x4 matrix, row-major, applied to column vectors (m * v).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Mat4 {
    pub fn identity() -> Self {
        let mut m = [[0.0; 4]; 4];
        for i in 0..4 {
            m[i][i] = 1.0;
        }
        Mat4 { m }
    }

    pub fn mul(&self, o: &Mat4) -> Mat4 {
        let mut r = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.m[i][k] * o.m[k][j];
                }
                r[i][j] = sum;
            }
        }
        Mat4 { m: r }
    }

    /// Transforms a point (w=1) and returns the homogeneous result (x, y, z, w).
    pub fn mul_point(&self, p: Vec3) -> [f32; 4] {
        let v = [p.x, p.y, p.z, 1.0];
        let mut r = [0.0; 4];
        for i in 0..4 {
            r[i] = self.m[i][0] * v[0] + self.m[i][1] * v[1] + self.m[i][2] * v[2] + self.m[i][3] * v[3];
        }
        r
    }

    pub fn translation(t: Vec3) -> Mat4 {
        let mut m = Mat4::identity();
        m.m[0][3] = t.x;
        m.m[1][3] = t.y;
        m.m[2][3] = t.z;
        m
    }

    pub fn rotation_y(radians: f32) -> Mat4 {
        let (s, c) = radians.sin_cos();
        let mut m = Mat4::identity();
        m.m[0][0] = c;
        m.m[0][2] = s;
        m.m[2][0] = -s;
        m.m[2][2] = c;
        m
    }

    pub fn rotation_x(radians: f32) -> Mat4 {
        let (s, c) = radians.sin_cos();
        let mut m = Mat4::identity();
        m.m[1][1] = c;
        m.m[1][2] = -s;
        m.m[2][1] = s;
        m.m[2][2] = c;
        m
    }

    /// Right-handed look-at view matrix.
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        let f = target.sub(eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(f);

        let mut m = Mat4::identity();
        m.m[0] = [s.x, s.y, s.z, -s.dot(eye)];
        m.m[1] = [u.x, u.y, u.z, -u.dot(eye)];
        m.m[2] = [-f.x, -f.y, -f.z, f.dot(eye)];
        m.m[3] = [0.0, 0.0, 0.0, 1.0];
        m
    }

    /// Right-handed perspective projection with fov in radians.
    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / (fov_y / 2.0).tan();
        let mut m = [[0.0; 4]; 4];
        m[0][0] = f / aspect;
        m[1][1] = f;
        m[2][2] = (far + near) / (near - far);
        m[2][3] = (2.0 * far * near) / (near - far);
        m[3][2] = -1.0;
        Mat4 { m }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_leaves_point_unchanged() {
        let p = Vec3::new(1.0, 2.0, 3.0);
        let r = Mat4::identity().mul_point(p);
        assert_eq!(r, [1.0, 2.0, 3.0, 1.0]);
    }

    #[test]
    fn translation_moves_point() {
        let p = Vec3::new(0.0, 0.0, 0.0);
        let r = Mat4::translation(Vec3::new(1.0, 2.0, 3.0)).mul_point(p);
        assert_eq!(r, [1.0, 2.0, 3.0, 1.0]);
    }

    #[test]
    fn cross_product_orthogonal() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        let z = x.cross(y);
        assert_eq!(z, Vec3::new(0.0, 0.0, 1.0));
    }
}
