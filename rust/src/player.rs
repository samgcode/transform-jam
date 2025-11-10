use godot::prelude::*;

pub struct Player {
  pub position: Vector3,
  pub velocity: Vector3,
}

impl Player {
  pub fn init() -> Self {
    return Self {
      position: Vector3::new(0.0, 0.0, -2.0),
      velocity: Vector3::new(0.0, 0.0, 0.0),
    };
  }

  pub fn update(&mut self, dt: f32, gravity: Vector3) {
    self.position += self.velocity * dt;
    self.velocity += gravity;
  }
}
