use std::error::Error;
use tracing::level_filters::LevelFilter;
use winit::event_loop::{ControlFlow, EventLoop};

pub mod core;
pub mod render_context;

mod game_renderer;
pub use game_renderer::*;

mod car;
pub use car::*;

mod track;
pub use track::*;

mod renderer;
pub use renderer::*;

mod physics;
pub use physics::*;

mod bindless;
pub use bindless::*;

mod texture;
pub use texture::*;

mod mesh;
pub use mesh::*;

mod load_gltf;
pub use load_gltf::*;

mod camera;
pub use camera::*;

mod vfs;
pub use vfs::*;

mod app;
pub use app::*;

mod audio;
pub use audio::play_music;


fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(LevelFilter::ERROR)
        .init();

    mount_vfs("./assets/shaders/spv");
    mount_vfs("");
    mount_vfs("../");
    mount_vfs("../../");
    mount_vfs("./assets/sounds");
    mount_vfs("./assets/meshes");
    mount_vfs("./assets/shaders");
    mount_vfs("./assets/textures");
    mount_vfs("./assets/fonts");
    mount_vfs("./assets");
    mount_vfs("./");
    mount_vfs(".");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::default())?;
    
    Ok(())
}
