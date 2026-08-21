# Design

## Framebuffer

`Framebuffer` (`src/framebuffer.rs`) holds two flat arrays sized `width * height`: `pixels`, an `[u8; 3]` RGB value per pixel, and `depth`, an `f32` per pixel initialized to infinity. Writing a pixel goes through `set_with_depth`, which only commits the color if the incoming depth is smaller than what is already stored. This single function is the entire hidden-surface algorithm.

## Barycentric rasterization

A triangle is filled by first computing its axis-aligned bounding box in screen space, clipped to the framebuffer. For every pixel center in that box, `raster.rs` computes three edge functions:

```
edge(a, b, p) = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
```

one per triangle edge. If all three share the same sign (or are zero), the point lies inside the triangle, regardless of winding order. Dividing each edge value by the total triangle area gives the barycentric weights `l0, l1, l2`, which sum to 1 and interpolate any per-vertex quantity, here just depth, linearly across the face.

## Z-buffer

Depth is interpolated in the same barycentric pass and compared against the z-buffer entry for that pixel before writing color. Because the test happens per pixel rather than per triangle, two overlapping triangles resolve correctly regardless of the order they are drawn in.

## Transform pipeline

Vertices in `src/scene.rs` move through four spaces:

1. **Model space**: vertex positions as authored (a unit cube centered at the origin).
2. **World space**: rotation matrices (`Mat4::rotation_x`, `Mat4::rotation_y`) place and orient the model.
3. **Camera space**: `Mat4::look_at` builds a view matrix from an eye position, a target, and an up vector.
4. **Clip space**: `Mat4::perspective` applies a right-handed perspective projection with a given field of view, aspect ratio, and near/far planes.

The three matrices are combined once (`proj * view * model`) and applied to every vertex. After the multiply, `x/w, y/w, z/w` gives normalized device coordinates in `[-1, 1]`, which are then mapped linearly to pixel coordinates and to the z-buffer's depth range.

Face normals for shading are computed in world space (after the model matrix, before view/projection) so the light direction stays fixed in world terms regardless of camera position.

## PPM format

The output is PPM in binary mode (P6): a text header `P6\n<width> <height>\n255\n` followed by exactly `width * height * 3` raw bytes, one RGB triple per pixel, row-major from top to bottom. No compression, no metadata, trivially checkable by byte length alone.
