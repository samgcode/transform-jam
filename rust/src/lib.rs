use godot::prelude::*;

mod sdf_controller;

struct RustExtension;

#[gdextension]
unsafe impl ExtensionLibrary for RustExtension {}
