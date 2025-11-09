use godot::classes::{IMeshInstance3D, MeshInstance3D, ShaderMaterial};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base = MeshInstance3D)]
pub struct SdfController {
  #[base]
  base: Base<MeshInstance3D>,
}

#[godot_api]
impl IMeshInstance3D for SdfController {
  fn init(base: Base<MeshInstance3D>) -> Self {
    Self { base }
  }

  fn physics_process(&mut self, _dt: f64) {
    let _material = self
      .base_mut()
      .get_mesh()
      .unwrap()
      .surface_get_material(0)
      .unwrap()
      .cast::<ShaderMaterial>();
  }
}
