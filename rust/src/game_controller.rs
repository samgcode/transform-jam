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
    let collision_event = self
      .sdf_controller()
      .bind_mut()
      .compute_collision(player_collider);

    if let Some(event) = collision_event {
      player.signals().collision().emit(event);
    }

    let mut transform = sdf_controller.get_transform();
    transform.origin = player.get_transform().origin;
    sdf_controller.set_transform(transform);

    self.grenades.iter().for_each(|(i, grenade)| {
      let position = grenade.bind().get_position();
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
    });
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

    let mut id = 0;
    let mut remove_id = 0;
    self.grenades.iter().for_each(|(i, _)| {
      if i.clone() == grenade_id as usize {
        sdf_controller.bind_mut().remove_shape(i.clone());
        remove_id = id;
      }
      id += 1;
    });

    self.grenades.remove(remove_id);
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
