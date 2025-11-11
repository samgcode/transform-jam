use godot::prelude::*;

mod player;
mod grenade;
mod sdf_controller;

struct RustExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RustExtension {}
