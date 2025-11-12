use godot::prelude::*;

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
  pub grenade_id: i32,
  pub exploded: bool,
  destroyed: bool,
}

#[godot_api]
impl INode3D for Grenade {
  fn init(base: Base<Node3D>) -> Self {
    return Self {
      base,
      velocity: Vector3::ZERO,
      grenade_id: 0,
      exploded: false,
      destroyed: false,
    };
  }

  fn ready(&mut self) {
    self.signals().collision().connect_self(Self::on_collision);
  }

  fn physics_process(&mut self, dt: f64) {
    self.add_position(self.velocity * dt as f32);

    let position = self.base().get_transform().origin;

    if self.destroyed {
      self.base_mut().queue_free();
    }

    if position.length() > 50.0 {
      self.exploded = true;
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

  fn on_collision(&mut self, _collision: Vector4) {
    self.exploded = true;
  }

  pub fn destroy(&mut self) {
    self.destroyed = true;
  }
}

#[godot_api]
impl Grenade {
  #[signal]
  pub fn collision(collision: Vector4);
}
