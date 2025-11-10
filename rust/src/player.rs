use godot::prelude::*;

pub struct Player {
  pub position: Vector3,
  pub velocity: Vector3,
}

impl Player {
  pub fn init() -> Self {
    return Self {
      position: Vector3::new(0.5, 0.0, -1.0),
      velocity: Vector3::new(0.0, -0.5, 0.0),
    };
  }

  pub fn update(&mut self, dt: f32, gravity: Vector3) {
    self.position += self.velocity * dt;
    self.velocity += gravity;
  }

  pub fn on_collision(&mut self, collision: Vector4, _dt: f32) {
    let normal = Vector3::new(collision.x, collision.y, collision.z).normalized();

    let parallel = Vector3::dot(self.velocity, normal) * normal;
    let perpendicular: Vector3 = self.velocity - parallel;

    self.position += normal * -collision.w;
    self.velocity = perpendicular * 0.95;
  }
}
