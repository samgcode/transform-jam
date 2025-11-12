use godot::prelude::*;

mod game_controller;
mod grenade;
mod player;
mod sdf_controller;

struct RustExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RustExtension {}
