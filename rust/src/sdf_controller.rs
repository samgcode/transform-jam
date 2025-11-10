use crate::player::Player;
use godot::classes::{IMeshInstance3D, MeshInstance3D, ShaderMaterial};
use godot::prelude::*;

const BACKGROUND: &str = "BACKGROUND_COLOR";
const POSITIONS: &str = "POSITIONS";
const PROPERTIES: &str = "PROPERTIES";
const COLORS: &str = "COLORS";

const PLAYER_ID: usize = 0;
const GRAVITY: Vector3 = Vector3 {
  x: 0.0,
  y: -0.05,
  z: 0.0,
};

#[derive(GodotClass)]
#[class(base = MeshInstance3D)]
pub struct SdfController {
  #[base]
  base: Base<MeshInstance3D>,

  background_color: ColorHsv,
  positions: PackedVector3Array,
  properties: PackedVector4Array,
  colors: PackedColorArray,

  player: Player,
}

#[godot_api]
impl IMeshInstance3D for SdfController {
  fn init(base: Base<MeshInstance3D>) -> Self {
    return Self {
      base,
      background_color: ColorHsv {
        h: 0.0,
        s: 1.0,
        v: 0.75,
        a: 1.0,
      },
      positions: PackedArray::from(&[
        Vector3::new(0.0, 0.0, -1.5),
        Vector3::new(0.0, -4.5, 0.0),
        Vector3::new(0.0, -3.0, 0.0),
      ]),
      properties: PackedArray::from(&[
        Vector4::new(0.1, 0.0, 0.0, 1.0),
        Vector4::new(3.0, 0.0, 0.0, 1.0),
        Vector4::new(5.0, 0.5, 5.0, 2.0),
      ]),
      colors: PackedArray::from(&[
        Color::from_rgb(1.0, 1.0, 1.0),
        Color::from_rgb(1.0, 0.0, 0.0),
        Color::from_rgb(0.0, 0.0, 1.0),
      ]),
      player: Player::init(),
    };
  }

  fn physics_process(&mut self, dt: f64) {
    let mut material = self
      .base_mut()
      .get_mesh()
      .unwrap()
      .surface_get_material(0)
      .unwrap()
      .cast::<ShaderMaterial>();

    self.background_color.h += dt as f32 * 0.05;
    if self.background_color.h > 1.0 {
      self.background_color.h -= 1.0;
    }

    self.player.update(dt as f32, GRAVITY);

    self.positions[PLAYER_ID] = self.player.position;

    material.set_shader_parameter(BACKGROUND, &self.background_color.to_rgb().to_variant());
    material.set_shader_parameter(POSITIONS, &self.positions.to_variant());
    material.set_shader_parameter(PROPERTIES, &self.properties.to_variant());
    material.set_shader_parameter(COLORS, &self.colors.to_variant());
  }
}
