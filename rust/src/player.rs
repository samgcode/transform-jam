use godot::{
  classes::{input::MouseMode, Input, InputEvent, InputEventMouseMotion},
  global::{Key, MouseButton},
  prelude::*,
};
use std::ops::Mul;

const MOVE_FORWARD: &str = "move_forward";
const MOVE_BACK: &str = "move_back";
const MOVE_LEFT: &str = "move_left";
const MOVE_RIGHT: &str = "move_right";

const JUMP: &str = "jump";

const SPEED: f32 = 2.0;
const JUMP_HEIGHT: f32 = 2.0;
const LOOK_SPEED: f32 = 0.002;

const GRAVITY: Vector3 = Vector3 {
  x: 0.0,
  y: -0.05,
  z: 0.0,
};

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct Player {
  #[base]
  base: Base<Node3D>,

  velocity: Vector3,
  mouse_captured: bool,
  look_rotation: Vector2,
}

#[godot_api]
impl INode3D for Player {
  fn init(base: Base<Node3D>) -> Self {
    return Self {
      base,

      velocity: Vector3::new(0.0, -0.5, 0.0),
      mouse_captured: false,
      look_rotation: Vector2::new(0.0, 0.0),
    };
  }

  fn ready(&mut self) {
    self.signals().collision().connect_self(Self::on_collision);
  }

  fn physics_process(&mut self, dt: f64) {
    let mut direction = Vector3::ZERO;

    let vertical = (
      Input::singleton().is_action_pressed(MOVE_FORWARD),
      Input::singleton().is_action_pressed(MOVE_BACK),
    );
    let horizontal = (
      Input::singleton().is_action_pressed(MOVE_LEFT),
      Input::singleton().is_action_pressed(MOVE_RIGHT),
    );

    if vertical.0 != vertical.1 {
      if vertical.0 {
        direction.z = -1.0;
      } else {
        direction.z = 1.0;
      }
    }

    if horizontal.0 != horizontal.1 {
      if horizontal.0 {
        direction.x = -1.0;
      } else {
        direction.x = 1.0;
      }
    }

    if direction != Vector3::ZERO {
      let direction = (self.base_mut().get_basis().mul(direction)).normalized() * SPEED;

      self.velocity.x = direction.x;
      self.velocity.z = direction.z;
    }

    if Input::singleton().is_action_just_pressed(JUMP) {
      self.velocity.y += JUMP_HEIGHT;
    }

    self.velocity += GRAVITY;
    self.add_position(self.velocity * dt as f32);
  }

  fn unhandled_input(&mut self, event: Gd<InputEvent>) {
    if Input::singleton().is_mouse_button_pressed(MouseButton::LEFT) {
      Input::singleton().set_mouse_mode(MouseMode::CAPTURED);
      self.mouse_captured = true;
    } else if Input::singleton().is_key_pressed(Key::ESCAPE) {
      Input::singleton().set_mouse_mode(MouseMode::VISIBLE);
      self.mouse_captured = false;
    }

    if self.mouse_captured {
      let mouse_event = event.try_cast::<InputEventMouseMotion>();
      if let Ok(mouse_event) = mouse_event {
        self.rotate_camera(mouse_event.get_relative());
      }
    }
  }
}

impl Player {
  pub fn on_collision(&mut self, collision: Vector4) {
    let normal = Vector3::new(collision.x, collision.y, collision.z).normalized();

    let parallel = Vector3::dot(self.velocity, normal) * normal;
    let perpendicular: Vector3 = self.velocity - parallel;

    self.add_position(normal * -collision.w);
    self.velocity = perpendicular * 0.1;
  }

  pub fn get_position(&self) -> Vector3 {
    return self.base().get_transform().origin;
  }

  pub fn add_position(&mut self, offset: Vector3) {
    let mut transform = self.base().get_transform();
    transform.origin += offset;
    self.base_mut().set_transform(transform);
  }

  fn rotate_camera(&mut self, input: Vector2) {
    let mut camera = self.base().get_node_as::<Node3D>("Camera");

    self.look_rotation.x = (self.look_rotation.x - input.y * LOOK_SPEED)
      .clamp(f32::to_radians(-85.0), f32::to_radians(85.0));
    let new_y = self.look_rotation.y - input.x * LOOK_SPEED;
    self.look_rotation.y = new_y;

    self
      .base_mut()
      .set_basis(Basis::IDENTITY.rotated(Vector3::new(0.0, 1.0, 0.0), new_y));
    camera.set_basis(Basis::IDENTITY.rotated(Vector3::new(1.0, 0.0, 0.0), self.look_rotation.x));
  }
}

#[godot_api]
impl Player {
  #[signal]
  pub fn collision(collision: Vector4);
}
