use godot::prelude::*;

mod player;
mod sdf_controller;

struct RustExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RustExtension {}
