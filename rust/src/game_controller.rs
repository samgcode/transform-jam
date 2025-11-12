use godot::prelude::*;

use crate::{
  grenade::{self, Grenade},
  player::Player,
  sdf_controller::{self, SdfController},
};

#[derive(GodotClass)]
#[class(base = Node3D)]
pub struct GameController {
  #[base]
  base: Base<Node3D>,

  grenade_scene: Gd<PackedScene>,
  grenades: Vec<(usize, Gd<Grenade>)>,

  #[export]
  player: Option<Gd<Player>>,
  #[export]
  sdf_controller: Option<Gd<SdfController>>,
}

#[godot_api]
impl INode3D for GameController {
  fn init(base: Base<Node3D>) -> Self {
    return Self {
      base,
      grenade_scene: load::<PackedScene>("res://grenade.tscn"),
      grenades: Vec::new(),

      player: None,
      sdf_controller: None,
    };
  }

  fn ready(&mut self) {
    self
      .signals()
      .spawn_grenade()
      .connect_self(Self::on_spawn_grenade);

    self
      .signals()
      .remove_grenade()
      .connect_self(Self::on_remove_grenade);
  }

  fn physics_process(&mut self, _dt: f64) {
    let player = self.player();
    let mut sdf_controller = self.sdf_controller();

    let player_collider = player.bind().get_points();
    let collision_events = self
      .sdf_controller()
      .bind_mut()
      .compute_collision(player_collider);

    let mut highest_depth = 0.0;
    let mut collision = Vector4::ZERO;
    for i in 0..collision_events.len() {
      if collision_events[i].w < highest_depth {
        highest_depth = collision.w;
        collision = collision_events[i];
      }
    }

    if collision != Vector4::ZERO {
      player.signals().collision().emit(collision);
    }

    if self.grenades.len() > 0 {
      let grenade_colliders = self.get_grenade_colliders();
      let collision_events = self
        .sdf_controller()
        .bind_mut()
        .compute_collision(grenade_colliders);

      for i in 0..self.grenades.len() {
        if collision_events[i].w < 0.0 {
          self.grenades[i]
            .1
            .signals()
            .collision()
            .emit(collision_events[i]);
        }
      }
    }

    let mut transform = sdf_controller.get_transform();
    transform.origin = player.get_transform().origin;
    sdf_controller.set_transform(transform);
    for i in 0..self.grenades.len() {
      if i >= self.grenades.len() {
        break;
      }

      let (i, grenade) = self.grenades[i].clone();
      if grenade.bind().exploded {
        self.on_remove_grenade(i as i32);
      } else {
        let position: Vector3 = grenade.bind().get_position();
        sdf_controller.bind_mut().update_shape(
          i.clone(),
          Vector4::new(
            position.x,
            position.y,
            position.z,
            sdf_controller::FLAG_NO_COLLISION,
          ),
          grenade::PROPERTIES,
          grenade::COLOR,
        );
      }
    }
  }
}

impl GameController {
  fn on_spawn_grenade(&mut self, position: Vector3, direction: Vector3) {
    let mut sdf_controller = self.sdf_controller();
    match sdf_controller.bind_mut().new_shape(
      Vector4::new(position.x, position.y, position.z, 1.0),
      grenade::PROPERTIES,
      grenade::COLOR,
    ) {
      Ok(address) => {
        let mut grenade = self.grenade_scene.instantiate_as::<Grenade>();

        grenade
          .bind_mut()
          .initialize(position, direction, address as i32);

        self.base_mut().add_child(&grenade);

        self.grenades.push((address, grenade));
      }
      Err(e) => panic!("{}", e),
    };
  }

  fn on_remove_grenade(&mut self, grenade_id: i32) {
    let mut sdf_controller = self.sdf_controller();

    let mut remove_id = 0;
    for i in 0..self.grenades.len() {
      let (id, mut grenade) = self.grenades[i].clone();

      if id.clone() == grenade_id as usize {
        sdf_controller.bind_mut().remove_shape(id.clone());
        remove_id = i;
        grenade.bind_mut().destroy();
      }
    }

    self.grenades.remove(remove_id);
  }

  fn get_grenade_colliders(&mut self) -> PackedVector4Array {
    let mut colliders = PackedVector4Array::new();
    self.grenades.iter().for_each(|(_, grenade)| {
      let position = grenade.bind().get_position();
      colliders.push(Vector4::new(
        position.x,
        position.y,
        position.z,
        sdf_controller::FLAG_NO_COLLISION,
      ));
    });

    return colliders;
  }

  fn player(&mut self) -> Gd<Player> {
    return self.base_mut().get_node_as::<Player>("Player");
  }

  fn sdf_controller(&mut self) -> Gd<SdfController> {
    return self
      .base_mut()
      .get_node_as::<SdfController>("SdfController");
  }
}

#[godot_api]
impl GameController {
  #[signal]
  pub fn spawn_grenade(position: Vector3, direction: Vector3);
  #[signal]
  pub fn remove_grenade(grenade_id: i32);
}
