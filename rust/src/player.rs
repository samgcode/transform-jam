use godot::{
  classes::{input::MouseMode, Input, InputEvent, InputEventMouseMotion},
  global::{Key, MouseButton},
  prelude::*,
};
use std::ops::Mul;

use crate::{grenade::Grenade, sdf_controller::SdfController};

const MOVE_FORWARD: &str = "move_forward";
const MOVE_BACK: &str = "move_back";
const MOVE_LEFT: &str = "move_left";
const MOVE_RIGHT: &str = "move_right";
const INPUT_THROW: &str = "throw";
// const INPUT_SWAP: &str = "swap";

const JUMP: &str = "jump";

const SPEED: f32 = 2.0;
const JUMP_HEIGHT: f32 = 2.0;
const LOOK_SPEED: f32 = 0.002;

const GRENADE_SPEED: f32 = 5.0;

const GRAVITY: Vector3 = Vector3 {
  x: 0.0,
  y: -0.05,
  z: 0.0,
};

const NUM_POINTS: usize = 6;

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct Player {
  #[base]
  base: Base<Node3D>,

  velocity: Vector3,
  mouse_captured: bool,
  look_rotation: Vector2,
  grounded: bool,

  grenade_scene: Gd<PackedScene>,
}

#[godot_api]
impl INode3D for Player {
  fn init(base: Base<Node3D>) -> Self {
    return Self {
      base,

      velocity: Vector3::new(0.0, -0.5, 0.0),
      mouse_captured: false,
      look_rotation: Vector2::new(0.0, 0.0),
      grounded: false,
      grenade_scene: load::<PackedScene>("res://grenade.tscn"),
    };
  }

  fn ready(&mut self) {
    self.signals().collision().connect_self(Self::on_collision);
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
      if self.grounded {
        self.velocity.y += JUMP_HEIGHT;
        self.grounded = false;
      }
    }

    self.velocity += GRAVITY;
    self.add_position(self.velocity * dt as f32);

    if Input::singleton().is_action_just_pressed(INPUT_THROW) {
      let mut grenade = self.grenade_scene.instantiate_as::<Grenade>();

      let camera = self.base().get_node_as::<Node3D>("Camera");
      let renderer = self.base().get_node_as::<SdfController>("SdfController");

      let direction = -camera.get_global_basis().col_c().normalized();

      grenade.bind_mut().initialize(
        self.get_position() + Vector3::new(0.0, 0.4, 0.0) + direction * 0.4,
        direction * GRENADE_SPEED,
        renderer,
      );

      self.base_mut().add_child(&grenade);
    }
  }
}

const Y_AXIS: Vector3 = Vector3 {
  x: 0.0,
  y: 1.0,
  z: 0.0,
};

impl Player {
  pub fn on_collision(&mut self, collision: Vector4) {
    let normal = Vector3::new(collision.x, collision.y, collision.z).normalized();

    let verticality = normal.dot(Y_AXIS);
    // let parallel = self.velocity.dot(normal) * normal;
    // let perpendicular: Vector3 = self.velocity - parallel;

    let vertical = self.velocity.dot(Y_AXIS) * Y_AXIS;
    let horizontal: Vector3 = self.velocity - vertical;

    self.add_position(normal * -collision.w);

    if verticality < 0.0 {
      // roof
      self.velocity = horizontal;
    } else if verticality < 0.5 {
      // wall
      self.velocity = vertical;
    } else {
      // floor
      self.velocity = horizontal * 0.1;

      self.grounded = true;
    }
  }

  pub fn get_position(&self) -> Vector3 {
    return self.base().get_transform().origin;
  }

  pub fn get_points(&self) -> PackedVector4Array {
    let pos = self.get_position();
    let feet = Vector4::new(pos.x, pos.y, pos.z, 1.0);

    let mut points = PackedVector4Array::from(vec![feet]);

    for i in 0..NUM_POINTS {
      let angle = i as f32 * (std::f32::consts::TAU / NUM_POINTS as f32);
      let x = angle.sin() * 0.1;
      let y = angle.cos() * 0.1;
      points.push(Vector4::new(x, 0.5, y, 0.0) + feet);
      points.push(Vector4::new(x, 0.3, y, 0.0) + feet);
      points.push(Vector4::new(x, 0.0, y, 0.0) + feet);
    }

    return points;
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
