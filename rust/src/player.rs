use godot::{
  classes::{input::MouseMode, Input, InputEvent, InputEventMouseMotion},
  global::{Key, MouseButton},
  prelude::*,
};
use std::ops::Mul;

use crate::game_controller::GameController;

const MOVE_FORWARD: &str = "move_forward";
const MOVE_BACK: &str = "move_back";
const MOVE_LEFT: &str = "move_left";
const MOVE_RIGHT: &str = "move_right";
const INPUT_THROW: &str = "throw";
// const INPUT_SWAP: &str = "swap";

const JUMP: &str = "jump";

const SPEED: f32 = 2.0;
const AIR_SPEED: f32 = 3.0;
const JUMP_HEIGHT: f32 = 2.0;
const LOOK_SPEED: f32 = 0.002;

const GRAVITY: Vector3 = Vector3 {
  x: 0.0,
  y: -5.0,
  z: 0.0,
};

const GRENADE_BOOST: f32 = 15.0;
const GRENADE_DIR: Vector3 = Vector3 {
  x: 1.0,
  y: 0.5,
  z: 1.0,
};
const NUM_POINTS: usize = 6;

const MOMENTUM: f32 = 0.5;
const FAST_MOMENTUM: f32 = 0.75;
const FAST_THRESHOLD: f32 = 2.0;
const AIR_ACCELERATION: f32 = 30.0;

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct Player {
  #[base]
  base: Base<Node3D>,

  velocity: Vector3,
  mouse_captured: bool,
  look_rotation: Vector2,
  grounded: bool,
}

#[godot_api]
impl INode3D for Player {
  fn init(base: Base<Node3D>) -> Self {
    return Self {
      base,
      velocity: Vector3::new(0.0, 0.0, 0.0),
      mouse_captured: false,
      look_rotation: Vector2::new(0.0, 0.0),
      grounded: false,
    };
  }

  fn ready(&mut self) {
    self
      .signals()
      .update_pos()
      .connect_self(Self::on_update_pos);
    self.signals().explosion().connect_self(Self::on_explosion);
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
      let direction = (self.base_mut().get_basis().mul(direction)).normalized();
      if self.grounded {
        if self.velocity.x.abs() < direction.x.abs() {
          self.velocity.x = direction.x * SPEED;
        }
        if self.velocity.z.abs() < direction.z.abs() {
          self.velocity.z = direction.z * SPEED;
        }
      } else {
        let vertical = self.velocity.dot(Y_AXIS) * Y_AXIS;
        let horizontal = self.velocity - vertical;

        let move_force = direction * AIR_ACCELERATION * dt as f32;

        if horizontal.length() < AIR_SPEED {
          let target_velocity = horizontal + move_force;
          let target_velocity =
            target_velocity.normalized() * target_velocity.length().clamp(0.0, AIR_SPEED);

          self.velocity += target_velocity - horizontal;
        } else {
          let constrained_move_force = project_on_plane(move_force, horizontal.normalized());
          if horizontal.dot(move_force) > 0.0 {
            self.velocity += constrained_move_force;
          } else {
            self.velocity += move_force * 0.75;
          }
        }
      }
    }

    if self.grounded {
      if Input::singleton().is_action_just_pressed(JUMP) {
        self.velocity.y += JUMP_HEIGHT;
        self.grounded = false;
      }
    } else {
      self.velocity += GRAVITY * dt as f32;
    }

    if Input::singleton().is_action_just_pressed(INPUT_THROW) {
      let camera = self.base().get_node_as::<Node3D>("Camera");

      let direction = -camera.get_global_basis().col_c().normalized();

      self.game_controller().signals().spawn_grenade().emit(
        self.get_position() + Vector3::new(0.0, 0.4, 0.0) + direction * 0.4,
        direction,
      );
    }

    let vertical = self.velocity.dot(Y_AXIS) * Y_AXIS;
    let horizontal = self.velocity - vertical;

    godot_print!(
      "vertical: {:.5} horizontal: {:.5}",
      vertical.length(),
      horizontal.length()
    );
  }
}

const Y_AXIS: Vector3 = Vector3 {
  x: 0.0,
  y: 1.0,
  z: 0.0,
};

impl Player {
  pub fn on_update_pos(&mut self, dt: f32, shapecast: Vector4) {
    if shapecast.w < 1.0 {
      let normal = Vector3::new(shapecast.x, shapecast.y, shapecast.z).normalized();

      let free_velocity = self.velocity * shapecast.w * 0.9;
      let remaining_velocity = self.velocity - free_velocity;
      let slide_velocity = project_on_plane(remaining_velocity, normal);

      self.velocity = free_velocity + slide_velocity;

      let vertical = self.velocity.dot(Y_AXIS) * Y_AXIS;
      let horizontal = self.velocity - vertical;

      if self.velocity.length() < FAST_THRESHOLD {
        self.velocity = horizontal * MOMENTUM + vertical;
      } else {
        self.velocity = horizontal * FAST_MOMENTUM + vertical;
      }

      if normal.dot(Y_AXIS) > 0.5 {
        self.grounded = true;
      }
    } else {
      self.grounded = false;
    }

    self.add_position(self.velocity * dt);
  }

  fn on_explosion(&mut self, position: Vector3) {
    let vector = self.get_position() - position;
    let direction = GRENADE_DIR.normalized();
    let direction = vector.normalized() * direction;
    let distance = vector.length();

    if distance < 5.0 {
      self.velocity += direction * GRENADE_BOOST;
    }
  }

  pub fn get_position(&self) -> Vector3 {
    return self.base().get_transform().origin;
  }

  pub fn get_velocity(&self) -> Vector3 {
    return self.velocity;
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
    let mut camera: Gd<Node3D> = self.camera();

    self.look_rotation.x = (self.look_rotation.x - input.y * LOOK_SPEED)
      .clamp(f32::to_radians(-85.0), f32::to_radians(85.0));
    let new_y = self.look_rotation.y - input.x * LOOK_SPEED;
    self.look_rotation.y = new_y;

    self
      .base_mut()
      .set_basis(Basis::IDENTITY.rotated(Vector3::new(0.0, 1.0, 0.0), new_y));
    camera.set_basis(Basis::IDENTITY.rotated(Vector3::new(1.0, 0.0, 0.0), self.look_rotation.x));
  }

  fn game_controller(&mut self) -> Gd<GameController> {
    return self
      .base_mut()
      .get_parent()
      .unwrap()
      .cast::<GameController>();
  }

  fn camera(&mut self) -> Gd<Node3D> {
    return self.base_mut().get_node_as::<Node3D>("Camera");
  }
}

#[godot_api]
impl Player {
  #[signal]
  pub fn update_pos(dt: f32, shapecast: Vector4);
  #[signal]
  pub fn explosion(position: Vector3);
}

fn project_on_plane(vector: Vector3, normal: Vector3) -> Vector3 {
  return vector - normal * vector.dot(normal);
}
