//! Prism: a from-scratch software 3D rasterizer. No GPU, no graphics crates.

pub mod framebuffer;
pub mod raster;
pub mod scene;
pub mod vec;

pub use framebuffer::Framebuffer;
pub use scene::render_demo;
