use std::error::Error;
use tracing::level_filters::LevelFilter;
use winit::event_loop::EventLoop;

pub mod core;
pub mod render_context;

mod renderer;
pub use renderer::*;

mod bindless;
pub use bindless::*;

mod texture;
pub use texture::*;

mod mesh;
pub use mesh::*;

mod load_gltf;
pub use load_gltf::*;

mod vfs;
pub use vfs::*;

mod app;
pub use app::*;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut App::default())?;

    Ok(())
}
