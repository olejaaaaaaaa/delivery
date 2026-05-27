use std::error::Error;
use tracing::level_filters::LevelFilter;
use winit::event_loop::{ControlFlow, EventLoop};

pub mod core;

pub mod render_context;
pub use render_context::*;

pub mod resources;
pub use resources::*;

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

mod state;
pub use state::*;

mod load_gltf;
pub use load_gltf::*;

mod camera;
pub use camera::*;

mod vfs;
pub use vfs::*;

mod app;
pub use app::*;

mod iced;
pub use iced::*;

mod audio;
pub use audio::play_music;


fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(LevelFilter::ERROR)
        .init();

    mount_vfs("shader://","./assets/shaders/spv");
    mount_vfs("shader://", "./assets/shaders");
    mount_vfs("shader://", "./assets");
    mount_vfs("shader://", "./");
    mount_vfs("shader://", "../");
    mount_vfs("shader://", "../../");

    mount_vfs("sound://","./assets/sounds");
    mount_vfs("sound://", "./assets");
    mount_vfs("sound://", "./");
    mount_vfs("sound://", "../");
    mount_vfs("sound://", "../../");

    mount_vfs("texture://","./assets/textures");
    mount_vfs("texture://", "./assets");
    mount_vfs("texture://", "./");
    mount_vfs("texture://", "../");
    mount_vfs("texture://", "../../");

    mount_vfs("mesh://","./assets/meshes");
    mount_vfs("mesh://", "./assets");
    mount_vfs("mesh://", "./");
    mount_vfs("mesh://", "../");
    mount_vfs("mesh://", "../../");

    mount_vfs("font://","./assets/fonts");
    mount_vfs("font://", "./assets");
    mount_vfs("font://", "./");
    mount_vfs("font://", "../");
    mount_vfs("font://", "../../");

    mount_vfs("res://","./assets/shaders/spv");
    mount_vfs("res://", "./assets/shaders");
    mount_vfs("res://", "./assets/meshes");
    mount_vfs("res://", "./assets/sounds");
    mount_vfs("res://", "./assets/textures");
    mount_vfs("res://", "./assets");
    mount_vfs("res://", "./");
    mount_vfs("res://", "../");
    mount_vfs("res://", "../../");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::default())?;
    
    Ok(())
}
