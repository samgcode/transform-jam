use godot::prelude::*;

use crate::game_controller::GameController;

pub const PROPERTIES: Vector4 = Vector4 {
  x: 0.01,
  y: 0.0,
  z: 0.0,
  w: 1.0,
};

pub const COLOR: Vector4 = Vector4 {
  x: 1.0,
  y: 0.0,
  z: 1.0,
  w: 0.0,
};

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct Grenade {
  #[base]
  base: Base<Node3D>,

  velocity: Vector3,
  grenade_id: i32,
}

#[godot_api]
impl INode3D for Grenade {
  fn init(base: Base<Node3D>) -> Self {
    return Self {
      base,
      velocity: Vector3::ZERO,
      grenade_id: 0,
    };
  }

  fn physics_process(&mut self, dt: f64) {
    self.add_position(self.velocity * dt as f32);

    let position = self.base().get_transform().origin;

    if position.length() > 5.0 {
      self
        .game_controller()
        .signals()
        .remove_grenade()
        .emit(self.grenade_id);

      self.base_mut().queue_free();
      return;
    }
  }
}

impl Grenade {
  pub fn initialize(&mut self, position: Vector3, velocity: Vector3, grenade_id: i32) {
    self.add_position(position);
    self.velocity = velocity;
    self.grenade_id = grenade_id;
  }

  pub fn add_position(&mut self, offset: Vector3) {
    let mut transform = self.base().get_transform();
    transform.origin += offset;
    self.base_mut().set_transform(transform);
  }

  pub fn get_position(&self) -> Vector3 {
    return self.base().get_transform().origin;
  }

  fn game_controller(&mut self) -> Gd<GameController> {
    return self
      .base_mut()
      .get_parent()
      .unwrap()
      .cast::<GameController>();
  }
}
