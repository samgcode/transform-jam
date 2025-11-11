use godot::prelude::*;

use crate::sdf_controller::SdfController;

const PROPERTIES: Vector4 = Vector4 {
  x: 0.01,
  y: 0.0,
  z: 0.0,
  w: 1.0,
};
const COLOR: Vector4 = Vector4 {
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
  shape_address: usize,
}

#[godot_api]
impl INode3D for Grenade {
  fn init(base: Base<Node3D>) -> Self {
    return Self {
      base,
      velocity: Vector3::ZERO,
      shape_address: usize::MAX,
    };
  }

  fn physics_process(&mut self, dt: f64) {
    self.add_position(self.velocity * dt as f32);

    let position: Vector3 = self.base().get_transform().origin;
    let mut renderer = self.base().get_node_as::<SdfController>("../SdfController");
    if position.length() > 5.0 {
      renderer.bind_mut().remove_shape(self.shape_address);
      self.base_mut().queue_free();
      return;
    }
    renderer.bind_mut().update_shape(
      self.shape_address,
      Vector4::new(position.x, position.y, position.z, 1.0),
      PROPERTIES,
      COLOR,
    );
  }
}

impl Grenade {
  pub fn initialize(
    &mut self,
    position: Vector3,
    velocity: Vector3,
    mut renderer: Gd<SdfController>,
  ) {
    match renderer.bind_mut().new_shape(
      Vector4::new(position.x, position.y, position.z, 1.0),
      PROPERTIES,
      COLOR,
    ) {
      Ok(address) => self.shape_address = address,
      Err(e) => panic!("{}", e),
    };

    self.add_position(position);
    self.velocity = velocity;
  }

  pub fn add_position(&mut self, offset: Vector3) {
    let mut transform = self.base().get_transform();
    transform.origin += offset;
    self.base_mut().set_transform(transform);
  }
}
