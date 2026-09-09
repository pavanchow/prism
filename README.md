<img src="docs/logo.svg" alt="Prism logo" width="96">

# Prism: a software 3D rasterizer in Rust

Prism is a software rasterizer written in Rust that draws real 3D triangles on the CPU, with no GPU and no graphics crates. It is a small, from-scratch rendering pipeline of hand-written matrix transforms, barycentric rasterization, a z-buffer, and flat directional shading, turning a scene of triangles into a PPM image. Read it start to finish in one sitting as a reference for how a 3D graphics pipeline works.

**[Live demo](https://pavanchow.github.io/prism/)** · MIT licensed · written in Rust

## What it is

A small, from-scratch 3D rendering pipeline you can read start to finish in one sitting. It takes a scene of triangles, transforms them through a model, view, and projection matrix, rasterizes each one with barycentric coordinates, resolves overlaps with a z-buffer, and shades faces with a simple directional light. The output is a binary PPM image, so there is no image codec to pull in either.

## How it works

1. **Vertices** are moved from model space to world space to camera space to clip space by a chain of 4x4 matrices, written by hand in `src/vec.rs`.
2. **Projection** divides by w to land in normalized device coordinates, then maps to pixel coordinates on the screen.
3. **Rasterization** walks the bounding box of each triangle and uses barycentric coordinates to decide which pixels are inside it (`src/raster.rs`).
4. **The z-buffer** keeps, for every pixel, the depth of the nearest surface drawn so far, so triangles behind others get correctly hidden.
5. **Shading** is flat per triangle: a dot product between the face normal and a fixed light direction sets its brightness.
6. **Output** is written as PPM (P6), a plain binary format any image viewer or `convert` can open.

## Usage

```
cargo run -- render -o out.ppm
cargo run -- render -o out.ppm --width 800 --height 600
```

This renders the built-in demo scene, a rotated, lit cube, and writes it to the given path.

## Tests

```
cargo test
```

Covers triangle fill (interior pixels set, exterior pixels clear), z-buffer depth resolution (the nearer triangle wins an overlap), and PPM output (correct header and byte length for the given dimensions).

## License

MIT. By Pavan Nallamothu.
